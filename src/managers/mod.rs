pub mod worker_states {
    pub struct Startup;
    pub struct Disconnected;
    pub struct Connected;
    pub struct Recovery;
}
pub mod messages;
mod main_loop;
mod udp;
mod tcp;

pub use udp::UdpManager;
pub use tcp::TcpManager;
