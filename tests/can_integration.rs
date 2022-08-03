use relay::can_extentions;
use relay::can_extentions::prelude::*;

#[test]
fn test_can_bus() {
  let can_socket = can_extentions::open_socket("vcan0").unwrap();
}
