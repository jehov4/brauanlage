use tokio::sync::{mpsc::Receiver, Mutex};

use tokio::sync::watch::Sender;
use tonic::Status;

use super::{brauanlage::{TempStatus, RelayStatus, Rcp}, service::BrauCommand};


struct ServiceLoop {
    temps_sender: Mutex<Sender<Result<TempStatus, Status>>>,
    relay_sender: Mutex<Sender<RelayStatus>>,
    rcp_sender: Mutex<Sender<Rcp>>,
    command_receiver: Mutex<Receiver<BrauCommand>>,

}

impl ServiceLoop {
    pub async fn new(
            temps_sender: Mutex<Sender<Result<TempStatus, Status>>>,
            relay_sender: Mutex<Sender<RelayStatus>>,
            rcp_sender: Mutex<Sender<Rcp>>,
            command_receiver: Mutex<Receiver<BrauCommand>>,
        ) -> ServiceLoop {
        ServiceLoop {
            temps_sender: temps_sender,
            relay_sender: relay_sender,
            rcp_sender: rcp_sender,
            command_receiver: command_receiver,
        }
    }
    pub async fn service_loop(&self) {
        let temps_sender = self.temps_sender.lock().await;
        let rcp_sender = self.rcp_sender.lock().await;
        let relay_sender = self.relay_sender.lock().await;
        let mut command_receiver = self.command_receiver.lock().await;

        let command = command_receiver.try_recv();
        if command.is_ok() {
            unimplemented!()
        }
    }
    fn get_temps() -> TempStatus {
        unimplemented!()
    }

}

