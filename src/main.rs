use std::sync::Arc;
use std::sync::Mutex;
use std::hash::{Hasher, Hash};

use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::watch;
use tokio::sync::watch::Receiver as WReceiver;

use tonic::transport::Server;
use tonic::{Request, Response, Status};

use brauanlage::brauanlage_server::{Brauanlage, BrauanlageServer};
use brauanlage::{TempStatus, RelayStatus, Rcp, RcpStep, Empty};

pub mod brauanlage {
    tonic::include_proto!("brauanlage");
}

#[derive(Debug)]
struct BrauanlageService {
    temps_sender: Arc<Mutex<Sender<TempStatus>>>,
    relay_sender: Arc<Mutex<Sender<RelayStatus>>>,
    rcp_sender: Arc<Mutex<Sender<RcpCommand>>>,
    temps_receiver: Arc<Mutex<WReceiver<TempStatus>>>,
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
   let (temp_sender,temp_receiver) = mpsc::channel(16);
   let (relay_sender, relay_receiver) = mpsc::channel(16); 
   let (rcp_sender,rcp_receiver) = mpsc::channel(16);

   let (temps_watch_sender,temps_watch_receiver) = 
       watch::channel(TempStatus { temps: vec![0,0,0] });
   let (relay_watch_sender,relay_watch_receiver) = 
       watch::channel( RelayStatus { stati: vec![false,false] });
   let (rcp_watch_sender, rcp_watch_receiver) = 
       watch::channel(RcpStep { 
           index: -1, 
           started: false, 
           time_started: -1, 
           duration: -1, 
           temps: vec![0,0,0], 
           relays: vec![false,false], 
           autostart: false });

   let brauanlage = new_brauanlage(temp_sender,relay_sender,rcp_sender,temps_watch_receiver,relay_watch_receiver,rcp_watch_receiver); 
}


fn new_brauanlage(
    temp_sender: Sender<TempStatus>, 
    relay_sender:Sender<RelayStatus>, 
    rcp_sender: Sender<RcpCommand>,
    temp_receiver: WReceiver<TempStatus>,
    relay_receiver: WReceiver<RelayStatus>,
    rcp_receiver: WReceiver<RcpStep>) -> BrauanlageService 
{
    BrauanlageService { 
        temps_sender: Arc::new(Mutex::new(temp_sender)), 
        relay_sender: Arc::new(Mutex::new(relay_sender)), 
        rcp_sender: Arc::new(Mutex::new(rcp_sender)), 
        temps_receiver: Arc::new(Mutex::new(temp_receiver)), 
        relay_receiver: Arc::new(Mutex::new(relay_receiver)), 
        rcp_receiver: Arc::new(Mutex::new(rcp_receiver)) }
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
    
    async fn set_temp(&self, _request: Request<TempStatus>) -> Result<Response<Empty>, Status> {
        unimplemented!();
    }
    
    async fn toggle_relay(&self, _request: Request<RelayStatus>) -> Result<Response<Empty>, Status> {
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
