mod can_frame;
mod can_socket;
mod can_command;
mod error;
pub mod fault_reports;
pub mod ack_nack;
use error::CanError as Error;

//* Helper function for opening a can socket
pub fn open_socket<'a, S>(interface: S) 
-> Result<socketcan::CANSocket, Error>
where S: Into<&'a str> {
    Ok(socketcan::CANSocket::open(interface.into())?)
}

/**
 * Traits and Error Types defined by can_extentions
 */
pub mod prelude {
    pub use super::can_frame::FrameHandler;
    pub use super::can_socket::{ RoboteqCanSocket, RelayCanSocket };
    pub use super::error::CanError;
    pub use super::can_command::CanCommand;
}