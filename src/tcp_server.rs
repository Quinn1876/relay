use polling::{ Event, Poller, Source };
use socketcan::CANSocket;
use std::time::Duration;
use std::net::{ TcpListener, TcpStream, IpAddr, Ipv4Addr, SocketAddr, UdpSocket };
use std::io::prelude::*;
use std::sync::Arc;
use json::object;

use crate::requests;
use crate::stream_utils;
use crate::can;

#[cfg(test)]
mod test {
    use super::*;
    use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    #[test]
    fn config_from_args_address() {
        let args = vec!["test program", "-a", "100.20.20.10:9090"];
        let args: Vec<String> = args.iter().map(|&arg| String::from(arg)).collect();
        let config_dut = Config::from_args(&args);
        let expected_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(100, 20, 20, 10)), 9090);
        assert_eq!(config_dut.address.ip(), expected_address.ip());
        assert_eq!(config_dut.address.port(), expected_address.port());
    }

    #[test]
    fn config_from_args_buffer_size() {
        let args = vec!["test program", "-b", "512"];
        let args: Vec<String> = args.iter().map(|&arg| String::from(arg)).collect();

        let config_dut = Config::from_args(&args);
        let expected_size: usize = 512;

        assert_eq!(config_dut.buffer_size, expected_size);
    }

    #[test]
    fn config_from_args_buffer_size_and_address() {
        let args = vec!["test program", "-b", "1024", "-a", "250.230.210.120:1000"];
        let args: Vec<String> = args.iter().map(|arg| String::from(*arg)).collect();

        let config_dut = Config::from_args(&args);

        let expected_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(250, 230, 210, 120)), 1000);
        let expected_size: usize = 1024;

        assert_eq!(config_dut.address.ip(), expected_address.ip());
        assert_eq!(config_dut.address.port(), expected_address.port());
        assert_eq!(config_dut.buffer_size, expected_size);
    }
}


pub struct Config<A: std::net::ToSocketAddrs> {
    address: A,
    buffer_size: usize,
    can_config: can::Config
}

impl<A: std::net::ToSocketAddrs> Config<A> {
    pub fn new(address: A, buffer_size: usize, can_config: can::Config) -> Config<A> {
        Config {
            address,
            buffer_size,
            can_config
        }
    }
}

impl Config<SocketAddr> {
    pub fn default() -> Config<SocketAddr> {
        Config {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            buffer_size: 256,
            can_config: can::Config::default()
        }
    }

    /**
     * @brief from_args
     * This builds a Config Item from a vector of command line arguments
     *
     * If the args vector is malformed, the function will panic and exit
     * TODO add proper error handling
     *
     * Currently Accepted arguments:
     * -a hostIpv4:port
     * -b buffer_size
     */
    pub fn from_args(args: &Vec<String>) -> Config<SocketAddr> {
        if args.len() % 2 == 0 {
            panic!("invalid arguments");
        }
        let mut i = args.len() - 1;
        let mut config = Config::default();

        while i > 1 {
            let param = &args[i];
            let param_type: &str = &args[i-1];

            match param_type {
                "-a" => {
                    let host_and_port: Vec<&str> = param.split(':').collect();
                    if host_and_port.len() != 2 {
                        panic!("Invalid address Argument, expected form -a <host>:<port>");
                    }
                    let host = host_and_port[0];
                    let port = host_and_port[1].parse::<u16>().unwrap();
                    let host: Vec<&str> = host.split('.').collect();

                    if host.len() != 4 {
                        panic!("Invalid host, expected form ##.##.##.##");
                    }

                    let host: Vec<u8> = host.iter().map(|val| val.parse::<u8>().unwrap()).collect();

                    config.address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(host[0], host[1], host[2], host[3])), port);
                },
                "-b" => {
                    let size = param.parse::<usize>().unwrap();

                    config.buffer_size = size;
                }
                _ => (),
            }
            i -= 2; // read arguments in pairs
        }
        config
    }
}

#[derive(Copy, Clone, Debug)]
enum RequestTypes {
    Handshake,
    Unknown,
    MockCAN
}

