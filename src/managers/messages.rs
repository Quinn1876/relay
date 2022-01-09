use std::net::SocketAddr;
use crate::{
    pod_data,
    pod_states
};

pub enum TcpMessage {
    EnteringRecovery,
    #[allow(dead_code)] // Not Dead, but it's only constructed when running in unix
    RecoveryComplete,
}


#[derive(Debug)]
pub enum UDPMessage {
    ConnectToHost(SocketAddr),
    DisconnectFromHost,
    StartupComplete,
    #[allow(dead_code)] // Not Dead, only constructed when running in unix, but the udp socket needs to be able to check it in all cases
    PodStateChanged(pod_states::PodState),
    #[allow(dead_code)]
    TelemetryDataAvailable(pod_data::PodData, chrono::NaiveDateTime)
}

pub enum CanMessage {
    ChangeState(pod_states::PodState)
}
