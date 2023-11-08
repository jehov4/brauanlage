use std::time::SystemTime;

use super::structs::{RecipeStatus, RecipeState, RecipeStep}; 
use super::peripheral::{Peripheral};

pub struct Control {
    temperatures: Vec<f32>,
    recipe_status: RecipeStatus,
    peripheral: Peripheral,
}

#[derive(PartialEq)]
enum HeaterState {
    ENABLED,
    DISABLED,
    IDLE,
}

impl Control {
    pub fn control_loop (&mut self) {
        self.update_temperatures();
        let temp_checking_states = vec![RecipeState::RUNNING, RecipeState::PAUSED, RecipeState::WAITING];
        // only do temp and recipe checking, when recipe is running
        if temp_checking_states.contains(&self.recipe_status.state) {
            self.check_temperatures(); 
        } else if self.recipe_status.state == RecipeState::FINISHED {
            self.peripheral.off();
        }
    }

    // Update the Temperture Vector to the current temperatures
    fn update_temperatures(&mut self) {
        let temperatures = self.peripheral.get_temperatures();
        self.temperatures.clone_from(&temperatures)
    }
    // check if the temperatures are too high or low
    fn check_temperatures(&mut self){
        let index: usize = 0;
        for temperature in &self.temperatures {
            let goal = self.recipe_status.recipe_steps.get(self.recipe_status.step_index).unwrap().temperatures.get(index).unwrap();
            // Check what the new state will be
            // ENABLED -> Heat
            // DISABLED -> Cool
            // IDLE -> Within Tolerances keep doing what you did
            let new_state = Control::get_new_heater_state(*temperature, *goal);
            // Check whether change is required
            if new_state != HeaterState::IDLE {
                // Enable/Disable
                self.peripheral.switch_temperature_relay(index, new_state == HeaterState::ENABLED);
            }
        }
    }

    // Calculate new state to stay within tolerances
    fn get_new_heater_state (temperature: f32, goal: f32) -> HeaterState {
        let tolerance = 0.5;
        if temperature > goal + tolerance {
            HeaterState::DISABLED
        } else if temperature < goal - tolerance {
            HeaterState::ENABLED
        } else {
            HeaterState::IDLE
        }
    }

    fn check_recipe(&mut self) {
        // Check whether the current step is finished
        // In case queue the next step
        // If it is on hold hold the current temperature
        // If it is on autostart set the new temperature
        let status = &self.recipe_status;
        let current_step = status.recipe_steps.get(status.step_index).unwrap();
        if status.state == RecipeState::RUNNING{
            if Self::get_sys_time_in_secs() > status.step_timestamp + current_step.duration {
                self.next_step()
            }
        }
    }

    fn next_step(&mut self) {
        let status = &mut self.recipe_status;
        let next_step_index = status.step_index + 1;
        if status.recipe_steps.len() < next_step_index {
            let next_step = status.recipe_steps.get(next_step_index).unwrap();
            status.step_index = next_step_index;
            if next_step.automatic {
                self.start_step();
            } else {
                status.state = RecipeState::WAITING;
            }

        } else {
            status.state = RecipeState::FINISHED;
        }
    }

    fn start_step(&mut self) {
        self.recipe_status.state = RecipeState::RUNNING;
        self.recipe_status.step_timestamp = Self::get_sys_time_in_secs();
    }

    fn manual_goal_override(&mut self, new_goals: Vec<f32>){
        let status = &mut self.recipe_status;
        status.recipe_steps.get_mut(status.step_index).unwrap().temperatures.clone_from(&new_goals);
    }

    // Helper to get current system time in Seconds
    fn get_sys_time_in_secs() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }

}


}
