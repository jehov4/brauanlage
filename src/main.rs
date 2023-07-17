use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::Mutex;

use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::watch;
use tokio::sync::watch::Receiver as WReceiver;
use tokio_stream::wrappers::WatchStream;

use tonic::transport::Server;
use tonic::{Request, Response, Result, Status};

use brauanlage::brauanlage_server::{Brauanlage, BrauanlageServer};
use brauanlage::{Empty, Rcp, RcpStatus, RcpStep, RelayStatus, TempStatus};

pub mod brauanlage {
    tonic::include_proto!("brauanlage");
}

mod brauanlage_service;

#[derive(Debug)]
struct BrauanlageService {
    temps_sender: Arc<Mutex<Sender<TempStatus>>>,
    relay_sender: Arc<Mutex<Sender<RelayStatus>>>,
    rcp_sender: Arc<Mutex<Sender<RcpCommand>>>,
    temps_receiver: Arc<Mutex<WReceiver<Result<TempStatus, Status>>>>,
    relay_receiver: Arc<Mutex<WReceiver<RelayStatus>>>,
    rcp_receiver: Arc<Mutex<WReceiver<Rcp>>>,
}

enum RcpCommand {
    Next,
    Start,
    Pause,
    TakeRcp(Rcp),
}

#[tokio::main]
async fn main() {
    // setup channels for communication
    //
    let (temp_sender, temp_receiver) = mpsc::channel(16);
    let (relay_sender, relay_receiver) = mpsc::channel(16);
    let (rcp_sender, rcp_receiver) = mpsc::channel(16);

    let (temps_watch_sender, temps_watch_receiver) = watch::channel(Ok(TempStatus {
        temps: vec![0, 0, 0],
    }));
    let (relay_watch_sender, relay_watch_receiver) = watch::channel(RelayStatus {
        stati: vec![false, false],
    });
    let (rcp_watch_sender, rcp_watch_receiver) = watch::channel(Rcp {
        steps: Vec::new(),
        status: RcpStatus::Initial.into(),
        step_started_timestamp: 0,
    });

    let brauanlage = new_brauanlage(
        temp_sender,
        relay_sender,
        rcp_sender,
        temps_watch_receiver,
        relay_watch_receiver,
        rcp_watch_receiver,
    );
}

fn new_brauanlage(
    temp_sender: Sender<TempStatus>,
    relay_sender: Sender<RelayStatus>,
    rcp_sender: Sender<RcpCommand>,
    temp_receiver: WReceiver<Result<TempStatus, Status>>,
    relay_receiver: WReceiver<RelayStatus>,
    rcp_receiver: WReceiver<Rcp>,
) -> BrauanlageService {
    BrauanlageService {
        temps_sender: Arc::new(Mutex::new(temp_sender)),
        relay_sender: Arc::new(Mutex::new(relay_sender)),
        rcp_sender: Arc::new(Mutex::new(rcp_sender)),
        temps_receiver: Arc::new(Mutex::new(temp_receiver)),
        relay_receiver: Arc::new(Mutex::new(relay_receiver)),
        rcp_receiver: Arc::new(Mutex::new(rcp_receiver)),
    }
}

impl BrauanlageService {
    async fn get_rcp_status_inner(&self) -> Rcp {
        let receiver_clone = self.rcp_receiver.lock().await.clone();
        let response = receiver_clone.borrow().clone();
        response
    }

    async fn get_relay_status_inner(&self) -> RelayStatus {
        let receiver_clone = self.relay_receiver.lock().await.clone();
        let response = receiver_clone.borrow().clone();
        response
    }
}
