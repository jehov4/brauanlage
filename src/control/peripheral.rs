use std::borrow::BorrowMut;
use std::{thread, time};
use rppal::gpio::Gpio;

struct Schuetz {
    enabled: bool,
    trigger_pin: u8,
}


impl Schuetz {
    pub fn trigger (&mut self) {
        let pin = self.trigger_pin;
        thread::spawn(move || {
            let mut trigger = Gpio::new().unwrap().get(pin).unwrap().into_output();
            trigger.set_high();
            thread::sleep(time::Duration::from_millis(50));
            trigger.set_low();        
        });
        self.enabled = !self.enabled
    }
}

pub struct Peripheral {
    temps: Vec<Schuetz>,
    pumps: Vec<Schuetz>,
}

impl Peripheral {
    pub fn get_temperatures(&self) -> Vec<f32> {
        unimplemented!();
    }
    pub fn switch_temperature_relay(&mut self, index: usize, state: bool) {
        let schuetz = self.temps.get_mut(index).unwrap();
        if schuetz.enabled != state {
            schuetz.borrow_mut().trigger();
        }

    }
    pub fn switch_pump_relay(&mut self, index: usize, state: bool) {
        let schuetz = self.pumps.get_mut(index).unwrap();
        if schuetz.enabled != state {
            schuetz.trigger();
        }
    }
    pub fn off(&mut self){
        for schuetz in &mut self.temps {
           if schuetz.enabled {
               schuetz.trigger()
           }
        }
        for schuetz in &mut self.pumps {
           if schuetz.enabled {
               schuetz.trigger()
           }
        }
    }
}
