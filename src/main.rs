use std::hash::{Hash, Hasher};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use tokio::sync::Mutex;

use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::{self, WeakSender};
use tokio::sync::watch;
use tokio::sync::watch::Receiver as WReceiver;
use tokio::sync::watch::Sender as WSender;
use tokio_stream::wrappers::WatchStream;

use tonic::transport::Server;
use tonic::{Request, Response, Result, Status};

use brauanlage_service::service::{BrauanlageService, BrauCommand};
use brauanlage::{Empty, Rcp, RcpStatus, RcpStep, RelayStatus, TempStatus};

mod brauanlage_service;
mod brauanlage;


#[derive(Debug)]
struct BrauService {
    command_receiver: Receiver<BrauCommand>,
    temps_sender: WSender<Result<TempStatus, Status>>,
    relay_sender: WSender<RelayStatus>,
    rcp_sender: WSender<Rcp>,
}

#[tokio::main]
async fn main() {
    let brauanlage_service = BrauanlageService::new();
}

