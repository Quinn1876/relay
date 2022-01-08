pub mod worker_states {
    pub struct Startup;
    pub struct Disconnected;
    pub struct Connected;
    pub struct Recovery;
}

pub mod udp_worker;
pub mod messages;
