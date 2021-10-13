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
    UnableToSetNonBlocking(io::Error)
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
