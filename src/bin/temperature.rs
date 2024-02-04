use redis::Commands;

fn main() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let tolerance = 0.5;
    let mut con0 = client.get_connection().unwrap();
    let mut pubsub = con0.as_pubsub();
    let mut con1 = client.get_connection().unwrap();
    pubsub.psubscribe("temp.*").expect("could not subscribe to temp.*");


    loop {
        let msg = pubsub.get_message().unwrap();
        let measured : f32 = msg.get_payload().unwrap();
        let channel : &str = msg.get_channel_name();
        let goal : f32 = con1.get(channel).unwrap();
        let relay_channel : String = String::from("relay.") + channel.split('.').last().unwrap();
        if measured > goal + tolerance {
           let _ : () = con1.publish(relay_channel, false).expect("deactivation publish failed"); 
        } else if measured < goal - tolerance {
           let _ : () = con1.publish(relay_channel, true).expect("activation publish failed");
        }
    }
}

