use std::net::SocketAddr;
#[cfg(unix)]
use socketcan::CANFrame;
use crate::{
    pod_data,
    pod_states,
};

pub enum TcpMessage {
    EnteringRecovery,
    #[allow(dead_code)] // Not Dead, but it's only constructed when running in unix
    RecoveryComplete,
    UdpFailedToConnect,
}

#[derive(Debug)]
pub enum UDPMessage {
    ConnectToDesktop(SocketAddr),
    DisconnectFromHost,
    StartupComplete,
    #[allow(dead_code)] // Not Dead, only constructed when running in unix, but the udp socket needs to be able to check it in all cases
    PodStateChangeAck,
    #[allow(dead_code)]
    TelemetryDataAvailable(pod_data::PodData, chrono::NaiveDateTime),
    SystemFault
}

#[derive(Clone)]
pub enum CanMessage {
    ChangeState(pod_states::PodState),
    DeviceLost
}

pub enum WorkerMessage {
    CanFrameAndTimeStamp(CANFrame, chrono::NaiveDateTime)
}
