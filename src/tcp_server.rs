#[allow(unused_doc_comments)]

use std::time::Duration;
use std::net::{
    TcpListener,
    TcpStream,
    IpAddr,
    Ipv4Addr,
    SocketAddr,
};
use std::io::prelude::*;
use std::sync::{
    mpsc::{
        channel,
        Receiver,
        Sender
    }
};
#[cfg(unix)]
use socketcan::{ CANSocket, ShouldRetry, CANFrame };

#[cfg(unix)]
use crate::roboteq::Roboteq;
use crate::requests;
use crate::stream_utils;
#[cfg(unix)]
use crate::can;
#[cfg(unix)]
use crate::can::{CanCommand, FrameHandler};

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
    #[cfg(unix)]
    can_config: can::Config
}

impl<A: std::net::ToSocketAddrs> Config<A> {
    #[cfg(unix)]
    pub fn new(address: A, buffer_size: usize, can_config: can::Config) -> Config<A> {
        Config {
            address,
            buffer_size,
            can_config
        }
    }
    #[cfg(windows)]
    pub fn new(address: A, buffer_size: usize) -> Config<A> {
        Config {
            address,
            buffer_size,
        }
    }
}

impl Config<SocketAddr> {
    #[cfg(unix)]
    pub fn default() -> Config<SocketAddr> {
        Config {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            buffer_size: 256,
            can_config: can::Config::default()
        }
    }
    #[cfg(windows)]
    pub fn default() -> Config<SocketAddr> {
        Config {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            buffer_size: 256,
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

#[derive(PartialEq, Copy, Clone, Debug)]
enum ServerState {
    Startup,
    Disconnected,
    Connected,
    Recovery
}

#[cfg(feature = "socketcan")]
#[derive(Debug)]
pub enum CanError {
    MessageError(socketcan::ConstructionError),
    WriteError(std::io::Error)
}

// START TODO: Move Section to own file
#[cfg(unix)]
trait RelayCan {
    fn send_pod_state(&self, state: &PodState) -> Result<(), CanError>;
}

#[cfg(feature = "socketcan")]
impl RelayCan for CANSocket {
    fn send_pod_state(&self, state: &PodState) -> Result<(), CanError> {
        self.write_frame_insist(
            &CANFrame::new(0, &[state.to_byte()], false, false).map_err(|e| CanError::MessageError(e))?
        ).map_err(|e| CanError::WriteError(e))
    }
}
// END Section

// Describes a message that can be sent to the CAN thread



#[cfg(unix)]
enum WorkerMessage {
    CanFrameAndTimeStamp(CANFrame, NaiveDateTime)
}

use crate::workers::messages::{
    TcpMessage,
    UDPMessage,
    CanMessage as CANMessage
};
use crate::workers;
use crate::error::Error;

pub fn run_threads() -> Result<(), Error> {
    let (udp_message_sender, udp_message_receiver): (Sender<UDPMessage>, Receiver<UDPMessage>) = channel();
    #[allow(unused_variables)] // can_message_receiver is only used in unix, but needs to exist so that other parts of the code can send messages without crashing
    let (can_message_sender, can_message_receiver): (Sender<CANMessage>, Receiver<CANMessage>) = channel();
    #[cfg(unix)] // Worker does not need to be created if running outside of unix
    let (worker_message_sender, worker_message_receiver): (Sender<WorkerMessage>, Receiver<WorkerMessage>) = channel();
    let (tcp_sender, tcp_receiver): (Sender<TcpMessage>, Receiver<TcpMessage>) = channel();

    // Configuration Values
    let tcp_message_buffer_size = 128;
    // TODO - Figure out what these value should be
    let udp_socket_read_timeout = Duration::from_millis(6000); // Amount of time the UDP Socket will wait for a message from the Controller
    let udp_max_number_timeouts = 10;
    // End Configuration Values

    // CAN Configuration
    #[cfg(unix)]
    let can_interface = "can0";
    #[cfg(unix)]
    let can_socket_read_timeout = Duration::from_millis(10000); // Amount of time the CAN Socket will wait for a message from the rest of the POD
    // End CAN Configuration

    // Thread Handles
    let tcp_handle = workers::TcpManager::run("0.0.0.0:8080", udp_message_sender.clone(), tcp_receiver, tcp_message_buffer_size);
    let udp_handle = workers::UdpManager::run(can_message_sender.clone(), tcp_sender.clone(), udp_message_receiver,udp_max_number_timeouts);

    #[allow(unused_doc_comments)]
    /**
     * @thread CAN Thread
     *
     * @desc The CAN thread is responsible for reading and writing to the CAN bus
     *       The CAN Thread tracks the time that a message is received and passes along
     *          can frames and their timestamp to the worker thread
     *      The Message that the CAN Frame sends depends on the state of the <Relay Board, Pod>:
     *          <Connected, AutoPilot>: Send Roboteq Throttle message
     *          <Connected, NotAutoPilot>: Send StateID
     *          <Disconnected, LowVoltage>: Send StateID
     *          <Recovery, *>: Send StateID
     *
     */
    #[cfg(unix)]
    {
        let udpSender = udpSender.clone();
        std::thread::Builder::new().name("CAN Thread".to_string()).spawn(move || {
            // Initialization
            if cfg!(unix) {
                let socket = socketcan::CANSocket::open(can_interface).expect(&format!("Unable to Connect to CAN interface: {}", can_interface));

                let mut requested_pod_state = PodState::LowVoltage;
                let mut bms_state = PodState::LowVoltage;
                // TODO: IMPLE mc_state

                socket.set_read_timeout(can_socket_read_timeout).expect("Unable to Set Timeout on CAN Socket");
                loop {
                    // poll read
                    let response = socket.read_frame(); // with timeout
                    if response.should_retry() {
                        // Timeout with no message
                        println!("CAN SOCKET: Read timeout no message Received");
                    } else if let Ok(frame) = response {
                        // Frame Received
                        // Check for state messages before passing the frame on to the worker
                        if let CanCommand::BmsStateChange(newState) = frame.get_command() {
                            bms_state = newState;
                            udpSender.send(UDPMessage::PodStateChanged(newState)).expect("To Be able to send message to udp from can");
                        }
                        worker_message_sender.send(WorkerMessage::CanFrameAndTimeStamp(frame, Utc::now().naive_local())).expect("Unable to send message from CAN Thread on Worker Channel");
                    } else {
                        // ERROR Reading from Can socket
                    }

                    // check for state message from udp
                    if let Ok(message) = can_message_receiver.try_recv() {
                        match message {
                            CANMessage::ChangeState(new_state) => {
                                requested_pod_state = new_state;
                            }
                        }
                    }

                    let message_result;
                    if requested_pod_state == bms_state && requested_pod_state == PodState::AutoPilot {
                        message_result = socket.set_motor_throttle(1, 1, 100); // TODO move this into config
                    } else {
                        message_result = socket.send_pod_state(&requested_pod_state);
                    }

                    match message_result {
                        Ok(()) => {},
                        Err(err) => {
                            println!("Error Sending Message on CAN bus: {:?}",  err);
                        }
                    }
                }
            }
        }).expect("Should be able to create Thread");
    }

    // Worker Thread
    // Initialization
    #[cfg(unix)]
    {
        let mut pod_data = PodData::new();
        loop {
            match worker_message_receiver.recv() {
                Ok(message) => {
                    match message {
                        WorkerMessage::CanFrameAndTimeStamp(frame, time) => {
                            // Handle CAN Frame in here
                            let mut new_data = true;
                            match frame.get_command() {
                                CanCommand::BmsHealthCheck{ battery_pack_current, cell_temperature } => {},
                                CanCommand::PressureHigh(pressure) => pod_data.pressure_high = Some(pressure),
                                CanCommand::PressureLow1(pressure) => pod_data.pressure_low_1 = Some(pressure),
                                CanCommand::PressureLow2(pressure) => pod_data.pressure_low_2 = Some(pressure),
                                CanCommand::Torchic1(data) => pod_data.torchic_1 = data,
                                CanCommand::Torchic2(data) => pod_data.torchic_2 = data,
                                _ => {
                                    new_data = false;
                                }
                            }
                            if new_data {
                                udpSender.send(UDPMessage::TelemetryDataAvailable(pod_data, time)).expect("To be able to send telemetry data to udp from worker");
                            }
                        }
                    }
                },
                Err(err) => {
                    println!("Worker Receiver Error: {:?}", err);
                    println!("Exiting");
                    return Ok(());
                }
            }
        }
    }

    udp_handle.join().expect("Should be able to join at the end of the Program");
    tcp_handle.join().expect("Should be able to join at the end of the Program");

    Ok(())
}

