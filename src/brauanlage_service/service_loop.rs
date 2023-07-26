use tokio::sync::{mpsc::Receiver, Mutex};

use tokio::sync::watch::{Sender, self};
use tokio::sync::watch::Receiver as WReceiver;
use tonic::Status;

use super::{brauanlage::{TempStatus, RelayStatus, Rcp, RcpStatus}, service::BrauCommand};


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
        let mut rcp_status = Rcp {
            steps: Vec::new(),
            status: RcpStatus::Uninitialized.into(),
            step_started_timestamp: 0,
        };

        // channels for status communication to frontent
        let temps_sender = self.temps_sender.lock().await;
        let rcp_sender = self.rcp_sender.lock().await;
        let relay_sender = self.relay_sender.lock().await;

        // channel for control commands from frontend
        let mut command_receiver = self.command_receiver.lock().await;

        // internal channel for relay control state irc
        let (irc_sender, irc_receiver) = watch::channel(RelayStatus {stati: vec!(false,false)});
        // internal channel for temperature control state itc
        let (itc_sender, itc_receiver) = watch::channel(TempStatus {temps: vec!(0,0,0)});

        // start control loops
        tokio::task::spawn(Self::temp_loop(itc_receiver));
        tokio::task::spawn(Self::relay_loop(irc_receiver));

        loop {
            let command = command_receiver.try_recv();
            if command.is_ok() {
                let foo = match command.unwrap() {
                    BrauCommand::UpdateTemp(status) => itc_sender.send(status).is_ok(),
                    BrauCommand::UpdateRelays(status) => irc_sender.send(status).is_ok(),
                    BrauCommand::Next => unimplemented!(),
                    BrauCommand::Start => unimplemented!(),
                    BrauCommand::Pause => unimplemented!(),
                    BrauCommand::TakeRcp(rcp) => {
                        rcp_status.steps.clone_from(&rcp.steps);
                        true
                    }
                };

            }
        }
    }
    fn get_temps() -> TempStatus {
        unimplemented!()
    }
    async fn temp_loop(control: WReceiver<TempStatus>){
        unimplemented!()
    }
    async fn relay_loop(control: WReceiver<RelayStatus>){
        unimplemented!()
    }

}

