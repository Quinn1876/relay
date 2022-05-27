use crate::pod_states::{ PodState };

/**
 * @brief Provides an interface for tracking the state of our embeded
 * boards.
 */
pub struct BoardStates {
    bms_state: PodState,
    motor_controller_state: PodState,
    pressure_state: PodState
}

pub enum Error {
    InvalidTransision
}

impl BoardStates {
    /**
     * @breif contruct a BoardState object with default values
     */
    pub fn default() -> BoardStates {
        BoardStates {
            bms_state: PodState::LowVoltage,
            motor_controller_state: PodState::LowVoltage,
            pressure_state: PodState::LowVoltage
        }
    }
    /**
     * @brief get the value of the bms state
     */
    pub fn get_bms_state(&self) -> &PodState {
        return &self.bms_state;
    }

    /**
     * @brief get the value of the motor controller state
     */
    pub fn get_motor_controller_state(&self) -> &PodState {
        return &self.motor_controller_state;
    }

    pub fn get_pressure_state(&self) -> &PodState {
        return &self.pressure_state;
    }


    /**
     * @brief Set the value of the bms state
     */
    pub fn set_bms_state(&mut self, new_state: &PodState) {
        self.bms_state = *new_state;
    }

    /**
     * @brief Set the value of the motor controller state
     */
    pub fn set_motor_controller_state(&mut self, new_state: &PodState) {
        self.motor_controller_state = *new_state;
    }

    pub fn set_pressure_state(&mut self, new_state: &PodState) {
        self.pressure_state = *new_state;
    }

    /**
     * @brief wrapper for setting the bms state which checks if the transistion
     * is valid and throws and error if it is not
     * @param new_state the value to set bms state to.
     */
    pub fn set_bms_state_transition_checked(&mut self, new_state: &PodState)
    -> Result<(), Error>
    {
        if self.get_bms_state().can_transition_to(&new_state) {
            self.set_bms_state(new_state);
            Ok(())
        } else {
            Err(Error::InvalidTransision)
        }
    }

    /**
     * @brief wrapper for setting the motor controller state which checks if the transistion
     * is valid and throws and error if it is not
     * @param new_state the value to set motor controller state to.
     */
    pub fn set_motor_controller_state_transition_checked(&mut self, new_state: &PodState)
    -> Result<(), Error>
    {
        if self.get_motor_controller_state().can_transition_to(&new_state) {
            self.set_motor_controller_state(new_state);
            Ok(())
        } else {
            Err(Error::InvalidTransision)
        }
    }
}
