pub mod worker_states {
    pub struct Startup;
    pub struct Disconnected;
    pub struct Connected;
    pub struct Recovery;
}
pub mod messages;
mod main_loop;

mod udp_worker;
mod udp_manger;
mod tcp_worker;
mod tcp_manager;

pub use udp_manger::UdpManager;
pub use tcp_manager::TcpManager;
