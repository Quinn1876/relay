use std::collections::HashMap;

#[derive(PartialEq, Hash, Eq, Debug)]
pub enum PodStates {
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
    use super::PodStates;
    use super::HashMap;
    #[test]
    fn test_states() {
        for i in 0u8..0x0Bu8 {
            assert_eq!(PodStates::from_byte(i).to_byte(), i);
        }
    }

    #[test]
    fn test_transitions() {
        let all_states = vec![
            PodStates::Resting,
            PodStates::LowVoltage,
            PodStates::Armed,
            PodStates::AutoPilot,
            PodStates::Braking,
            PodStates::EmergencyBrake,
            PodStates::SystemFailure,
            PodStates::ManualOperationWaiting,
            PodStates::Accelerating,
            PodStates::AtSpeed,
            PodStates::Decelerating,
            PodStates::Invalid
        ];

        let mut valid_transitions: HashMap<PodStates, Vec<PodStates>> = HashMap::new();

        valid_transitions.insert(PodStates::Resting, vec![PodStates::LowVoltage]);
        valid_transitions.insert(PodStates::LowVoltage, vec![PodStates::Resting, PodStates::Armed]);
        valid_transitions.insert(PodStates::Armed, vec![PodStates::LowVoltage, PodStates::AutoPilot, PodStates::EmergencyBrake]);
        valid_transitions.insert(PodStates::AutoPilot, vec![PodStates::Braking, PodStates::EmergencyBrake]);
        valid_transitions.insert(PodStates::Braking, vec![PodStates::LowVoltage]);
        valid_transitions.insert(PodStates::EmergencyBrake, vec![PodStates::SystemFailure]);
        valid_transitions.insert(PodStates::SystemFailure, vec![]);

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
impl PodStates {
    pub fn to_byte(&self) -> u8 {
        match self {
            PodStates::Resting                  => 0x00,
            PodStates::LowVoltage               => 0x01,
            PodStates::Armed                    => 0x02,
            PodStates::AutoPilot                => 0x03,
            PodStates::Braking                  => 0x04,
            PodStates::EmergencyBrake           => 0x05,
            PodStates::SystemFailure            => 0x06,
            PodStates::ManualOperationWaiting   => 0x07,
            PodStates::Accelerating             => 0x08,
            PodStates::AtSpeed                  => 0x09,
            PodStates::Decelerating             => 0x0A,
            PodStates::Invalid                  => 0x0B
        }
    }

    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => PodStates::Resting,
            0x01 => PodStates::LowVoltage,
            0x02 => PodStates::Armed,
            0x03 => PodStates::AutoPilot,
            0x04 => PodStates::Braking,
            0x05 => PodStates::EmergencyBrake,
            0x06 => PodStates::SystemFailure,
            0x07 => PodStates::ManualOperationWaiting,
            0x08 => PodStates::Accelerating,
            0x09 => PodStates::AtSpeed,
            0x0A => PodStates::Decelerating,
            _ => PodStates::Invalid
        }
    }

    pub fn can_transition_to(&self, new_state: &PodStates) -> bool {
        match self {
            PodStates::Resting => matches!(new_state, PodStates::LowVoltage),
            PodStates::LowVoltage =>  matches!(new_state, PodStates::Resting | PodStates::Armed),
            PodStates::Armed => matches!(new_state, PodStates::LowVoltage | PodStates::AutoPilot | PodStates::EmergencyBrake),
            PodStates::AutoPilot => matches!(new_state, PodStates::Braking | PodStates::EmergencyBrake),
            PodStates::Braking => matches!(new_state, PodStates::LowVoltage),
            PodStates::EmergencyBrake => matches!(new_state, PodStates::SystemFailure),
            PodStates::SystemFailure => false,
            PodStates::ManualOperationWaiting => false, // TODO Implement manual controls
            PodStates::Accelerating => false,
            PodStates::AtSpeed => false,
            PodStates::Decelerating => false,
            PodStates::Invalid => false
        }
    }
}
