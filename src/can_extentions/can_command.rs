/**
 * @file can.rs
 * This module implements a socket can interface for connecting to the Staravia
 * board and the rest of the pod
 *
 */
use crate::can_extentions::fault_reports::{ BmsFaultReport, MotorControllerFaultReport };
use crate::can_extentions::ack_nack::AckNack;

// The full list that need to be supported
// can be found here: (Can Communication Protocol) [https://docs.google.com/document/d/1pAAAPyWClxrq7MwrA0_AGxnqU6B5r5MHmvRERMY6hUo/edit]
pub enum CanCommand {
    BmsHealthCheck { battery_pack_current: f32, cell_temperature: f32 },
    MotorControllerHealthCheck { igbt_temp: f32, motor_voltage: f32 },
    BmsFaultReport(BmsFaultReport),
    BmsStateChange(AckNack),
    BmsData1 { battery_pack_voltage: f32, state_of_charge: f32 },
    BmsData2 { buck_temperature: f32, bms_current: f32 },
    BmsData3 { link_cap_voltage: f32 },
    MotorControllerFaultReport(MotorControllerFaultReport),
    MotorControllerStateChange(AckNack),
    MotorControllerData1 { mc_pod_speed: f32, motor_current: f32 },
    MotorControllerData2 { battery_current: f32, battery_voltage: f32 },
    PodSpeed { pod_speed: f32 },
    PressureHigh(f32),
    PressureLow1(f32),
    PressureLow2(f32),
    Torchic1([Option<f32>; 2]),
    Torchic2([Option<f32>; 2]),
    Current5V(f32),
    Current12V(f32),
    Current24V(f32),
    Unknown(u32), // Arbitration ID provided for debugging Purposes
}
