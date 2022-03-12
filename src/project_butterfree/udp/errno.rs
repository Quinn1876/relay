#[derive(Copy, Clone)]
pub enum UdpErrno {
    NoError,
    InvalidTransitionRequest,
    ArmingFault,
    ControllerTimeout,
    GeneralPodFailure
}

impl UdpErrno {
    pub fn to_byte(&self) -> u8 {
        match self {
            UdpErrno::NoError                  => 0x0,
            UdpErrno::InvalidTransitionRequest => 0x1,
            UdpErrno::ArmingFault              => 0x2,
            UdpErrno::ControllerTimeout        => 0x3,
            UdpErrno::GeneralPodFailure        => 0x4
        }
    }
}
