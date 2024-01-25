use std::{time::SystemTime, collections::HashMap};
use serde::{Deserialize, Serialize};

pub fn get_sys_time_in_secs() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RecipeStep {
    // holds the temperature goals
    // TODO: mapping to relays has to be done via configuration
    pub temperatures: HashMap<String, f32>,
    // Duration in Seconds
    pub duration: u64,
    // holds the states target states of the pumps
    // true -> on
    // false -> off
    // TODO: mapping to relays has to be done via configuration
    pub relays: HashMap<String, bool>,
    pub automatic: bool,
}
