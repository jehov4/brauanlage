use std::borrow::BorrowMut;
use std::{thread, time};
use rppal::gpio::Gpio;

#[derive(Clone)]
pub struct Schuetz {
    pub enabled: bool,
    trigger_pin: u8,
}

impl Schuetz {
    pub fn new(pin: u8) -> Self {
        Schuetz { enabled: false, trigger_pin: pin }
    }
}


impl Schuetz {
    #[cfg(not(test))]
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
    #[cfg(test)]
    pub fn trigger (&mut self) {
        println!("{} triggered", self.trigger_pin)
    }
}

pub struct Peripheral {
    pub temps: Vec<Schuetz>,
    pub pumps: Vec<Schuetz>,
}

impl Peripheral {
    pub fn new() -> Self {
        Peripheral { temps: vec![Schuetz::new(1), Schuetz::new(2)], pumps: vec![Schuetz::new(3), Schuetz::new(4), Schuetz::new(5)] }
    }
    
    #[cfg(not(test))]
    pub fn get_temperatures(&self) -> Vec<f32> {
        unimplemented!();
    }
    #[cfg(test)]
    pub fn get_temperatures(&self) -> Vec<f32> {
        vec![0.0,0.0] 
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

    pub fn match_pump_relays(&mut self, states: &Vec<bool>) {
        let mut index = 0;
        for state in states {
            self.switch_pump_relay(index, *state);
            index = index + 1;
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

    pub fn get_pump_states(&self) -> Vec<bool>{
        let mut schuetze: Vec<bool> = Vec::new();
        for schuetz in &self.pumps {
            schuetze.push(schuetz.enabled);
        }
        schuetze
    }
}
