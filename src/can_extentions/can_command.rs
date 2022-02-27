/**
 * @file can.rs
 * This module implements a socket can interface for connecting to the Staravia
 * board and the rest of the pod
 *
 */
use crate::pod_states::PodState;

// This is an incomplete list of CAN Commands.
// The full list that will eventually need to be supported
// can be found here: (Can Communication Protocol) [https://docs.google.com/document/d/1pAAAPyWClxrq7MwrA0_AGxnqU6B5r5MHmvRERMY6hUo/edit]
pub enum CanCommand {
    BmsHealthCheck { battery_pack_current: u32, cell_temperature: u32 },
    BmsStateChange(PodState),
    PressureHigh(f32),
    PressureLow1(f32),
    PressureLow2(f32),
    Torchic1([Option<f32>; 2]),
    Torchic2([Option<f32>; 2]),
    Unknown(u32), // Arbitration ID provided for debugging Purposes
}
