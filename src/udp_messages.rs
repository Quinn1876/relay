use json::{
    object,
    JsonValue::Number
};
use chrono::{ NaiveDateTime };
use crate::pod_states::PodStates;
use crate::pod_data::PodData;
use std::net::UdpSocket;

pub struct DesktopStateMessage {
    pub requested_state: PodStates,
    pub most_recent_timestamp: NaiveDateTime
}

#[derive(Debug)]
pub enum DesktopStateMessageError {
    JsonParseError(json::Error),
    InvalidMessage(String),
}

mod test {
    use super::DesktopStateMessage;
    #[test]
    fn test_json_bytes() {
        let message = b"{\"requested_state\":1,\"most_recent_timestamp\":1636842789806}";
        assert_eq!(DesktopStateMessage::from_json_bytes(message).expect("Unable to Convert message").to_json_bytes(), message);
    }
}

const REQUESTED_STATE: &'static str = "requested_state";
const MOST_RECENT_TIMESTAMP: &'static str = "most_recent_timestamp";

impl DesktopStateMessage {
    pub fn from_json_bytes(json_bytes: &[u8]) -> Result<DesktopStateMessage, DesktopStateMessageError> {
        let parsed = json::parse(&String::from_utf8_lossy(json_bytes)).map_err(|e| DesktopStateMessageError::JsonParseError(e))?;
        if parsed[REQUESTED_STATE].is_null() || parsed[MOST_RECENT_TIMESTAMP].is_null() {
            return Err(DesktopStateMessageError::InvalidMessage(format!("An expected field was null in message: {:?}", parsed.dump())));
        }
        if let Number(requested_state) = parsed[REQUESTED_STATE] {
            if let Number(timestamp) = parsed[MOST_RECENT_TIMESTAMP] {
                if let Some(requested_state) = requested_state.as_fixed_point_u64(0) {
                    if requested_state < 256 {
                        let requested_state_byte = (requested_state & 0xff) as u8;
                        if let Some(timestamp) = timestamp.as_fixed_point_i64(0) {
                            return Ok(DesktopStateMessage {
                                requested_state: PodStates::from_byte(requested_state_byte),
                                most_recent_timestamp: NaiveDateTime::from_timestamp(timestamp, 0)
                            });
                        }
                    }
                }
            }
        }
        return Err(DesktopStateMessageError::InvalidMessage(format!("Unable to read numbers from parsed message: {:?}", parsed.dump())));
    }

    pub fn to_json_bytes(&self) -> Vec<u8> {
        let json_data = object!{
            requested_state: self.requested_state.to_byte(),
            most_recent_timestamp: self.most_recent_timestamp.timestamp(),
        };

        json_data.dump().into_bytes()
    }
}

#[derive(Copy, Clone)]
pub enum Errno {
    NoError,
    InvalidTransitionRequest,
    ArmingFault,
    ControllerTimeout,
    GeneralPodFailure
}

impl Errno {
    pub fn to_byte(&self) -> u8 {
        match self {
            Errno::NoError                  => 0x0,
            Errno::InvalidTransitionRequest => 0x1,
            Errno::ArmingFault              => 0x2,
            Errno::ControllerTimeout        => 0x3,
            Errno::GeneralPodFailure        => 0x4
        }
    }
}

pub struct PodStateMessage {
    current_state: PodStates,
    pending_next_state: PodStates,
    errno: Errno,
    telemetry: Option<PodData>,
    telemetry_timestamp: NaiveDateTime,
    recovering: bool
}

impl PodStateMessage {
    pub fn to_json_bytes(&self) -> Vec<u8> {
        let json_data = object!{
            current_state: self.current_state.to_byte(),
            pending_next_state: self.pending_next_state.to_byte(),
            errno: self.errno.to_byte(),
            telemetry: if let Some(telemetry) = self.telemetry.as_ref() { telemetry.to_json() } else { json::JsonValue::Null },
            telemetry_timestamp: self.telemetry_timestamp.timestamp(),
            recovering: self.recovering
        };
        json_data.dump().into_bytes()
    }

    pub fn new(current_state: PodStates, pending_next_state: PodStates, errno: Errno, telemetry: &PodData, telemetry_timestamp: NaiveDateTime, recovering: bool) -> PodStateMessage {
        PodStateMessage {
            current_state,
            errno,
            pending_next_state,
            recovering,
            telemetry: Some(*telemetry).clone(),
            telemetry_timestamp,
        }
    }

    pub fn new_no_telemetry(current_state: PodStates, pending_next_state: PodStates, errno: Errno, telemetry_timestamp: NaiveDateTime, recovering: bool) -> PodStateMessage {
        PodStateMessage {
            current_state,
            errno,
            pending_next_state,
            recovering,
            telemetry: None,
            telemetry_timestamp,
        }
    }
}

pub trait CustomUDPSocket {
    fn send_pod_state_message(&self, msg: &PodStateMessage) -> std::io::Result<usize>;
}

impl CustomUDPSocket for UdpSocket {
    fn send_pod_state_message(&self, msg: &PodStateMessage) -> std::io::Result<usize> {
        self.send(&msg.to_json_bytes())
    }
}
