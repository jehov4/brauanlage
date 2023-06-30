use std::sync::Arc;
use std::sync::Mutex;
use std::hash::{Hasher, Hash};

use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::{wrappers::ReceiverStream, wrappers::BroadcastStream, Stream, StreamExt};

use tonic::transport::Server;
use tonic::{Request, Response, Status};

use brauanlage::brauanlage_server::{Brauanlage, BrauanlageServer};
use brauanlage::{TempStatus, RelayStatus, Rcp, RcpStep, SetTempRequest, ToggleRelayRequest, Empty};

pub mod brauanlage {
    tonic::include_proto!("brauanlage");
}

#[derive(Debug)]
struct BrauanlageService {
    tempsChannels: Arc<Mutex<Sender<SetTempRequest>>>,
    relayChannels: Arc<Mutex<Sender<ToggleRelayRequest>>>,
    rcpChannels: Arc<Mutex<Sender<String>>>,
    tempsStatusBroadcast: Arc<Mutex<Vec<Sender<TempStatus>>>>,
    relayStatusBroadcast: Arc<Mutex<Vec<Sender<RelayStatus>>>>,
    rcpStatusBroadcast: Arc<Mutex<Vec<Sender<RcpStep>>>>, 
}

fn main() {
    println!("Hello, world!");
}

#[tonic::async_trait]
impl Brauanlage for BrauanlageService {
    async fn send_rcp(&self, _request: Request<Rcp>) -> Result<Response<Rcp>, Status> {
        unimplemented!();
    }

    async fn get_rcp(&self, _request: Request<Empty>) -> Result<Response<Rcp>, Status> {
        unimplemented!();
    }
    
    async fn start_step(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        unimplemented!();
    }
    
    async fn skip_step(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        unimplemented!();
    }
    
    async fn set_temp(&self, _request: Request<SetTempRequest>) -> Result<Response<Empty>, Status> {
        unimplemented!();
    }
    
    async fn toggle_relay(&self, _request: Request<ToggleRelayRequest>) -> Result<Response<Empty>, Status> {
        unimplemented!();
    }

    async fn get_temp_status(&self, _request: Request<TempStatus>) -> Result<Response<TempStatus>, Status> {
        unimplemented!();
    }

    async fn get_relay_status(&self, _request: Request<RelayStatus>) -> Result<Response<RelayStatus>, Status> {
        unimplemented!();
    }

    async fn get_rcp_status(&self, _request: Request<RcpStep>) -> Result<Response<RcpStep>, Status> {
        unimplemented!();
    }
}

impl Hash for RcpStep { 
    fn hash<H>(&self, state: &mut H)
    where H: Hasher,
    {
       self.index.hash(state);
       self.started.hash(state);
    }
}
