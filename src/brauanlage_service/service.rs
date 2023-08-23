use std::sync::Arc;
use tokio::sync::Mutex;

use tokio::sync::mpsc::{self,Sender,Receiver};
use tokio::sync::watch;
use tokio::sync::watch::Receiver as WReceiver;
use tokio::sync::watch::Sender as WSender;
use tokio_stream::wrappers::WatchStream;

use tonic::codegen::http::response;
use tonic::transport::Server;
use tonic::{Request, Response, Result, Status};

use super::brauanlage::brauanlage_server::{Brauanlage, BrauanlageServer};
use super::brauanlage::{Empty, Rcp, RcpStatus, RcpStep, RelayStatus, TempStatus};

#[derive(Debug)]
pub struct BrauanlageService {
    command_sender: Arc<Mutex<Sender<BrauCommand>>>,
    temps_receiver: Arc<Mutex<WReceiver<Result<TempStatus, Status>>>>,
    status_receiver: Arc<Mutex<WReceiver<AnlagenStatus>>>,
    command_receiver: Arc<Mutex<Receiver<BrauCommand>>>,
}

#[derive(Debug)]
pub struct AnlagenStatus {
    pub relays: RelayStatus,
    pub temps_set: TempStatus,
    pub rcp: Rcp,
} 

pub enum BrauCommand {
    Next,
    Start,
    Pause,
    TakeRcp(Rcp),
    UpdateTemp(TempStatus),
    UpdateRelays(RelayStatus),
}

impl BrauanlageService {
    async fn get_rcp_status_inner(&self) -> Rcp {
        let receiver_clone = self.status_receiver.lock().await.clone();
        let response = receiver_clone.borrow().rcp.clone();
        response
    }

    async fn get_relay_status_inner(&self) -> RelayStatus {
        let receiver_clone = self.status_receiver.lock().await.clone();
        let response = receiver_clone.borrow().relays.clone();
        response
    }

    async fn get_temps_set_inner(&self) -> TempStatus {
        let receiver_clone = self.status_receiver.lock().await.clone();
        let response = receiver_clone.borrow().temps_set.clone();
        response
    }

    pub fn new() -> BrauanlageService {

        let (command_sender, command_receiver) = mpsc::channel(16);

        let (temps_watch_sender, temps_watch_receiver) = watch::channel(Ok(TempStatus {
            temps: vec![0, 0, 0],
        }));
        let (status_watch_sender, status_watch_receiver) = watch::channel(AnlagenStatus {
                    rcp: Rcp { steps: Vec::new(), status: RcpStatus::Initial.into(), step_started_timestamp: 0, step_index: 0, },
                    temps_set: TempStatus{temps: vec![0,0]},
                    relays: RelayStatus{stati: vec![false, false]},
                }
            );
        let (relay_watch_sender, relay_watch_receiver) = watch::channel(RelayStatus {
            stati: vec![false, false],
        });
        let (rcp_watch_sender, rcp_watch_receiver) = watch::channel(Rcp {
            steps: Vec::new(),
            status: RcpStatus::Initial.into(),
            step_started_timestamp: 0,
            step_index: 0,
        });

        BrauanlageService {
            command_sender: Arc::new(Mutex::new(command_sender)),
            temps_receiver: Arc::new(Mutex::new(temps_watch_receiver)),
            status_receiver: Arc::new(Mutex::new(status_watch_receiver)),
            command_receiver: Arc::new(Mutex::new(command_receiver)),
        }
    }
}

#[tonic::async_trait]
impl Brauanlage for BrauanlageService {
    async fn send_rcp(&self, _request: Request<Rcp>) -> Result<Response<Rcp>, Status> {
        self.command_sender.lock().await.send(BrauCommand::TakeRcp(_request.into_inner()));
        Ok(Response::new(self.get_rcp_status_inner().await))
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
        self.command_sender.lock().await.send(BrauCommand::UpdateTemp(_request.into_inner()));
        Ok(Response::new(Empty {}))
    }

    async fn toggle_relay(
        &self,
        _request: Request<RelayStatus>,
    ) -> Result<Response<RelayStatus>, Status> {
        self.command_sender.lock().await.send(BrauCommand::UpdateRelays(_request.into_inner()));
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
        let response = self.get_rcp_status_inner().await;
        Ok(Response::new(response))
    }
}

