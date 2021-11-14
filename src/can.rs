/**
 * @file can.rs
 * This module implements a socket can interface for connecting to the Staravia
 * board and the rest of the pod
 *
 */
use socketcan::CANSocket;
use std::fs;
use polling::{ Event, Poller };
use std::io;

use crate::pod_states::PodStates;

#[derive(Clone)]
pub struct Config {
    interface_name: String,
    // input_pipe_file: String,
    // output_pipe_file: String,
}

impl Config {
    pub fn default() -> Config {
        Config {
            interface_name: String::from("can0"),
            // input_pipe_file: String::from("/home/pi/pipes/relay/canInput"),
            // output_pipe_file: String::from("/home/pi/pipes/relay/canOutput"),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    FailedToOpenSocket(socketcan::CANSocketOpenError),
    PollerError(io::Error),
    UnableToSetNonBlocking(io::Error),
    ReadError(io::Error),
    WriteError(io::Error),
}


pub fn open_socket(config: &Config) -> Result<CANSocket, Error> {
    Ok(CANSocket::open(&config.interface_name).map_err(|e| Error::FailedToOpenSocket(e))?)
}

fn _run(config: Config) -> Result<(), Error> {
    let socket = CANSocket::open(&config.interface_name).map_err(|e| Error::FailedToOpenSocket(e))?;
    let socket_key = 21;
    let socket_poller = Poller::new().map_err(|e| Error::PollerError(e))?;

    socket_poller.add(&socket, Event::readable(socket_key)).map_err(|e| Error::PollerError(e))?;

    let mut messages = Vec::new();
    loop {
        messages.clear();
        socket_poller.wait(&mut messages, None).map_err(|e| Error::PollerError(e))?;
    }
}


// This is an incomplete list of CAN Commands.
// The full list that will eventually need to be supported
// can be found here: (Can Communication Protocol) [https://docs.google.com/spreadsheets/d/18rGH__yyJPf3jil74yTlVyFFqCOyuNzP3DCFmmIWWbo/edit#gid=0]
pub enum CanCommand {
    BmsHealthCheck { battery_pack_current: u32, cell_temperature: u32 },
    BmsStateChange(PodStates),
    Unknown(u32), // Arbitration ID provided for debugging Purposes
}

/**
 *
 * CAN FRAME HANDLER
 *
 * Functions for decoding a Can Frame
 */
pub trait FrameHandler {
    fn get_command(&self) -> CanCommand;
}

impl FrameHandler for socketcan::CANFrame {
    fn get_command(&self) -> CanCommand {
        let id = self.id();
        let data = self.data();

        match id {
            0x00B => CanCommand::BmsStateChange(get_state_change_data(data)),
            id => CanCommand::Unknown(id)
        }
    }
}

/**
 * @func:  get_state_change_data
 * @brief: A State change CAN frame will be a single byte which represents
 * the new state. Only the first byte will be checked regardless of if
 * there are more bytes in the data field. This is to allow expansion should
 * we in the future add more fields to these messages. The alternative would be
 * to throw an error if the data slice is larger then one, but this would create
 * fragile software without the proper error handling.
 */
fn get_state_change_data(data: &[u8]) -> PodStates {
    if data.len() == 0 { PodStates::Invalid }
    else { PodStates::from_byte(data[0]) }
}

