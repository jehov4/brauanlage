pub struct RecipeStep {
    // holds the temperature goals
    // TODO: mapping to relays has to be done via configuration
    pub temperatures: Vec<f32>,
    // Duration in Seconds
    pub duration: usize,
    // holds the states target states of the pumps
    // true -> on
    // false -> off
    // TODO: mapping to relays has to be done via configuration
    pub pumps: Vec<bool>,
}

pub struct RecipeStatus {
    pub recipe_steps: Vec<RecipeStep>,
    pub step_index: usize,
    pub state: RecipeState,
    pub step_timestamp: usize,
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


