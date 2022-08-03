#[derive(Debug, Eq, PartialEq)]
pub enum AckNack {
    Ack,
    Nack,
    UNKNOWN
}

impl From<u8> for AckNack {
    fn from(other: u8) -> AckNack {
        match other {
            0x00 => AckNack::Ack,
            0xFF => AckNack::Nack,
            _ => AckNack::UNKNOWN
        }
    }
}
