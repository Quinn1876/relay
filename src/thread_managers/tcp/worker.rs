use crate::error::Error;
use crate::requests;
use crate::stream_utils;

use super::super::worker_states::*;
use super::super::messages::*;
use super::super::main_loop::*;

use std::io::prelude::*;
use std::net::{
    TcpListener,
    TcpStream
};
use std::sync::mpsc::{
    Sender,
    Receiver,
};

#[derive(Copy, Clone, Debug)]
enum RequestTypes {
    Connect,
    Disconnect,
    Unknown
}

trait CustomTcpStream {
    fn write_message(&mut self, buf: &[u8]) -> Result<usize, Error>;
}

impl CustomTcpStream for TcpStream {
    fn write_message(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.write(buf).map_err(|e| Error::TcpSocketError(e))
    }
}

trait TcpWorkerRequestParser {
    fn init(self) -> Self;
}


impl TcpWorkerRequestParser for requests::RequestParser<RequestTypes> {
    fn init(mut self) -> Self {
        /*
        * Add Supported TCP Queries here
        * Each Query string will correspond to a RequestType
        * Each Request Type will have a corresponding handler function which is ran
        * when the match occurs
        */
        self.insert("CONNECT\r\n", RequestTypes::Connect);
        self.insert("DISCONNECT\r\n", RequestTypes::Disconnect);
        self.insert("@@Failed@@\r\n", RequestTypes::Unknown); // Special Message which is written into the request in the event of an error reading the message
        self
    }
}

#[repr(C)] // Required for type transmutations
pub struct TcpWorker<State = Disconnected> {
    listener: TcpListener,
    request_parser: requests::RequestParser<RequestTypes>,
    udp_message_sender: Sender<UDPMessage>,
    tcp_message_receiver: Receiver<TcpMessage>,
    tcp_message_buffer_size: usize,
    state: std::marker::PhantomData<State>
}

pub type TcpWorkerState = WorkerState<TcpWorker<Startup>, TcpWorker<Recovery>, TcpWorker<Connected>, TcpWorker<Disconnected>>;

impl TcpWorkerState {
    pub fn new<A: std::net::ToSocketAddrs>(
        address: A,
        udp_message_sender: Sender<UDPMessage>,
        tcp_message_receiver: Receiver<TcpMessage>,
        tcp_message_buffer_size: usize
    ) -> TcpWorkerState {
        TcpWorkerState::Disconnected(TcpWorker::new(address, udp_message_sender, tcp_message_receiver, tcp_message_buffer_size))
    }
}

impl TcpWorker {
    pub fn new<A: std::net::ToSocketAddrs>(
        address: A,
        udp_message_sender: Sender<UDPMessage>,
        tcp_message_receiver: Receiver<TcpMessage>,
        tcp_message_buffer_size: usize
    ) -> TcpWorker<Disconnected> {
        let listener = TcpListener::bind(address).expect("Unable to Connect to Port");
        listener.set_nonblocking(true).expect("Unable to set non blocking");
        TcpWorker {
            listener,
            request_parser: requests::RequestParser::new().init(),
            udp_message_sender,
            tcp_message_receiver,
            tcp_message_buffer_size,
            state: std::marker::PhantomData
        }
    }
}

/**
 * ALERT: Unsafe code!!!!!
 * Why it's safe:
 * std::mem::transmute takes one object and tells the compiler to treat it like another object. It is very easy to
 * screw this up when using pointers as the compiler would not be able to determine if the new types would be the same size upon dereference.
 * Thankfully, here we are only changing the meta data about the type. This means that the type itself is the same size and contains the same information.
 * We still need to be careful though!! The TcpWorker Struct is a generic type. Due to rules of Rust struct representation, two structs who differ only in
 * PhantomData types cannot be assumed to have the same layout in memory. This is why we opt to use the C representation of  the struct. While it is not as
 * efficient in terms of storage, it will always give us a consistent layout in memory which is important for these operations to work properly.
 */
