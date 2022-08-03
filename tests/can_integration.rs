use relay::can_extentions;
use relay::can_extentions::ack_nack::AckNack;
use relay::can_extentions::prelude::*;
use relay::pod_states::PodState;
use virtual_can_device::config::{
  Producer,
  Responder,
  common::OutMessage
};
use virtual_can_device::device_manager::DeviceManager;

/**
 * Once we implement G6 can protocol, we'll move this into it's own file.
 */
mod can_ids {
  pub const BMS_STATE_ACK: u32 = 0x0B;
}

#[test]
fn test_can_bus() {
  let can_socket = can_extentions::open_socket("vcan0").unwrap();
  let producer = Producer{
    messages: vec![
      OutMessage {
        id: can_ids::BMS_STATE_ACK,
        data: vec![PodState::LowVoltage.to_byte(), 0],
      }
    ],
    name: String::from("Producer 1"),
    period: 10, // 10 seconds
  };
  let mut device_manager = DeviceManager::new("vcan0");
  device_manager.add_producer(&producer);
  let device_manager = device_manager.start_devices();
  let frame = can_socket.read_frame().unwrap();
  match frame.get_command() {
    CanCommand::BmsStateChange(ack_nack) => {
      assert_eq!(ack_nack, AckNack::Ack);
    },
    _ => panic!("Only expects Bms State Change")
  }
  device_manager.stop_devices();
}
