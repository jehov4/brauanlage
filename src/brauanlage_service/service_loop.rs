use tokio::sync::{mpsc::Receiver, Mutex};

use std::sync::Arc;
use std::time::SystemTime;

use tokio::sync::watch::{Sender, self};
use tokio::sync::watch::Receiver as WReceiver;
use tonic::Status;

use itertools::izip;

use super::brauanlage::RcpStep;
use super::service::AnlagenStatus;
use super::{brauanlage::{TempStatus, RelayStatus, Rcp, RcpStatus}, service::BrauCommand};
use super::peripheral::{Peripheral, self};

struct ServiceLoop {
    temps_sender: Arc<Mutex<Sender<Result<TempStatus, Status>>>>,
    anlagen_sender: Mutex<Sender<AnlagenStatus>>,
    command_receiver: Mutex<Receiver<BrauCommand>>,
}

impl ServiceLoop {
    pub async fn new(
            temps_sender: Arc<Mutex<Sender<Result<TempStatus, Status>>>>,
            anlagen_sender: Mutex<Sender<AnlagenStatus>>,
            command_receiver: Mutex<Receiver<BrauCommand>>,
        ) -> ServiceLoop {
        ServiceLoop {
            temps_sender: temps_sender,
            anlagen_sender: anlagen_sender,
            command_receiver: command_receiver,
        }
    }
    pub async fn service_loop(&self) {
        let mut rcp_status = Rcp {
            steps: Vec::new(),
            status: RcpStatus::Uninitialized.into(),
            step_started_timestamp: 0,
            step_index: 0,
        };

        // TODO: Pins Hardcoded
        let peripheral_controller = Peripheral::new(vec![1,2,3], vec![4,5]);

        // channel for control commands from frontend
        let mut command_receiver = self.command_receiver.lock().await;

        // internal channel for relay control state irc
        let (mut irc_sender, irc_receiver) = watch::channel(RelayStatus {stati: vec!(false,false)});
        // internal channel for temperature control state itc
        let (mut itc_sender, itc_receiver) = watch::channel(TempStatus {temps: vec!(0,0,0)});
        // internal channel for communication from temperature task to relay task
        // TODO: Potential Command Loss if delay in relay swiching operations!!
        let (mut trc_sender, trc_receiver) = watch::channel(RelayStatus {stati: vec!(false,false)});

        let temps_sender = self.temps_sender.clone();

        // start control loops
        tokio::task::spawn(Self::temp_loop(itc_receiver.clone(), trc_sender, temps_sender));
        tokio::task::spawn(Self::relay_loop(irc_receiver.clone(),trc_receiver, peripheral_controller));

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
                Self::check_rcp_status(&mut rcp_status, &mut itc_sender, &mut irc_sender );
                Self::send_status(&self.anlagen_sender, &irc_receiver, &itc_receiver, &rcp_status); 
            }
        }
    }

    async fn send_status(sender: &Mutex<Sender<AnlagenStatus>>, relays: &WReceiver<RelayStatus>, temps_set: &WReceiver<TempStatus>, rcp: &Rcp) {
        let status = AnlagenStatus {
           relays: relays.borrow().clone(),
           rcp: rcp.clone(),
           temps_set: temps_set.borrow().clone(),
        };
        sender.lock().await.send(status);
    }

    fn get_temps() -> TempStatus {
        unimplemented!()
    }

    fn calc_switch_operations (goal: &TempStatus, current: &TempStatus) -> RelayStatus {
        let heating_buffer = 1;
        let cooling_buffer = 1;
        let mut operations = RelayStatus { stati: vec![] };
        // calculate whether a relay has to be switched for each temperature, the first 2 relays
        // control the temerature sensitive heaters, the rest (3) are either full or none
        // (boiling/pumps)
        for (goalv, currentv) in izip!(&goal.temps, &current.temps) {
            if *currentv > *goalv + heating_buffer {
                operations.stati.push(false)
            }
            if *currentv < *goalv - cooling_buffer {
                operations.stati.push(true)
            }
        }
        operations
    }

    async fn temp_loop(mut control: WReceiver<TempStatus>, relay_control: Sender<RelayStatus>, sender: Arc<Mutex<Sender<Result<TempStatus, Status>>>>){
        let mut goal = control.borrow().clone();
        let inner_sender = sender.lock().await;
        loop {
           let current = Self::get_temps();
           relay_control.send(Self::calc_switch_operations(&goal, &current));
           // Send update into Stream
           inner_sender.send(Ok(current));
           if control.has_changed().unwrap() {
              goal = control.borrow_and_update().clone(); 
           }
        }
        
    }
    
    async fn relay_loop(mut control: WReceiver<RelayStatus>, mut temp_control: WReceiver<RelayStatus>, mut peripheral_controller: Peripheral){
        loop {
            // TODO: check on unwraps
            if control.has_changed().unwrap() {
                peripheral_controller.set_relays(control.borrow_and_update().clone());
            } 
            if temp_control.has_changed().unwrap() {                
                peripheral_controller.set_relays(temp_control.borrow_and_update().clone());
            }
        }
    }


    fn check_rcp_status(rcp: &mut Rcp, temps_channel: &Sender<TempStatus>, relay_channel: &mut Sender<RelayStatus>) {
        if rcp.status == RcpStatus::Started.into() {
            // Get a reference to the currently active step
            let mut active_step = rcp.steps.get(usize::try_from(rcp.step_index).unwrap()).unwrap();
            // Check whether the duration of the current step is due
            if rcp.step_started_timestamp + active_step.duration > i32::try_from(Self::get_secs()).unwrap()  {
                // check whether its the last step
                if i32::try_from(rcp.steps.len()).unwrap() <= rcp.step_index {
                    rcp.status = RcpStatus::Finished.into();
                } else {
                    rcp.step_index = rcp.step_index + 1;
                    active_step = rcp.steps.get(usize::try_from(rcp.step_index).unwrap()).unwrap();
                    // Send out updates in case the current step is autostart
                    if active_step.autostart {
                        Self::send_updates(active_step, temps_channel, relay_channel);
                        rcp.step_started_timestamp = i32::try_from(Self::get_secs()).unwrap();
                    } else {
                        rcp.status = RcpStatus::Paused.into();
                    }
                }
            } 
        }
    }

    // Send out the internal updates to perform peripheral actions
    fn send_updates(active_step: &RcpStep, temps_channel: &Sender<TempStatus>, relay_channel: &mut Sender<RelayStatus>) {
        temps_channel.send(TempStatus { temps: active_step.temps.clone()});
        relay_channel.send(RelayStatus { stati: active_step.relays.clone()});
    }

    fn get_secs() -> u64 {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
    }

}

