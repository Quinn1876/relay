use crate::pod_states::{ PodState };

/**
 * @brief Provides an interface for tracking the state of our embeded 
 * boards. 
 */
pub struct BoardStates {
    bms_state: PodState
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
            bms_state: PodState::LowVoltage
        }
    }
    /**
     * @brief get the value of the bms state
     */
    pub fn get_bms_state(&self) -> &PodState {
        return &self.bms_state;
    }

    /**
     * @brief Set the value of the bms state
     */
    pub fn set_bms_state(&mut self, new_state: &PodState) {
        self.bms_state = *new_state;
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
}