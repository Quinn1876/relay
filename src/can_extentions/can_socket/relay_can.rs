/**
 * @Trait RelayCanSocket
 * @brief A CANSocket extention which implements methods
 * that allow for sending messages as defined in:
 * https://docs.google.com/document/d/1pAAAPyWClxrq7MwrA0_AGxnqU6B5r5MHmvRERMY6hUo/edit
 *
 * @authors Quinn Hodges
 */

use crate::pod_states::PodState;
use crate::can_extentions::prelude::CanError as Error;
use socketcan::{ CANSocket, CANFrame };

pub trait RelayCanSocket {
    fn send_pod_state(&self, state: &PodState) -> Result<(), Error>;
}

impl RelayCanSocket for CANSocket {
    fn send_pod_state(&self, state: &PodState) -> Result<(), Error> {
        self.write_frame_insist(
            &CANFrame::new(0, &[state.into()], false, false)?
        ).map_err(|e| Error::WriteError(e))
    }
}
