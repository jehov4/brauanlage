use std::{fs, collections::HashMap};
use redis::Commands;

fn main() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con0 = client.get_connection().unwrap();

    let basepath = String::from("/sys/bus/w1/devices/");
    let paths = HashMap::from([
                              ("temp.0", basepath.clone() + "foo"),
                              ("temp.1", basepath.clone() + "bar"),]);

    loop {
        for (reference, path) in &paths {
            let set : f32 =  con0.get(reference).unwrap();
            if set > 0.0 {
                let val = fs::read_to_string(path).unwrap().parse::<f32>().unwrap()/1000.0;
                let _ : () = con0.publish(reference, val.to_string()).expect("publish failed");
            }
        }
    }

}
