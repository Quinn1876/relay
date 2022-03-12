use super::pod_state_message::PodStateMessage;
use std::net::UdpSocket;


pub trait ProjectButterfreeUDPSocket {
    fn send_pod_state_message(&self, msg: &PodStateMessage) -> std::io::Result<usize>;
}

impl ProjectButterfreeUDPSocket for UdpSocket {
    fn send_pod_state_message(&self, msg: &PodStateMessage) -> std::io::Result<usize> {
        self.send(&msg.to_json_bytes())
    }
}
