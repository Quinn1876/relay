/**
 * Roboteq CAN Interface
 * Transpiled for Rust by Quinn Hodges
 * Original Python code by Guy Blumenthal
 *
 * @brief implements methods specific for interfacing with the roboteq motor controller.
 *
 * Source DataSheet Outlineing CAN specification: https://drive.google.com/file/d/1ALK8BErG0tjE8fcfFuHN_62qg2OUG2fF/view?usp=sharing
 */

use socketcan;
use super::super::error::CanError as Error;

pub trait RoboteqCanSocket {
    fn send_msg(&self, node_id: u32, is_query: bool, empty_bytes: u32, index: u16, subindex: u8, data: &[u8]) -> Result<(), Error>;
    fn set_motor_throttle(&self, node_id: u32, motor_number: u8, throttle_percent: u32) -> Result<(), Error>;
    fn roboteq_read_encoder_motor_speed(&self, node_id: u32, motor_number: u8) -> Result<(), Error>;
    fn roboteq_read_battery_amps(&self, node_id: u32, motor_number: u8) -> Result<(), Error>;
    fn roboteq_read_temps(&self, node_id: u32) -> Result<(), Error>;
    fn roboteq_emergency_stop(&self, node_id: u32) -> Result<(), Error>;
}

impl RoboteqCanSocket for socketcan::CANSocket {
    /**
     * Framework for sending messages to the roboteq controller
     */
    fn send_msg(&self, node_id: u32, is_query: bool, empty_bytes: u32, index: u16, subindex: u8, data: &[u8]) -> Result<(), Error> {
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
        let message = socketcan::CANFrame::new(0x600 + node_id, &data, false, false).map_err(|e| Error::MessageError(e))?;
        self.write_frame(&message).map_err(|e| Error::WriteError(e))?;
        Ok(())
    }

    /**
     * @brief Send a motor throttle message to the roboteq
     */
    fn set_motor_throttle(&self, node_id: u32, motor_number: u8, throttle_percent: u32) -> Result<(), Error> {
        self.send_msg(node_id, false, 0, 0x2000, motor_number, &to_bytes(throttle_percent))
    }

    fn roboteq_read_encoder_motor_speed(&self, node_id: u32, motor_number: u8) -> Result<(), Error> {
        self.send_msg(node_id, true, 4, 0x2103, motor_number, &[0;4])
    }

    fn roboteq_read_battery_amps(&self, node_id: u32, motor_number: u8) -> Result<(), Error> {
        self.send_msg(node_id, true, 4, 0x210C, motor_number, &[0;4])
    }

    fn roboteq_read_temps(&self, node_id: u32) -> Result<(), Error> {
        self.send_msg(node_id, true, 4, 0x210F, 1, &[0;4])?;
        self.send_msg(node_id, true, 4, 0x210F, 2, &[0;4])?;
        self.send_msg(node_id, true, 4, 0x210F, 3, &[0;4])?;
        Ok(())
    }
    fn roboteq_emergency_stop(&self, node_id: u32) -> Result<(), Error> {
        self.send_msg(node_id, false, 4,  0x200C, 0x00, &[0;4])
    }
}

// TODO -> See if byteorder crate can implment this smoother
fn to_bytes(number: u32) -> [u8; 4] {
    [
        (number & 0xFF) as u8,
        ((number >> 8) & 0xFF) as u8,
        ((number >> 8 * 2) & 0xFF) as u8,
        ((number >> 8 * 3) & 0xFF) as u8,
    ]
}
