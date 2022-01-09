#[allow(unused_doc_comments)]
/**
 * In order to Compile Unit Tests in windows, we've defined socketcan as an optional dependency.
 * This makes it easier to test code in windows that does not use socketcan. socketcan requires
 * support for unix primitives which are not available on windows and so the code needs to be compiled on the raspberry pi itself for full testing.
 */

pub mod tcp_server;
#[cfg(unix)]
pub mod roboteq;
#[cfg(unix)]
pub mod can;

pub mod stream_utils;
pub mod requests;
pub mod udp_messages;
pub mod pod_states;
pub mod pod_data;
pub mod workers;
pub mod error;
