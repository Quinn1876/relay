use relay::can_extentions::prelude::*;
use socketcan::CANSocket;

fn main() {
    let node_id = 0;
    let max_motors = 1;
    let throttle_percent = 100;
    let socket = CANSocket::open("can0").unwrap();
    loop {
        socket.set_motor_throttle(node_id, max_motors, throttle_percent).expect("To Send the message");
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    println!("Hello, world!");
}
