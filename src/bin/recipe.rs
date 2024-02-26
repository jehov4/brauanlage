use redis::Commands;
use redis_test::RecipeStep;
use serde::Deserialize;

fn main() {
    let mut recipe : Vec<RecipeStep> = Vec::new(); 
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con0 = client.get_connection().unwrap();
    let mut pubsub = con0.as_pubsub();
    let mut con1 = client.get_connection().unwrap();
    pubsub.psubscribe("recipe.*").expect("could not subscribe to recipe");

    // Set Recipe Status
    // 0 - Not Loaded
    // 1 - Loaded
    // 2 - Started
    // 3 - Paused
    // 4 - Finished
    let _ : () = con1.set("recipe.status", 0).unwrap();
    let _ : () = con1.set("recipe.start", 0).unwrap();
    let _ : () = con1.set("recipe.step_index", 0).unwrap();    
    let _ : () = con1.set("recipe.duration", 0).unwrap();
    
    let _ : () = con1.set("temp.0", 0.0).unwrap();
    let _ : () = con1.set("temp.1", 0.0).unwrap();
     
    loop {
        let msg = pubsub.get_message().unwrap();
        // let message : i32 = msg.get_payload().unwrap();
        let channel : &str = msg.get_channel_name();
        // let goal : i32 = con1.get(channel).unwrap();
        // println!("goal {}: measured {}", goal, measured);
        match channel.split(".").last().unwrap() {
           "start" => unimplemented!(),
           "stop" => unimplemented!(),
           "pause" => unimplemented!(),
           "load" => {
                let recipe_string : String = msg.get_payload().unwrap();
                load_recipe(&mut recipe, recipe_string.as_str())
           },
           "set_goal" => unimplemented!(),
           "add_duration" => unimplemented!(),
           "next" => unimplemented!(),
           _ => unimplemented!(),
        }        
    }   
}

fn load_recipe(dest: &mut Vec<RecipeStep>, src: &str){
    let steps: Vec<RecipeStep> = serde_yaml::from_str(src).ok().unwrap();
    dest.clone_from(&&steps)
}
