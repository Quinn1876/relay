use chrono::{ NaiveDateTime };
use json::{
    object
};
use crate:: {
    pod_data::PodData,
    pod_states::PodState
};
use super::{
    errno::UdpErrno
};

pub struct PodStateMessage {
    current_state: PodState,
    pending_next_state: PodState,
    errno: UdpErrno,
    telemetry: Option<PodData>,
    telemetry_timestamp: NaiveDateTime,
    recovering: bool
}

impl PodStateMessage {
    pub fn to_json_bytes(&self) -> Vec<u8> {
        let telemetry: json::JsonValue = match self.telemetry {
            Some(data) => data.into(),
            _ => json::JsonValue::Null
        };
        let json_data = object!{
            current_state: self.current_state.to_byte(),
            pending_next_state: self.pending_next_state.to_byte(),
            errno: self.errno.to_byte(),
            telemetry: telemetry,
            telemetry_timestamp: self.telemetry_timestamp.timestamp(),
            recovering: self.recovering
        };
        json_data.dump().into_bytes()
    }

    pub fn new(current_state: PodState, pending_next_state: PodState, errno: UdpErrno, telemetry: &PodData, telemetry_timestamp: NaiveDateTime, recovering: bool) -> PodStateMessage {
        PodStateMessage {
            current_state,
            errno,
            pending_next_state,
            recovering,
            telemetry: Some((*telemetry).clone()),
            telemetry_timestamp,
        }
    }

    pub fn new_no_telemetry(current_state: PodState, pending_next_state: PodState, errno: UdpErrno, telemetry_timestamp: NaiveDateTime, recovering: bool) -> PodStateMessage {
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