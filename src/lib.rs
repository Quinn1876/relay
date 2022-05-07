#[allow(unused_doc_comments)]
/**
 * In order to Compile Unit Tests in windows, we've defined socketcan as an optional dependency.
 * This makes it easier to test code in windows that does not use socketcan. socketcan requires
 * support for unix primitives which are not available on windows and so the code needs to be compiled on the raspberry pi itself for full testing.
 */

pub mod run_threads;
#[cfg(unix)]
pub mod can_extentions;

mod utils;
pub use utils::stream_utils;
pub use utils::requests;
pub use utils::device_watchdog;
pub mod project_butterfree;
pub mod pod_states;
pub mod board_states;
pub mod pod_data;
pub mod thread_managers;
pub mod error;
pub mod config;
