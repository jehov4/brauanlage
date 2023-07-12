use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::Mutex;

use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::watch;
use tokio::sync::watch::Receiver as WReceiver;
use tokio_stream::wrappers::WatchStream;

use tonic::codegen::http::status;
use tonic::transport::Server;
use tonic::{Request, Response, Result, Status};

use brauanlage::brauanlage_server::{Brauanlage, BrauanlageServer};
use brauanlage::{Empty, Rcp, RcpStep, RelayStatus, TempStatus};

pub mod brauanlage {
    tonic::include_proto!("brauanlage");
}

#[derive(Debug)]
struct BrauanlageService {
    temps_sender: Arc<Mutex<Sender<TempStatus>>>,
    relay_sender: Arc<Mutex<Sender<RelayStatus>>>,
    rcp_sender: Arc<Mutex<Sender<RcpCommand>>>,
    temps_receiver: Arc<Mutex<WReceiver<Result<TempStatus, Status>>>>,
    relay_receiver: Arc<Mutex<WReceiver<RelayStatus>>>,
    rcp_receiver: Arc<Mutex<WReceiver<RcpStep>>>,
}

enum RcpCommand {
    Next,
    Start,
    Pause,
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
    let (rcp_watch_sender, rcp_watch_receiver) = watch::channel(RcpStep {
        index: -1,
        started: false,
        time_started: -1,
        duration: -1,
        temps: vec![0, 0, 0],
        relays: vec![false, false],
        autostart: false,
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
    rcp_receiver: WReceiver<RcpStep>,
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

#[tonic::async_trait]
impl Brauanlage for BrauanlageService {
    async fn send_rcp(&self, _request: Request<Rcp>) -> Result<Response<Rcp>, Status> {
        unimplemented!();
    }

    async fn get_rcp(&self, _request: Request<Empty>) -> Result<Response<Rcp>, Status> {
        unimplemented!();
    }

    async fn start_step(&self, _request: Request<Empty>) -> Result<Response<RcpStep>, Status> {
        unimplemented!();
    }

    async fn skip_step(&self, _request: Request<Empty>) -> Result<Response<RcpStep>, Status> {
        unimplemented!();
    }

    async fn set_temp(
        &self,
        _request: Request<TempStatus>,
    ) -> Result<Response<TempStatus>, Status> {
        unimplemented!();
    }

    async fn toggle_relay(
        &self,
        _request: Request<RelayStatus>,
    ) -> Result<Response<RelayStatus>, Status> {
        unimplemented!();
    }

    type GetTempStatusStream = WatchStream<Result<TempStatus, Status>>;

    async fn get_temp_status(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::GetTempStatusStream>, Status> {
        let stream_clone = self.temps_receiver.lock().await.clone();
        Ok(Response::new(WatchStream::new(stream_clone)))
    }

    async fn get_relay_status(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<RelayStatus>, Status> {
        let receiver_clone = self.relay_receiver.lock().await.clone();
        let response = receiver_clone.borrow().clone();
        Ok(Response::new(response))
    }

    async fn get_rcp_status(&self, _request: Request<Empty>) -> Result<Response<RcpStep>, Status> {
        let receiver_clone = self.rcp_receiver.lock().await.clone();
        let response = receiver_clone.borrow().clone();
        Ok(Response::new(response))
    }
}
