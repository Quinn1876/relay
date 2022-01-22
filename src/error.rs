#[derive(Debug)]
pub enum Error {
    InvalidState(&'static str),
    TcpSocketError(std::io::Error),
    UdpSocketError(std::io::Error),
    #[cfg(unix)]
    CanSocketError(crate::can::Error),
    PollerError(std::io::Error),
    InvalidAddr(std::io::Error),
    UninitializedUdpSocket,
    UninitializedCanSocket,
    AddrParseError,
    UnableToHandleTcpMessage,
}
