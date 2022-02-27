const REQUESTED_STATE: &'static str = "requested_state";
const MOST_RECENT_TIMESTAMP: &'static str = "most_recent_timestamp";
use json::{
    object,
    JsonValue::Number
};
use chrono::{ NaiveDateTime };
use crate::pod_states::PodState;

pub struct DesktopStateMessage {
    pub requested_state: PodState,
    pub most_recent_timestamp: NaiveDateTime
}

#[derive(Debug)]
pub enum DesktopStateMessageError {
    JsonParseError(json::Error),
    InvalidMessage(String),
}


impl DesktopStateMessage {
    pub fn from_json_bytes(json_bytes: &[u8]) -> Result<DesktopStateMessage, DesktopStateMessageError> {
        let string = &String::from_utf8_lossy(json_bytes);
        let trimmed = string.trim_matches(char::from(0)); // Remove NULL Terminators if there are any from the buffer
        let parsed = json::parse(trimmed).map_err(|e| DesktopStateMessageError::JsonParseError(e))?;
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
                                requested_state: PodState::from_byte(requested_state_byte),
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
