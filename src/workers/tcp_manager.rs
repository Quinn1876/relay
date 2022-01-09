use super::tcp_worker::TcpWorkerState;
use std::sync::mpsc::{
    Sender,
    Receiver
};
use super::messages::*;
pub struct TcpManager {
}

impl TcpManager {
    pub fn run<A: std::net::ToSocketAddrs + Send + 'static>(
        address: A,
        udp_message_sender: Sender<UDPMessage>,
        tcp_message_receiver: Receiver<TcpMessage>,
        tcp_message_buffer_size: usize
    ) -> std::thread::JoinHandle<()> {
        std::thread::Builder::new().name("TCP Thread".to_string()).spawn(move || {
            // Setup
            let mut tcp_worker = TcpWorkerState::new(address, udp_message_sender, tcp_message_receiver, tcp_message_buffer_size);
            loop {
                tcp_worker = tcp_worker.main_loop();
            }
        }).expect("Should be able to create Thread")
    }
}
