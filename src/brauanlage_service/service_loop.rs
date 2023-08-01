use tokio::sync::{mpsc::Receiver, Mutex};

use std::sync::Arc;

use tokio::sync::watch::{Sender, self};
use tokio::sync::watch::Receiver as WReceiver;
use tonic::Status;

use itertools::izip;

use super::{brauanlage::{TempStatus, RelayStatus, Rcp, RcpStatus}, service::BrauCommand};
use super::peripheral::Peripheral;

struct ServiceLoop {
    temps_sender: Arc<Mutex<Sender<Result<TempStatus, Status>>>>,
    relay_sender: Mutex<Sender<RelayStatus>>,
    rcp_sender: Mutex<Sender<Rcp>>,
    command_receiver: Mutex<Receiver<BrauCommand>>,
}

impl ServiceLoop {
    pub async fn new(
            temps_sender: Arc<Mutex<Sender<Result<TempStatus, Status>>>>,
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

        // channel for control commands from frontend
        let mut command_receiver = self.command_receiver.lock().await;

        // internal channel for relay control state irc
        let (irc_sender, irc_receiver) = watch::channel(RelayStatus {stati: vec!(false,false)});
        // internal channel for temperature control state itc
        let (itc_sender, itc_receiver) = watch::channel(TempStatus {temps: vec!(0,0,0)});

        let temps_sender = self.temps_sender.clone();

        // start control loops
        tokio::task::spawn(Self::temp_loop(itc_receiver, temps_sender));
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

    fn calc_switch_operations (goal: &TempStatus, current: &TempStatus) -> Vec<bool> {
        let heating_buffer = 1;
        let cooling_buffer = 1;
        let mut operations = Vec::new();
        for (goalv, currentv) in izip!(&goal.temps, &current.temps) {
            if *currentv > *goalv + heating_buffer {
                operations.push(false)
            }
            if *currentv < *goalv - cooling_buffer {
                operations.push(true)
            }
        }
        operations
    }

    async fn temp_loop(mut control: WReceiver<TempStatus>, sender: Arc<Mutex<Sender<Result<TempStatus, Status>>>>){
        let mut goal = control.borrow().clone();
        let inner_sender = sender.lock().await;
        loop {
           let current = Self::get_temps();
           Peripheral::set_relays(vec!(1,2,3), Self::calc_switch_operations(&goal, &current));
           inner_sender.send(Ok(current));
           if control.has_changed().unwrap() {
              goal = control.borrow_and_update().clone(); 
           }
        }
        
    }
    
    async fn relay_loop(mut control: WReceiver<RelayStatus>){
        loop {
            Peripheral::set_relays(vec!(1,2), control.borrow().stati.clone());
            let fut = control.changed().await; 
            if fut.is_err()  {
                println!("Something went wrong waiting for relay Updates")
            };
        }
    }

}

