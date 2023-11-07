use super::structs::{RecipeStatus, RecipeState}; 
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


}
