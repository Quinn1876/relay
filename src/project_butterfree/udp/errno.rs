use std::fmt;
use serde::{Serialize, Deserialize};
use serde::ser::{ Serializer };
use serde::de::{ self, Visitor, Deserializer };

#[derive(Copy, Clone)]
pub enum UdpErrno {
    NoError,
    InvalidTransitionRequest,
    ArmingFault,
    ControllerTimeout,
    GeneralPodFailure,
    UnknownError
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

impl From<u8> for UdpErrno {
    fn from(byte: u8) -> UdpErrno {
        match byte {
            0x0 => UdpErrno::NoError                  ,
            0x1 => UdpErrno::InvalidTransitionRequest ,
            0x2 => UdpErrno::ArmingFault              ,
            0x3 => UdpErrno::ControllerTimeout        ,
            0x4 => UdpErrno::GeneralPodFailure        ,
            _   => UdpErrno::UnknownError
        }
    }
}

struct PodStateVisitor;

impl<'de> Visitor<'de> for PodStateVisitor {
    type Value = UdpErrno;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between 0 and 0x0B")
    }

    fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UdpErrno::from(value))
    }

    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value < 0 {
            return Err(E::custom(format!("i8 out of range: {}", value)));
        }
        Ok(UdpErrno::from(value as u8))
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value > u32::from(u8::MAX) {
            return Err(E::custom(format!("u32 out of range: {}", value)));
        }
        Ok(UdpErrno::from(value as u8))
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value < 0 || value > i32::from(u8::MAX) {
            return Err(E::custom(format!("i32 out of range: {}", value)));
        }
        Ok(UdpErrno::from(value as u8))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value > u64::from(u8::MAX) {
            return Err(E::custom(format!("u64 out of range: {}", value)));
        }
        Ok(UdpErrno::from(value as u8))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value < 0 || value > i64::from(u8::MAX) {
            return Err(E::custom(format!("i64 out of range: {}", value)));
        }
        Ok(UdpErrno::from(value as u8))
    }
}

impl Serialize for UdpErrno {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.to_byte())
    }
}

impl<'de> Deserialize<'de> for UdpErrno {
    fn deserialize<D>(deserializer: D) -> Result<UdpErrno, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(PodStateVisitor)
    }
}
