use chrono::{ NaiveDateTime };

use crate:: {
    pod_data::PodData,
    pod_states::PodState
};
use super::{
    errno::UdpErrno,
    serializable_date::SerializableDate
};
use serde_derive::{ Serialize as DSerialize, Deserialize as DDeserialize };


#[derive(DSerialize, DDeserialize)]
pub struct PodStateMessage {
    current_state: PodState,
    pending_next_state: PodState,
    errno: UdpErrno,
    telemetry: Option<PodData>,
    telemetry_timestamp: SerializableDate,
    recovering: bool
}

impl PodStateMessage {
    pub fn new(current_state: PodState, pending_next_state: PodState, errno: UdpErrno, telemetry: &PodData, telemetry_timestamp: NaiveDateTime, recovering: bool) -> PodStateMessage {
        PodStateMessage {
            current_state,
            errno,
            pending_next_state,
            recovering,
            telemetry: Some((*telemetry).clone()),
            telemetry_timestamp: telemetry_timestamp.into(),
        }
    }

    pub fn new_no_telemetry(current_state: PodState, pending_next_state: PodState, errno: UdpErrno, telemetry_timestamp: NaiveDateTime, recovering: bool) -> PodStateMessage {
        PodStateMessage {
            current_state,
            errno,
            pending_next_state,
            recovering,
            telemetry: None,
            telemetry_timestamp: telemetry_timestamp.into(),
        }
    }
}
