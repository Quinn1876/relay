use std::collections::HashMap;

#[derive(PartialEq, Hash, Eq, Debug, Copy, Clone)]
pub enum PodState {
    LowVoltage,
    Armed,
    AutoPilot,
    Braking,
    EmergencyBrake,
    SystemFailure,
    Resting,
    ManualOperationWaiting,
    Accelerating,
    AtSpeed,
    Decelerating,
    Invalid
    // More to come for manual operation
}

mod test {
    #[allow(unused_imports)]
    use super::{ PodState, HashMap };
    #[test]
    fn test_states() {
        for i in 0u8..0x0Bu8 {
            assert_eq!(PodState::from_byte(i).to_byte(), i);
        }
    }

    #[test]
    fn test_transitions() {
        let all_states = vec![
            PodState::Resting,
            PodState::LowVoltage,
            PodState::Armed,
            PodState::AutoPilot,
            PodState::Braking,
            PodState::EmergencyBrake,
            PodState::SystemFailure,
            PodState::ManualOperationWaiting,
            PodState::Accelerating,
            PodState::AtSpeed,
            PodState::Decelerating,
            PodState::Invalid
        ];

        let mut valid_transitions: HashMap<PodState, Vec<PodState>> = HashMap::new();

        valid_transitions.insert(PodState::Resting, vec![PodState::LowVoltage]);
        valid_transitions.insert(PodState::LowVoltage, vec![PodState::Resting, PodState::Armed]);
        valid_transitions.insert(PodState::Armed, vec![PodState::LowVoltage, PodState::AutoPilot, PodState::EmergencyBrake]);
        valid_transitions.insert(PodState::AutoPilot, vec![PodState::Braking, PodState::EmergencyBrake]);
        valid_transitions.insert(PodState::Braking, vec![PodState::LowVoltage]);
        valid_transitions.insert(PodState::EmergencyBrake, vec![PodState::SystemFailure]);
        valid_transitions.insert(PodState::SystemFailure, vec![]);

        for state in all_states {
            let transitions = valid_transitions.get(&state);
            for new_states in transitions {
                for new_state in new_states {
                    // println!("State: {:?}, NewState: {:?}", state, new_state);
                    assert_eq!(state.can_transition_to(new_state), true);
                }
            }
        }
    }
}

/**
 * This Section should be kept in line with the definition in the CAN Communication Protocol Document
 * source: https://docs.google.com/spreadsheets/d/18rGH__yyJPf3jil74yTlVyFFqCOyuNzP3DCFmmIWWbo/edit?usp=drive_web&ouid=109880063725320746438
 */
impl PodState {
    pub fn to_byte(&self) -> u8 {
        match self {
            PodState::Resting                  => 0x00,
            PodState::LowVoltage               => 0x01,
            PodState::Armed                    => 0x02,
            PodState::AutoPilot                => 0x03,
            PodState::Braking                  => 0x04,
            PodState::EmergencyBrake           => 0x05,
            PodState::SystemFailure            => 0x06,
            PodState::ManualOperationWaiting   => 0x07,
            PodState::Accelerating             => 0x08,
            PodState::AtSpeed                  => 0x09,
            PodState::Decelerating             => 0x0A,
            PodState::Invalid                  => 0x0B
        }
    }

    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => PodState::Resting,
            0x01 => PodState::LowVoltage,
            0x02 => PodState::Armed,
            0x03 => PodState::AutoPilot,
            0x04 => PodState::Braking,
            0x05 => PodState::EmergencyBrake,
            0x06 => PodState::SystemFailure,
            0x07 => PodState::ManualOperationWaiting,
            0x08 => PodState::Accelerating,
            0x09 => PodState::AtSpeed,
            0x0A => PodState::Decelerating,
            _ => PodState::Invalid
        }
    }

    /**
     * @brief validates state transitions
     */
    pub fn can_transition_to(&self, new_state: &PodState) -> bool {
        (match self {
            PodState::Resting => matches!(new_state, PodState::LowVoltage),
            PodState::LowVoltage =>  matches!(new_state, PodState::Resting | PodState::Armed),
            PodState::Armed => matches!(new_state, PodState::LowVoltage | PodState::AutoPilot | PodState::EmergencyBrake),
            PodState::AutoPilot => matches!(new_state, PodState::Braking | PodState::EmergencyBrake),
            PodState::Braking => matches!(new_state, PodState::LowVoltage),
            PodState::EmergencyBrake => matches!(new_state, PodState::SystemFailure),
            PodState::SystemFailure => false,
            PodState::ManualOperationWaiting => false, // TODO Implement manual controls
            PodState::Accelerating => false,
            PodState::AtSpeed => false,
            PodState::Decelerating => false,
            PodState::Invalid => false
        }) || *new_state == PodState::SystemFailure
    }

    pub fn is_error_state(&self) -> bool {
        matches!(self, PodState::EmergencyBrake | PodState::SystemFailure)
    }
}

impl From<u8> for PodState {
    fn from(byte: u8) -> PodState {
        return PodState::from_byte(byte);
    }
}

impl Into<u8> for &PodState {
    fn into(self) -> u8 {
        return self.to_byte();
    }
}

