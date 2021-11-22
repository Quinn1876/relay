/**
 * roboteq CAN Interface
 * Transpiled for Rust by Quinn Hodges
 * Original Python code by Guy Blumenthal
 *
 * Source DataSheet Outlineing CAN specification: https://drive.google.com/file/d/1ALK8BErG0tjE8fcfFuHN_62qg2OUG2fF/view?usp=sharing
 */

use socketcan;
use crate::tcp_server::CanError;

pub trait Roboteq {
    fn send_msg(&self, node_id: u32, is_query: bool, empty_bytes: u32, index: u16, subindex: u8, data: &[u8]) -> Result<(), CanError>;
    fn set_motor_throttle(&self, node_id: u32, max_motors: u8, throttle_percent: u32) -> Result<(), CanError>;
}

impl Roboteq for socketcan::CANSocket {
    fn send_msg(&self, node_id: u32, is_query: bool, empty_bytes: u32, index: u16, subindex: u8, data: &[u8]) -> Result<(), CanError> {
        let byte_0: u8 = 0b00000000 | ((if is_query { 4u8 } else { 2u8 }) << 4) | ((empty_bytes as u8) << 2);
        let data: [u8; 8] = [
            byte_0,
            ((index) & 0xFF) as u8,
            ((index >> 8) & 0xFF) as u8,
            subindex,
            data[0],
            data[1],
            data[2],
            data[3]
        ];
        let message = socketcan::CANFrame::new(0x600 + node_id, &data, false, false).map_err(|e| CanError::MessageError(e))?;
        self.write_frame(&message).map_err(|e| CanError::WriteError(e))?;
        Ok(())
    }

    fn set_motor_throttle(&self, node_id: u32, max_motors: u8, throttle_percent: u32) -> Result<(), CanError> {
        self.send_msg(node_id, false, 0, 0x2000, max_motors, &to_bytes(throttle_percent))
    }
}

fn to_bytes(number: u32) -> [u8; 4] {
    [
        (number & 0xFF) as u8,
        ((number >> 8) & 0xFF) as u8,
        ((number >> 8 * 2) & 0xFF) as u8,
        ((number >> 8 * 3) & 0xFF) as u8,
    ]
}