#[derive(Debug)]
pub enum Error {
    InvalidState(&'static str),
    TcpSocketError(std::io::Error),
    UdpSocketError(std::io::Error),
    CanSocketError(can::Error),
    PollerError(std::io::Error),
    InvalidAddr(std::io::Error),
    UninitializedUdpSocket,
    UninitializedCanSocket,
    AddrParseError,
}

struct PollKeys {
    tcp_socket: usize,
    udp_socket: usize,
    can_socket: usize,
    tcp_stream: usize,
}

#[derive(PartialEq)]
enum ServerState {
    Startup,
    Disconnected,
    UdpCanPassThrough,
}

enum PollType {
    Add,
    Modify
}

pub struct Server<A: std::net::ToSocketAddrs> {
    keys: PollKeys,
    tcp_socket: Option<TcpListener>,
    udp_socket: Option<Arc<UdpSocket>>,
    can_socket: Option<socketcan::CANSocket>,
    tcp_stream: Option<TcpStream>,
    socket_poller: Poller,                      // Socket poller is our interface to listen to the os to tell us when io (sockets) are ready
    current_state: ServerState,
    config: Config<A>,
    request_parser: requests::RequestParser::<RequestTypes>,
}

impl<A: std::net::ToSocketAddrs> Server<A> {
    pub fn new(config: Config<A>) -> Server<A> {
        Server {
            keys: PollKeys {
                tcp_socket: 1,
                udp_socket: 2,
                can_socket: 3,
                tcp_stream: 4
            },
            current_state: ServerState::Startup,
            can_socket: None,
            tcp_socket: None,
            tcp_stream: None,
            udp_socket: None,
            socket_poller: Poller::new().expect("Unable to create Poller"),
            config,
            request_parser: requests::RequestParser::new(),
        }
    }

    pub fn run_poll(&mut self) -> Result<(), Error> {
        if self.current_state != ServerState::Startup {
            return Err(Error::InvalidState("run poll called on a running server"));
        }

        if self.tcp_socket.is_some() {
            return Err(Error::InvalidState("tcp socket is initialized before entering initialization"))
        }

        self.initialize_tcp_socket()?;
        self.intialize_request_parser();

        let mut addr = self.config.address.to_socket_addrs().map_err(|e| Error::InvalidAddr(e))?.next().ok_or(Error::AddrParseError)?;
        addr.set_port(8888);
        self.open_udp_socket(addr)?;
        self.open_can_socket()?;

        let (sender, receiver): (std::sync::mpsc::Sender<socketcan::CANFrame>, std::sync::mpsc::Receiver<socketcan::CANFrame>) = std::sync::mpsc::channel();
        {

            let udp_socket = self.udp_socket.as_ref().expect("Udp Socket should be initialized by this point").clone();
            // START CAN FRAME HANDLER -> TODO Move this somewhere else =============================
            std::thread::spawn(move || loop {
                // When the sender goes out of scope, this call will fail and the thread will close
                let frame = receiver.recv().unwrap();
                if frame.is_error() {
                    println!("Frame Error Received");
                } else {
                    // TODO - Better Structure here including data translation and not just forwarding as json
                    let packet = object!{
                        id: frame.id(),
                        data: frame.data(),
                    };
                    match udp_socket.send(packet.dump().as_bytes()) {
                        Ok(bytes_written) => println!("Send {} bytes on udp", bytes_written),
                        Err(e) => println!("Error sending bytes: {:?}", e),
                    }
                }
            });
            // END CAN FRAME HANDLER ================================================================
        }


        self.poll_tcp_socket(PollType::Add)?;
        self.poll_can_socket(PollType::Add)?;
        self.poll_udp_socket(PollType::Add)?;
        self.current_state = ServerState::Disconnected;

        let mut events = Vec::new();
        loop {
            events.clear();
            self.poller_wait(&mut events, None)?;

            for event in &events {
                if event.key == self.keys.tcp_socket {
                    self.handle_tcp_socket_event(event)?;
                    self.poll_tcp_socket(PollType::Modify)?;
                } else if event.key == self.keys.udp_socket {
                    println!("UDP Message Received: {:?}", self.read_udp()?);
                    let frame = socketcan::CANFrame::new(0, b"test", false, false).map_err(|e| Error::InvalidState("Failed to make a frame"))?;
                    sender.send(frame).unwrap();
                    self.poll_udp_socket(PollType::Modify)?;
                } else if event.key == self.keys.can_socket {
                    sender.send(self.get_can_frame()?).unwrap();
                    self.poll_can_socket(PollType::Modify)?;
                }
            }
        }

    }

    /**
     * poll_*_socket
     * There are two values for these functions, either add or modify.
     * The first time the user would like to begin polling, the socket must be "added"
     * After each event received, the socket ust be repolled with a "Modify" call
     */
    // START poll_*_socket functions
    fn poll_tcp_socket(&self, poll_type: PollType) -> Result<(), Error> {
        match poll_type {
            PollType::Add => self.socket_poller.add(self.tcp_socket.as_ref().unwrap(), Event::readable(self.keys.tcp_socket)).map_err(|e| Error::PollerError(e)),
            PollType::Modify => self.socket_poller.modify(self.tcp_socket.as_ref().unwrap(), Event::readable(self.keys.tcp_socket)).map_err(|e| Error::PollerError(e))
        }
    }

