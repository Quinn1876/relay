use std::io;

#[derive(Debug)]
pub enum CanError {
    FailedToOpenSocket(socketcan::CANSocketOpenError),
    MessageError(socketcan::ConstructionError),
    UnableToSetNonBlocking(io::Error),
    ReadError(io::Error),
    WriteError(io::Error),
}

impl From<socketcan::CANSocketOpenError> for CanError {
    fn from(error: socketcan::CANSocketOpenError) -> CanError {
        CanError::FailedToOpenSocket(error)
    }
}

impl From<socketcan::ConstructionError> for CanError {
    fn from(error: socketcan::ConstructionError) -> CanError {
        CanError::MessageError(error)
    }
}