impl<State> TcpWorker<State> {
    #[allow(non_snake_case)]
    fn EnterRecovery(self) -> TcpWorker<Recovery> {
        unsafe { std::mem::transmute::<TcpWorker<State>, TcpWorker<Recovery>>(self) }
    }
    #[allow(non_snake_case)]
    fn EnterConnected(self) -> TcpWorker<Connected> {
        unsafe { std::mem::transmute::<TcpWorker<State>, TcpWorker<Connected>>(self) }
    }
    #[allow(non_snake_case)]
    fn EnterDisconnected(self) -> TcpWorker<Disconnected> {
        unsafe { std::mem::transmute::<TcpWorker<State>, TcpWorker<Disconnected>>(self) }
    }
}

impl MainLoop<TcpWorkerState> for TcpWorker<Disconnected> {
    fn main_loop(mut self) -> TcpWorkerState {
        // Check for notifications from the other threads
        while let Ok(message) = self.tcp_message_receiver.try_recv() {
            match message {
                TcpMessage::EnteringRecovery => return TcpWorkerState::Recovery(self.EnterRecovery()),
                TcpMessage::RecoveryComplete => return TcpWorkerState::Disconnected(self),
                TcpMessage::UdpFailedToConnect => return TcpWorkerState::Disconnected(self)
            }
        }
        // Check for incoming connections on TCP Socket
        if let Some(stream) = self.listener.incoming().next() {
            match stream {
                Ok(s) => {
                    // do something with the TcpStream
                    match self.handle_connection(s) {
                        Ok(result) => match result {
                            RequestTypes::Connect => return TcpWorkerState::Connected(self.EnterConnected()),
                            _ => return TcpWorkerState::Disconnected(self),
                        },
                        Err(err) => {
                            println!("Error Occured While Processing TCP Stream: {:?}", err)
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return TcpWorkerState::Disconnected(self);
                }
                Err(e) => panic!("encountered IO error: {}", e),
            }
        }


        TcpWorkerState::Disconnected(self)
    }
}

impl TcpWorker<Disconnected> {
    fn handle_connection(
        &mut self,
        mut stream: TcpStream
    ) -> Result<RequestTypes, Error> {
        let mut addr = stream.peer_addr().map_err(|e| Error::TcpSocketError(e))?;
        println!("Connected to a new stream with addr: {}", addr);
        let request = stream_utils::read_all(&mut stream, self.tcp_message_buffer_size).unwrap_or(b"@@Failed@@\r\n".to_vec());
        println!("Request: \n{}", std::str::from_utf8(&request).unwrap());

        match self.request_parser.strip_line_and_get_value(request.as_slice()) {
            requests::RequestParserResult::Success((&value, _request)) => {
                match value {
                    RequestTypes::Connect => {
                        println!("Connection Attempt received");
                        addr.set_port(8081);
                        self.udp_message_sender.send(UDPMessage::ConnectToDesktop(addr)).expect("Should be able to send Message to UDP Socket from TCP Socket");
                        stream.write_message(b"OK 8090 8080")?;
                    },
                    RequestTypes::Disconnect => {
                        println!("TCP HANDLER: Received a disconnect request while not connected");
                        stream.write_message(b"DISCONNECTED")?;
                    },
                    RequestTypes::Unknown => {
                        println!("Received a Malformed Input");
                    }
                }
                return Ok(value);
            },
            requests::RequestParserResult::InvalidRequest => {
                println!("Invalid Request Received");
            },
            _ => {}
        }
        Err(Error::UnableToHandleTcpMessage)
    }
}

impl MainLoop<TcpWorkerState> for TcpWorker<Connected> {
    fn main_loop(mut self) -> TcpWorkerState {
        // Check for notifications from the other threads
        while let Ok(message) = self.tcp_message_receiver.try_recv() {
            match message {
                TcpMessage::EnteringRecovery => return TcpWorkerState::Recovery(self.EnterRecovery()),
                TcpMessage::RecoveryComplete => return TcpWorkerState::Disconnected(self.EnterDisconnected()),
                TcpMessage::UdpFailedToConnect => return TcpWorkerState::Disconnected(self.EnterDisconnected())
            }
        }
        // Check for incoming connections on TCP Socket
        if let Some(stream) = self.listener.incoming().next() {
            match stream {
                Ok(s) => {
                    // do something with the TcpStream
                    match self.handle_connection(s) {
                        Ok(result) => match result {
                            RequestTypes::Disconnect => return TcpWorkerState::Disconnected(self.EnterDisconnected()),
                            _ => {}
                        },
                        Err(err) => {
                            println!("Error Occured While Processing TCP Stream {:?}", err)
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return TcpWorkerState::Connected(self);
                }
                Err(e) => panic!("encountered IO error: {}", e),
            }
        }


        TcpWorkerState::Connected(self)
    }
}

impl TcpWorker<Connected> {
    fn handle_connection(
        &mut self,
        mut stream: TcpStream
    ) -> Result<RequestTypes, Error> {
        let addr = stream.peer_addr().map_err(|e| Error::TcpSocketError(e))?;
        println!("Connected to a new stream with addr: {}", addr);
        let request = stream_utils::read_all(&mut stream, self.tcp_message_buffer_size).unwrap_or(b"@@Failed@@\r\n".to_vec());
        println!("Request: \n{}", std::str::from_utf8(&request).unwrap());

        match self.request_parser.strip_line_and_get_value(request.as_slice()) {
            requests::RequestParserResult::Success((&value, _request)) => {
                match value {
                    RequestTypes::Connect => {
                        stream.write_message(b"ERROR POD Already Connected to Controller")?;
                    },
                    RequestTypes::Disconnect => {
                        println!("TCP THREAD: Disconnect Received");
                        self.udp_message_sender.send(UDPMessage::DisconnectFromHost).expect("Should be able to send message to UDP socket");
                        stream.write_message(b"DISCONNECTED")?;
                    },
                    RequestTypes::Unknown => {
                        println!("Received a Malformed Input");
                    }
                }
                return Ok(value);
            },
            requests::RequestParserResult::InvalidRequest => {
                println!("Invalid Request Received");
            },
            _ => {}
        }
        Err(Error::UnableToHandleTcpMessage)
    }
}


impl MainLoop<TcpWorkerState> for TcpWorker<Recovery> {
    fn main_loop(mut self) -> TcpWorkerState {
        // Check for notifications from the other threads
        while let Ok(message) = self.tcp_message_receiver.try_recv() {
            match message {
                TcpMessage::EnteringRecovery => return TcpWorkerState::Recovery(self.EnterRecovery()),
                TcpMessage::RecoveryComplete => return TcpWorkerState::Disconnected(self.EnterDisconnected()),
                TcpMessage::UdpFailedToConnect => {} // Continue in Recovery
            }
        }
        // Check for incoming connections on TCP Socket
        if let Some(stream) = self.listener.incoming().next() {
            match stream {
                Ok(s) => {
                    // do something with the TcpStream
                    match self.handle_connection(s) {
                        Ok(_) => return TcpWorkerState::Recovery(self),
                        Err(err) => {
                            println!("Error Occured While Processing TCP Stream {:?}", err)
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return TcpWorkerState::Recovery(self);
                }
                Err(e) => panic!("encountered IO error: {}", e),
            }
        }


        TcpWorkerState::Recovery(self)
    }
}

impl TcpWorker<Recovery> {
    fn handle_connection(
        &mut self,
        mut stream: TcpStream
    ) -> Result<(), Error> {
        let addr = stream.peer_addr().map_err(|e| Error::TcpSocketError(e))?;
        println!("Connected to a new stream with addr: {}", addr);
        let request = stream_utils::read_all(&mut stream, self.tcp_message_buffer_size).unwrap_or(b"@@Failed@@\r\n".to_vec());
        println!("Request: \n{}", std::str::from_utf8(&request).unwrap());

        match self.request_parser.strip_line_and_get_value(request.as_slice()) {
            requests::RequestParserResult::Success((&value, _request)) => {
                match value {
                    RequestTypes::Connect => {
                        stream.write_message(b"ERROR POD Already Connected to Controller")?;
                    },
                    RequestTypes::Disconnect => {
                        println!("TCP HANDLER: Received a disconnect request while not connected");
                        stream.write_message(b"DISCONNECTED")?;
                    },
                    RequestTypes::Unknown => {
                        println!("Received a Malformed Input");
                    }
                }
            },
            requests::RequestParserResult::InvalidRequest => {
                println!("Invalid Request Received");
            },
            _ => {}
        }
        Ok(())
    }
}

impl MainLoop<TcpWorkerState> for TcpWorker<Startup> {
    fn main_loop(self) -> TcpWorkerState {
        panic!("This function is here for completeness but it should never be called");
    }
}