    fn poll_udp_socket(&self, poll_type: PollType) -> Result<(), Error> {
        match poll_type {
            PollType::Add => self.socket_poller.add(&self.udp_socket.as_ref().unwrap() as &std::net::UdpSocket, Event::readable(self.keys.udp_socket)).map_err(|e| Error::PollerError(e)),
            PollType::Modify => self.socket_poller.modify(&self.udp_socket.as_ref().unwrap() as &std::net::UdpSocket, Event::readable(self.keys.udp_socket)).map_err(|e| Error::PollerError(e))
        }
    }

    fn poll_can_socket(&self, poll_type: PollType) -> Result<(), Error> {
        match poll_type {
            PollType::Add => self.socket_poller.add(self.can_socket.as_ref().unwrap(), Event::readable(self.keys.can_socket)).map_err(|e| Error::PollerError(e)),
            PollType::Modify => self.socket_poller.modify(self.can_socket.as_ref().unwrap(), Event::readable(self.keys.can_socket)).map_err(|e| Error::PollerError(e))
        }
    }
    // END OF poll_*_socket functions


    fn poller_wait(&self, events: &mut Vec<Event>, timeout: Option<Duration>) -> Result<usize, Error> {
        self.socket_poller.wait(events, timeout).map_err(|e| Error::PollerError(e))
    }

    fn initialize_tcp_socket(&mut self) -> Result<(), Error> {
        self.tcp_socket = Some(TcpListener::bind(&self.config.address).map_err(|e| Error::TcpSocketError(e))?);
        self.tcp_socket.as_ref().unwrap().set_nonblocking(true).map_err(|e| Error::TcpSocketError(e))?; // TCP Socket neesd to be non_blocking so that it can be polled
        println!("Listening on {}", self.tcp_socket.as_ref().unwrap().local_addr().ok().unwrap());
        Ok(())
    }

    fn intialize_request_parser(&mut self) {
        /*
        * Add Supported TCP Queries here
        * Each Query string will correspond to a RequestType
        * Each Request Type will have a corresponding handler function which is ran
        * when the match occurs
        */
        self.request_parser.insert("HANDSHAKE\r\n", RequestTypes::Handshake);
        self.request_parser.insert("SEND MOCK CAN\r\n", RequestTypes::MockCAN);
        self.request_parser.insert("@@Failed@@\r\n", RequestTypes::Unknown); // Special Message which is written into the request in the event of an error reading the message
    }

    fn open_udp_socket<Addr: std::net::ToSocketAddrs>(&mut self, addr: Addr) -> Result<(), Error> {
        self.udp_socket = Some(Arc::new(UdpSocket::bind(addr).map_err(|e| Error::UdpSocketError(e))?));
        self.udp_socket.as_ref().unwrap().set_nonblocking(true).map_err(|e| Error::UdpSocketError(e))?;
        Ok(())
    }

    fn open_can_socket(&mut self) -> Result<(), Error> {
        self.can_socket = Some(can::open_socket(&self.config.can_config).map_err(|e| Error::CanSocketError(e))?);
        self.can_socket.as_ref().unwrap().set_nonblocking(true).map_err(|e| Error::CanSocketError(can::Error::UnableToSetNonBlocking(e)))?;
        Ok(())
    }


    /**
     * @func handle_tcp_socket_event
     * @brief
     */
    fn handle_tcp_socket_event(&mut self, _event: &Event) -> Result<(), Error> {
        if let Some(tcp_socket) = &self.tcp_socket {
            match tcp_socket.accept() {
                Ok((mut stream, mut addr)) => {
                    println!("Connected to a new stream with addr: {}", addr);
                    let request = stream_utils::read_all(&mut stream, self.config.buffer_size).unwrap_or(b"@@Failed@@\r\n".to_vec());
                    println!("Request: \n{}", std::str::from_utf8(&request).unwrap());
                    /* Remove the Query String from the request and match it to the associated handler function */
                    match self.request_parser.strip_line_and_get_value(request.as_slice()) {
                        requests::RequestParserResult::Success((&value, request)) => {
                            println!("Value: {:?}", value);
                            match value {
                                RequestTypes::Handshake => {
                                    println!("HandShake received");
                                    addr.set_port(8888);
                                    self.udp_socket.as_ref().unwrap().connect(addr).map_err(|e| Error::UdpSocketError(e))?; // Connect to the desktop to send it messages over udp
                                    stream.write(b"8888").map_err(|e| Error::TcpSocketError(e))?; // Tell the Handshake requester what udp port to listen on
                                },
                                RequestTypes::MockCAN => {
                                    println!("Send Mock Can Received");
                                    handle_mock_can(request, &mut stream).unwrap();
                                },
                                RequestTypes::Unknown => {
                                    println!("Received a Malformed Input");
                                }
                                _ => ()
                            }
                        },
                        requests::RequestParserResult::InvalidRequest => {
                            println!("Invalid Request Received");
                        },
                        _ => ()
                    };
                },
                Err(e) => return Err(Error::TcpSocketError(e))
            }
        }
        Ok(())
    }

