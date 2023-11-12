use super::helper::Helper;

#[derive(Clone)]
pub struct RecipeStep {
    // holds the temperature goals
    // TODO: mapping to relays has to be done via configuration
    pub temperatures: Vec<f32>,
    // Duration in Seconds
    pub duration: u64,
    // holds the states target states of the pumps
    // true -> on
    // false -> off
    // TODO: mapping to relays has to be done via configuration
    pub schuetze: Vec<bool>,
    pub automatic: bool,
}

#[derive(Clone)]
pub struct RecipeStatus {
    pub recipe_steps: Vec<RecipeStep>,
    pub step_index: usize,
    pub state: RecipeState,
    pub step_timestamp: u64,
}

pub struct FullStatus {
    pub recipe_status: RecipeStatus,
    pub temperatures: Vec<f32>,
    pub schuetze: Vec<bool>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum RecipeState {
    EMPTY,
    LOADED,
    RUNNING,
    WAITING,
    FINISHED,
}


#[derive(PartialEq)]
pub enum HeaterState {
    ENABLED,
    DISABLED,
    IDLE,
}

pub enum Command {
    START,
    STOP,
    SKIP,
    RECIPE(Vec<RecipeStep>),
    STEP(RecipeStep),
    OVTEMPS(Vec<f32>),
    OVPUMPS(Vec<bool>),
    OVTEMP(TempOverride),
    OVPUMP(PumpOverride),
    OVDURATION(u64),
}


pub struct TempOverride {
    pub index: usize,
    pub temp: f32,
}

pub struct PumpOverride {
    pub index: usize,
    pub state: bool,
}

impl RecipeStatus {
    pub fn current_step(&mut self) -> &mut RecipeStep {
        self.recipe_steps.get_mut(self.step_index).unwrap()
    }
    pub fn new() -> Self {
        let empty_step = RecipeStep::new_empty();        
        RecipeStatus { recipe_steps: vec![empty_step], step_index: 0, state: RecipeState::EMPTY, step_timestamp: Helper::get_sys_time_in_secs() }
    }
}

impl RecipeStep {
    pub fn new_empty() -> Self {
        RecipeStep { temperatures: vec![0.0,0.0], schuetze: vec![false, false, false], automatic: false, duration: u64::max_value() }
    }
}
