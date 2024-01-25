use redis::Commands;
use rppal::gpio::Gpio;
use std::{thread, time};

fn main() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con0 = client.get_connection().unwrap();
    let mut pubsub = con0.as_pubsub();
    let mut con1 = client.get_connection().unwrap();
    pubsub.psubscribe("relay.*").expect("could not subscribe to relay.*");

    let _ : () = con1.set("relay.0", false).unwrap();
    let _ : () = con1.set("relay.1", false).unwrap();
    let _ : () = con1.set("relay.2", false).unwrap();

    loop {
        let msg = pubsub.get_message().unwrap();
        let goal : bool = msg.get_payload().unwrap();
        let channel = msg.get_channel_name();
        let current : bool = con1.get(channel).unwrap();
        if goal != current {
           switch_temperature_relay(&channel);
           let _ : () = con1.set(channel, goal).unwrap();        
        }
    }

}
pub fn switch_temperature_relay(relay: &str) {
    let pin : u8 = match relay {
        "relay.0" => 0,
        "relay.1" => 1,
        "relay.2" => 2,
        _ => 0,
    };
    let mut trigger = Gpio::new().unwrap().get(pin).unwrap().into_output();
    trigger.set_high();
    thread::sleep(time::Duration::from_millis(50));
    trigger.set_low();        
}
