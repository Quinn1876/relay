use relay::can_extentions::prelude::*;
use socketcan::CANSocket;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use termion::input::TermRead;
use termion::event::Key;
use termion::raw::IntoRawMode;

fn main() {
    let throttle_percent = Arc::new(Mutex::new(0));
    let node_id = 1;
    let max_motors = 1;
    let mut stdout = std::io::stdout().into_raw_mode();
    // let throttle_percent = 100;
    let socket = CANSocket::open("can0").unwrap();

    let throttle_percent_2 = throttle_percent.clone();

    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        for c in stdin.keys() {
            let mut throttle_percent = throttle_percent_2.lock().unwrap();
            match c.unwrap() {
                Key::Up => { *throttle_percent += 1 },
                Key::Down => { *throttle_percent -= 1 }
                Key::Ctrl('c') => { panic!(); }
                _ => {}
            }
        }
    });


    loop {
        {
            let throttle_percent = throttle_percent.lock().unwrap();
            socket.set_motor_throttle(node_id, max_motors, *throttle_percent).expect("To Send the message");
            println!("Throttle percent {:?}\n\r", throttle_percent);
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    println!("Hello, world!");
}