    fn get_can_frame(&mut self) -> Result<socketcan::CANFrame, Error> {
        match self.can_socket.as_ref() {
            Some(can_socket) => {
                Ok(can_socket.read_frame().map_err(|e| Error::CanSocketError(can::Error::ReadError(e)))?)
            },
            None => Err(Error::UninitializedCanSocket)
        }
    }

    fn read_udp(&mut self) -> Result<Vec<u8>, Error> {
        match self.udp_socket.as_ref() {
            Some(udp_socket) => {
                let mut buffer = [0u8; 256];
                let (amount, src) = udp_socket.recv_from(&mut buffer).map_err(|e| Error::UdpSocketError(e))?;
                // TODO: LOG Events
                Ok(buffer.to_vec())
            },
            None => Err(Error::UninitializedUdpSocket)
        }
    }
    fn write_udp(&mut self, msg: &[u8], dest: std::net::SocketAddr) -> Result<(), Error> {
        match self.udp_socket.as_ref() {
            Some(udp_socket) => {
                udp_socket.send_to(msg, dest).map_err(|e| Error::UdpSocketError(e))?;
                // TODO: Log Event
                Ok(())
            },
            None => Err(Error::UninitializedUdpSocket)
        }
    }
}


/**
 * @func handle_handshake
 * @param _request: This is the body of the request
 * @param stream: Tcp stream, can be written to and read from
 *
 * @brief: This is the main function for running the POD
 * A UDP port is bound and watched for information coming from the
 * controller
 * Once opened, the controller is notified of the port to begin sending to
 * over the tcp stream.
 *
 * TODO: Interface with the Can Board to retrieve CAN Packets to send to the controller
 * TODO: Interface with the CAN Board to send CAN packets with information from the controller
 */
fn handle_handshake(_request: &[u8], stream: &mut TcpStream, can_config: & can::Config) -> Result<(), Error> {

    let mut addr = stream.local_addr().map_err(|e| Error::UdpSocketError(e))?;
    addr.set_port(8888);

    let udp_socket = UdpSocket::bind(addr).map_err(|e| Error::UdpSocketError(e))?;
    let mut can_socket = can::open_socket(can_config).map_err(|e| Error::CanSocketError(e))?;

    let udp_key = 0;
    let can_key = 1;

    let socket_poller = Poller::new().map_err(|e| Error::PollerError(e))?;
    socket_poller.add(&can_socket, Event::readable(can_key)).map_err(|e| Error::PollerError(e))?;
    socket_poller.add(&udp_socket, Event::readable(udp_key)).map_err(|e| Error::PollerError(e))?;


    println!("Bound to udpSocket {}", addr);
    stream.write(b"8888").map_err(|e| Error::UdpSocketError(e))?; // Tell the Handshake requester what port to listen on

    let mut events = Vec::new();
    loop {
        events.clear();
        socket_poller.wait(&mut events, None).map_err(|e| Error::PollerError(e))?;

        for event in &events {
            if event.key == udp_key {
                let mut buffer = [0u8; 256];
                let (amount, src) = udp_socket.recv_from(&mut buffer).map_err(|e| Error::UdpSocketError(e))?;
                println!("UDP Packet: {}", String::from_utf8(buffer.to_vec()).unwrap());

                socket_poller.modify(&udp_socket, Event::readable(udp_key)).map_err(|e| Error::PollerError(e))?;
            }
            else if event.key == can_key {
                let frame = can_socket.read_frame().unwrap();
                println!("Frame: {:?}", frame);
                socket_poller.modify(&can_socket, Event::readable(can_key)).map_err(|e| Error::PollerError(e))?;
            }
        }
    }
}


fn handle_mock_can(_request: &[u8], stream: &mut TcpStream) -> std::io::Result<()> {
    let mut addr = stream.local_addr()?;
    addr.set_port(8888);
    let udp_socket = UdpSocket::bind(addr)?;
    println!("Bound to udpSocket {}", addr);
    stream.write(b"8888")?; // Tell the Handshake requester what port to listen on

    let mut buffer = [0u8; 256];

    let (_amount, src) = udp_socket.recv_from(&mut buffer)?;

    for _i in 0..7 {
        let buf = [b'B', b'M', b'S', b'1', b'\r', b'\n', 100u8];
        udp_socket.send_to(&buf, src)?;
    }

    Ok(())
}

