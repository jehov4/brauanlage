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
    pub pumps: Vec<bool>,
    pub automatic: bool,
}

pub struct RecipeStatus {
    pub recipe_steps: Vec<RecipeStep>,
    pub step_index: usize,
    pub state: RecipeState,
    pub step_timestamp: u64,
}

#[derive(PartialEq, Eq)]
pub enum RecipeState {
    EMPTY,
    LOADED,
    RUNNING,
    PAUSED,
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
}
