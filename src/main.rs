use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use tokio::sync::Mutex;

use tokio::sync::mpsc::{self, WeakSender};
use tokio::sync::mpsc::Sender;
use tokio::sync::watch;
use tokio::sync::watch::Receiver as WReceiver
use tokio::sync::watch::Sender as WSender;
use tokio_stream::wrappers::WatchStream;

use tonic::transport::Server;
use tonic::{Request, Response, Result, Status};

use brauanlage::brauanlage_server::{Brauanlage, BrauanlageServer};
use brauanlage::{Empty, Rcp, RcpStatus, RcpStep, RelayStatus, TempStatus};

pub mod brauanlage {
    tonic::include_proto!("brauanlage");
}

#[derive(Debug)]
struct BrauanlageService {
    command_sender: Arc<Mutex<Sender<BrauCommand>>>,
    temps_receiver: Arc<Mutex<WReceiver<Result<TempStatus, Status>>>>,
    relay_receiver: Arc<Mutex<WReceiver<RelayStatus>>>,
    rcp_receiver: Arc<Mutex<WReceiver<Rcp>>>,
}

#[derive(Debug)]
struct BrauService {
    command_receiver: Receiver<BrauCommand>,
    temps_sender: WSender<Result<TempStatus, Status>>,
    relay_sender: WSender<RelayStatus>,
    rcp_sender: WSender<Rcp>,
}

enum BrauCommand {
    Next,
    Start,
    Pause,
    TakeRcp(Rcp),
    UpdateTemp(TempStatus),
    UpdateRelays(RelayStatus),
}

#[tokio::main]
async fn main() {
    // setup channels for communication
    //
    let (command_sender, command_receiver) = mpsc::channel(16);

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

    let brauanlage = BrauanlageService::new(
        command_sender,
        temps_watch_receiver,
        relay_watch_receiver,
        rcp_watch_receiver,
    );
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
    pub fn new(
        command_sender: Sender<BrauCommand>,
        temp_receiver: WReceiver<Result<TempStatus, Status>>,
        relay_receiver: WReceiver<RelayStatus>,
        rcp_receiver: WReceiver<Rcp>,
    ) -> BrauanlageService {
        BrauanlageService {
            command_sender: Arc::new(Mutex::new(command_sender)),
            temps_receiver: Arc::new(Mutex::new(temp_receiver)),
            relay_receiver: Arc::new(Mutex::new(relay_receiver)),
            rcp_receiver: Arc::new(Mutex::new(rcp_receiver)),
        }
    }
}

#[tonic::async_trait]
impl Brauanlage for BrauanlageService {
    async fn send_rcp(&self, _request: Request<Rcp>) -> Result<Response<Rcp>, Status> {
        self.command_sender
            .lock()
            .await
            .send(BrauCommand::TakeRcp(_request.into_inner()));
        Ok(Response::new(self.get_rcp_status_inner().await))
    }

    async fn get_rcp(&self, _request: Request<Empty>) -> Result<Response<Rcp>, Status> {
        unimplemented!();
    }

    async fn start_rcp(&self, _request: Request<Empty>) -> Result<Response<Rcp>, Status> {
        self.command_sender.lock().await.send(BrauCommand::Start);
        Ok(Response::new(self.get_rcp_status_inner().await))
    }

    async fn skip_step(&self, _request: Request<Empty>) -> Result<Response<Rcp>, Status> {
        self.command_sender.lock().await.send(BrauCommand::Next);
        Ok(Response::new(self.get_rcp_status_inner().await))
    }

    async fn set_temp(&self, _request: Request<TempStatus>) -> Result<Response<Empty>, Status> {
        self.command_sender
            .lock()
            .await
            .send(BrauCommand::UpdateTemp(_request.into_inner()));
        Ok(Response::new(Empty {}))
    }

    async fn toggle_relay(
        &self,
        _request: Request<RelayStatus>,
    ) -> Result<Response<RelayStatus>, Status> {
        self.command_sender
            .lock()
            .await
            .send(BrauCommand::UpdateRelays(_request.into_inner()));
        let response = self.get_relay_status_inner().await;
        Ok(Response::new(response))
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
        let response = self.get_relay_status_inner().await;
        Ok(Response::new(response))
    }

    async fn get_rcp_status(&self, _request: Request<Empty>) -> Result<Response<Rcp>, Status> {
        Ok(Response::new(self.get_rcp_status_inner().await))
    }
}
