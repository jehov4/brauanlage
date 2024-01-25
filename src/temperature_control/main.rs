use redis::Commands;

fn main() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con0 = client.get_connection().unwrap();
    let mut pubsub = con0.as_pubsub();
    let mut con1 = client.get_connection().unwrap();
    pubsub.psubscribe("temp.*");

    let _ : () = con1.set("temp.0", 0).unwrap();
    let _ : () = con1.set("temp.1", 0).unwrap();

    loop {
        let msg = pubsub.get_message().unwrap();
        let measured : i32 = msg.get_payload().unwrap();
        let channel : String = msg.get_channel_name().to_string();
        let goal : i32 = con1.get(channel).unwrap();
        println!("goal {}: measured {}", goal, measured);
    }
}

