use std::sync::mpsc;

use super::structs::{RecipeStatus, RecipeState, RecipeStep, Command, HeaterState, PumpOverride, TempOverride}; 
use super::peripheral::Peripheral;
use super::helper::Helper;

pub struct Control {
    temperatures: Vec<f32>,
    recipe_status: RecipeStatus,
    peripheral: Peripheral,
    command_channel: mpsc::Receiver<Command>,
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
        let rcv_command = self.command_channel.try_recv();
        if rcv_command.is_ok() {
            let command = rcv_command.ok().unwrap();
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
            let goal = self.recipe_status.current_step().temperatures.get(index).unwrap();
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
            if Helper::get_sys_time_in_secs() > status.step_timestamp + current_step.duration {
                self.next_step()
            }
        }
    }

    fn next_step(&mut self) {
        let status = &mut self.recipe_status;
        let next_step_index = status.step_index + 1;
        if status.recipe_steps.len() < next_step_index {
            status.step_index = next_step_index;
            let next_step = status.current_step();
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
        self.recipe_status.step_timestamp = Helper::get_sys_time_in_secs();
    }

    fn manual_goal_override(&mut self, new_goals: Vec<f32>){
        let status = &mut self.recipe_status;
        status.current_step().temperatures.clone_from(&new_goals);
    }

    fn manual_pump_override(&mut self, new_states: Vec<bool>){
        let status = &mut self.recipe_status;
        status.current_step().pumps.clone_from(&new_states);
    }

    fn manual_goal_override_single(&mut self, oride: TempOverride) {    
        let status = &mut self.recipe_status;
        let temps = &status.current_step().temperatures;
        temps.get(oride.index).unwrap().clone_from(&&oride.temp);
    }

    fn manual_pump_override_single(&mut self, oride: PumpOverride) {    
        let status = &mut self.recipe_status;
        let pumps = &status.current_step().pumps;
        pumps.get(oride.index).unwrap().clone_from(&&oride.state);
    }

    fn manual_override_duration(&mut self, duration: u64) {
        let status = &mut self.recipe_status;
        status.current_step().duration = duration;
    }

    fn stop(&mut self) {
        self.peripheral.off();
    }

    fn import_recipe(&mut self, steps: Vec<RecipeStep>){
        let mut steps =  steps.clone();
        self.recipe_status.recipe_steps.append(steps.as_mut());
    }

    fn process_command(&mut self, command: Command) {
        match command {
            Command::SKIP => self.next_step(),
            Command::START => self.start_step(),
            Command::STOP => self.stop(),
            Command::OVTEMP(value) => self.manual_goal_override_single(value),
            Command::OVPUMP(value) => self.manual_pump_override_single(value),
            Command::OVTEMPS(value) => self.manual_goal_override(value),
            Command::OVPUMPS(value) => self.manual_pump_override(value),
            Command::OVDURATION(value) => self.manual_override_duration(value),
            Command::RECIPE(value) => self.import_recipe(value),
            Command::STEP(value) => self.import_recipe(vec![value]),
        }
    }
}
