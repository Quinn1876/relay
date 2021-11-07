use polling::{ Event, Poller, Source };
use socketcan::{ CANSocket, ShouldRetry, CANFrame };
use std::time::Duration;
use std::net::{ TcpListener, TcpStream, IpAddr, Ipv4Addr, SocketAddr, UdpSocket };
use std::io::prelude::*;
use std::sync::{
    Arc,
    mpsc::{
        channel,
        Receiver,
        Sender
    }
};
use json::object;

use crate::roboteq::Roboteq;
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
    Connect,
    Disconnect,
    Unknown
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

#[derive(PartialEq)]
enum ServerState {
    Startup,
    Disconnected,
    Connected,
    Recovery
}

#[derive(PartialEq)]
enum PodStates {
    LowVoltage,
    Armed,
    AutoPilot,
    Braking,
    EmergencyBreak,
    SystemFailure,
    Resting,
    ManualOperationWaiting,
    Accellerating,
    AtSpeed,
    Decelerating,
    Invalid
    // More to come for manual operation
}

impl PodStates {
    fn to_byte(self) -> u8 {
        match self {
            PodStates::Resting                  => 0x00,
            PodStates::LowVoltage               => 0x01,
            PodStates::Armed                    => 0x02,
            PodStates::AutoPilot                => 0x03,
            PodStates::Braking                  => 0x04,
            PodStates::EmergencyBreak           => 0x05,
            PodStates::SystemFailure            => 0x06,
            PodStates::ManualOperationWaiting   => 0x07,
            PodStates::Accellerating            => 0x08,
            PodStates::AtSpeed                  => 0x09,
            PodStates::Decelerating             => 0x0A,
            PodStates::Invalid                  => 0x0B
        }
    }

    fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => PodStates::Resting,
            0x01 => PodStates::LowVoltage,
            0x02 => PodStates::Armed,
            0x03 => PodStates::AutoPilot,
            0x04 => PodStates::Braking,
            0x05 => PodStates::EmergencyBreak,
            0x06 => PodStates::SystemFailure,
            0x07 => PodStates::ManualOperationWaiting,
            0x08 => PodStates::Accellerating,
            0x09 => PodStates::AtSpeed,
            0x0A => PodStates::Decelerating,
            _ => PodStates::Invalid
        }
    }
}

trait RelayCan {
    fn send_pod_state(&self, state: &PodStates) -> std::io::Result<()>;
}

impl RelayCan for CANSocket {
    fn send_pod_state(&self, state: &PodStates) -> std::io::Result<()> {
        self.write_frame_insist(
            &CANFrame::new(0, &[state.to_byte()], false, false).expect("Frame to work")
        )
    }
}

// Describes a message that can be sent to the CAN thread
enum CANMessage {
    ChangeState(PodStates)
}

pub fn run_threads() -> Result<(), Error> {
    let (udpSender, udpReceiver) = channel();
    let (canSender, canReceiver): (Sender<CANMessage>, Receiver<CANMessage>) = channel();

    // TCP Thread
    std::thread::spawn(move || {
        // Open and Bind to port 8080 TODO: Move into config
        let listener = TcpListener::bind("0.0.0.0:8080").expect("Should be able to connect");
        let mut request_parser = requests::RequestParser::new();
        intialize_request_parser(&mut request_parser);
        let buffer_size = 128; // TODO: Move into Config

        // accept connections and process them sequentially
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => handle_tcp_socket_event(stream, &request_parser, buffer_size).unwrap(),
                Err(e) => println!("An Error Occured While Handling a TCP Conecction: {:?}", e),
            }
        }
    });

    // UDP Thread
    std::thread::spawn(move || {

    });

    // CAN Thread
    std::thread::spawn(move || {
        // Initialization
        let interface = "can0"; // TODO move to config
        let socket = socketcan::CANSocket::open(interface).expect(format!("Unable to Connect to CAN interface: {}", interface));

        let mut requested_pod_state = PodStates::LowVoltage;
        let mut bms_state = PodStates::LowVoltage;
        // TODO: IMPLE mc_state

        socket.set_read_timeout(Duration::from_millis(1)); // TODO Move Duration into config
        loop {
            // poll read
            let response = socket.read_frame(); // with timeout
            if response.should_retry() {
                // Timeout with no message

            } else if let Ok(frame) = response {
                // Frame Received
            } else {
                // ERROR
            }

            // check for state message from udp
            if let Ok(message) = canReceiver.try_recv() {
                match message {
                    CANMessage::ChangeState(newState) => {
                        requested_pod_state = newState;
                    }
                }
            }

            if requested_pod_state == bms_state && requested_pod_state == PodStates::AutoPilot {
                socket.set_motor_throttle(1, 1, 100); // TODO move this into config
            } else {
                socket.send_pod_state(&requested_pod_state)?;
            }

        }
    });


    // Worker Thread
    loop {

    }
    Ok(())
}

fn intialize_request_parser(request_parser: &mut requests::RequestParser<RequestTypes>) {
    /*
    * Add Supported TCP Queries here
    * Each Query string will correspond to a RequestType
    * Each Request Type will have a corresponding handler function which is ran
    * when the match occurs
    */
    request_parser.insert("CONNECT\r\n", RequestTypes::Connect);
    request_parser.insert("DISCONNECT\r\n", RequestTypes::Disconnect);
    request_parser.insert("@@Failed@@\r\n", RequestTypes::Unknown); // Special Message which is written into the request in the event of an error reading the message
}

/**
 * @func handle_tcp_socket_event
 * @brief
 */
fn handle_tcp_socket_event(mut stream: TcpStream, request_parser: &requests::RequestParser<RequestTypes>, buffer_size: usize) -> Result<(), Error> {
    let mut addr = stream.peer_addr().map_err(|e| Error::TcpSocketError(e))?;
    println!("Connected to a new stream with addr: {}", addr);
    let request = stream_utils::read_all(&mut stream, buffer_size).unwrap_or(b"@@Failed@@\r\n".to_vec());
    println!("Request: \n{}", std::str::from_utf8(&request).unwrap());

    /* Remove the Query String from the request and match it to the associated handler function */
    match request_parser.strip_line_and_get_value(request.as_slice()) {
        requests::RequestParserResult::Success((&value, _request)) => {
            match value {
                RequestTypes::Connect => {
                    println!("Connection Attempted received");
                    // TODO add error checking for existing connections
                    addr.set_port(8888);
                    // TODO Send message to UDP thread to connect to addr
                    stream.write(b"8888").map_err(|e| Error::TcpSocketError(e))?; // Tell the Handshake requester what udp port to listen on
                },
                RequestTypes::Disconnect => {
                    println!("Disconnect Received");
                    todo!()
                },
                RequestTypes::Unknown => {
                    println!("Received a Malformed Input");
                }
                _ => {
                    println!("RequestTypeParsed: {:?}", value);
                }
            }
        },
        requests::RequestParserResult::InvalidRequest => {
            println!("Invalid Request Received");
        },
        _ => ()
    };
    Ok(())
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

