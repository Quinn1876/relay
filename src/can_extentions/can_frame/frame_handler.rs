use super::super::can_command::CanCommand;
use crate::pod_states::PodState;
use byteorder::{ LittleEndian, ByteOrder };

/**
 *
 * CAN FRAME HANDLER
 *
 * Functions for decoding a Can Frame
 */
pub trait FrameHandler {
    fn get_command(&self) -> CanCommand;
}

impl FrameHandler for socketcan::CANFrame {
    fn get_command(&self) -> CanCommand {
        let id = self.id();
        let data = self.data();

        match id {
            0x00B => CanCommand::BmsStateChange(get_state_change_data(data)),
            0x040 => CanCommand::Torchic1(get_torchic_data(data)),
            id => CanCommand::Unknown(id)
        }
    }
}

/**
 * @func:  get_state_change_data
 * @brief: A State change CAN frame will be a single byte which represents
 * the new state. Only the first byte will be checked regardless of if
 * there are more bytes in the data field. This is to allow expansion should
 * we in the future add more fields to these messages. The alternative would be
 * to throw an error if the data slice is larger then one, but this would create
 * fragile software without the proper error handling.
 */
fn get_state_change_data(data: &[u8]) -> PodState {
    if data.len() == 0 { PodState::Invalid }
    else { PodState::from_byte(data[0]) }
}

/**
 * @func get_torchic_data
 * @brief Torchic frames consist of 2 4-byte floats containing temperature data
 */
fn get_torchic_data(data: &[u8]) -> [Option<f32>; 2] {
    [Some(LittleEndian::read_f32(&[data[0], data[1], data[2], data[3]])), Some(LittleEndian::read_f32(&[data[4], data[5], data[6], data[7]]))]
}
