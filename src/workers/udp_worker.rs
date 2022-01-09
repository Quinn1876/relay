use std::net::{
    UdpSocket,
};
use crate::{
    pod_data,
    pod_states,
    udp_messages,
    udp_messages::{
        UdpErrno
    }
};
use crate::udp_messages::CustomUDPSocket;

use chrono;
use std::sync::mpsc::{
    Sender,
    Receiver
};

use super::worker_states::*;
use super::messages::*;
use super::main_loop::*;

#[repr(C)] // Required for type transmutations
pub struct UdpWorker<State = Startup> {
    udp_socket: UdpSocket,
    current_pod_state: pod_states::PodState,
    next_pod_state: pod_states::PodState,
    errno: UdpErrno,
    timeout_counter: u32,
    last_received_telemetry_timestamp: chrono::NaiveDateTime,
    current_pod_data: pod_data::PodData,
    current_telemetry_timestamp: chrono::NaiveDateTime,
    tcp_sender: Sender<TcpMessage>,
    udp_message_receiver: Receiver<UDPMessage>,
    can_message_sender: Sender<CanMessage>,
    udp_max_number_timeouts: u32,
    state: std::marker::PhantomData<State>
}

impl UdpWorker<Connected> {
    fn send_pod_state_message(&self) {
        // Send Message Back to Desktop
        let pod_state_message = if self.current_telemetry_timestamp.timestamp() > self.last_received_telemetry_timestamp.timestamp() {
            udp_messages::PodStateMessage::new(self.current_pod_state, self.next_pod_state, self.errno, &self.current_pod_data, self.current_telemetry_timestamp, false)
        } else {
            udp_messages::PodStateMessage::new_no_telemetry(self.current_pod_state, self.next_pod_state, self.errno, self.current_telemetry_timestamp, false)
        };
        match self.udp_socket.send_pod_state_message(&pod_state_message) {
            Ok(bytes_sent) => {
                // println!("UDP THREAD: Send {} to Desktop", bytes_sent);
            },
            Err(error) => {
                println!("Error Sending Message on UDP Thread: {:?}", error);
            }
        }
    }
}


impl UdpWorker<Recovery> {
    fn send_pod_state_message(&self) {
        // Send Message Back to Desktop
        let pod_state_message = if self.current_telemetry_timestamp.timestamp() > self.last_received_telemetry_timestamp.timestamp() {
            udp_messages::PodStateMessage::new(self.current_pod_state, self.next_pod_state, self.errno, &self.current_pod_data, self.current_telemetry_timestamp, true)
        } else {
            udp_messages::PodStateMessage::new_no_telemetry(self.current_pod_state, self.next_pod_state, self.errno, self.current_telemetry_timestamp, true)
        };
        match self.udp_socket.send_pod_state_message(&pod_state_message) {
            Ok(bytes_sent) => {
                // println!("UDP THREAD: Send {} to Desktop", bytes_sent);
            },
            Err(error) => {
                println!("Error Sending Message on UDP Thread: {:?}", error);
            }
        }
    }
}



impl<State> UdpWorker<State> {
    fn get_udp_receiver_message_or_panic(&self) -> UDPMessage {
        if let Ok(message) = self.udp_message_receiver.recv() {
            message
        } else {
            // This should only happen if the channel closes. Which is a panic situation
            panic!("Error Reading from UDP mpsc channel, Exiting");
        }
    }

    fn notify_recovery(&self) {
        self.tcp_sender.send(TcpMessage::EnteringRecovery).expect("To be able to notify tcp thread that we are entering recovery mode");
    }

    fn invalid_transition_recognized(mut self) -> UdpWorker<Recovery> {
        self.notify_recovery();
        self.errno = UdpErrno::InvalidTransitionRequest;
        self.EnterRecovery()
    }

    fn handle_telemetry_timestamp(&mut self, timestamp: chrono::NaiveDateTime) {
        self.last_received_telemetry_timestamp = timestamp;
    }

    fn trigger_transition_to_new_state(&mut self, requested_state: pod_states::PodState) {
        self.can_message_sender.send(CanMessage::ChangeState(requested_state.clone())).expect("Should be able to Send a message to the Can thread from the UDP thread");
        self.next_pod_state = requested_state;
    }

    pub fn new(
        can_sender: Sender<CanMessage>,
        tcp_sender: Sender<TcpMessage>,
        udp_receiver: Receiver<UDPMessage>,
        udp_max_number_timeouts: u32,
    ) -> UdpWorker<Startup> {
        UdpWorker {
            udp_socket: UdpSocket::bind("0.0.0.0:8080").expect("Unable to Bind to UDP Socket on port 8080"),
            current_pod_state: pod_states::PodState::LowVoltage, // *************  TODO Figure out what the initial Value for this should be
            next_pod_state: pod_states::PodState::LowVoltage, // *************  TODO Figure out what the initial Value for this should be
            errno: UdpErrno::NoError,
            timeout_counter: 0,
            last_received_telemetry_timestamp: chrono::Utc::now().naive_local(),
            current_pod_data: pod_data::PodData::new(),
            current_telemetry_timestamp: chrono::Utc::now().naive_local(),
            tcp_sender: tcp_sender,
            udp_message_receiver: udp_receiver,
            can_message_sender: can_sender,
            udp_max_number_timeouts,
            state: std::marker::PhantomData
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
     * efficient in terms of storage, it will always give us a consistent layout in memory which is important for these operations to work properly
     */
    fn EnterRecovery(self) -> UdpWorker<Recovery> {
        unsafe { std::mem::transmute(self) }
    }
    fn EnterConnected(self) -> UdpWorker<Connected> {
        unsafe { std::mem::transmute(self) }
    }
    fn EnterDisconnected(self) -> UdpWorker<Disconnected> {
        unsafe { std::mem::transmute(self) }
    }
}
pub type UdpWorkerState = WorkerState<UdpWorker<Startup>, UdpWorker<Recovery>, UdpWorker<Connected>, UdpWorker<Disconnected>>;

impl UdpWorkerState {
    pub fn new(
        can_sender: Sender<CanMessage>,
        tcp_sender: Sender<TcpMessage>,
        udp_receiver: Receiver<UDPMessage>,
        udp_max_number_timeouts: u32,
    ) -> UdpWorkerState {
        let worker: UdpWorker<Startup> = UdpWorker::<Startup>::new(can_sender, tcp_sender, udp_receiver, udp_max_number_timeouts);
        UdpWorkerState::Startup(worker)
    }
}

impl MainLoop<UdpWorkerState> for UdpWorker<Startup> {
    fn main_loop(self) ->  UdpWorkerState {
        match self.get_udp_receiver_message_or_panic() {
            UDPMessage::StartupComplete => {
                UdpWorkerState::Disconnected(self.EnterDisconnected())
            },
            message => {
                println!("Received Message on UDP mpsc channel during Startup: {:?}", message);
                UdpWorkerState::Startup(self)
            }
        }
    }
}

impl MainLoop<UdpWorkerState> for UdpWorker<Disconnected> {
    fn main_loop(self) -> UdpWorkerState {
        match self.get_udp_receiver_message_or_panic() {
            UDPMessage::ConnectToHost(addr) => {
                if let Ok(_) = self.udp_socket.connect(addr) {
                    println!("UDP THREAD: Connected to addr: {:?}", addr);
                    return UdpWorkerState::Connected(self.EnterConnected());
                } else {
                    println!("UDP THREAD: Unable to connect to {:?}", addr);
                }
            },
            message => {
                println!("UDP THREAD: Received Message on UDP mpsc channel while Disconnected: {:?}", message);
            }
        }
        UdpWorkerState::Disconnected(self)
    }
}

impl MainLoop<UdpWorkerState> for UdpWorker<Connected> {
    fn main_loop(mut self) -> UdpWorkerState {
        let mut socket_buffer = [0u8; 1024];
        match self.udp_socket.recv(&mut socket_buffer) {
            Ok(_bytes_received) => {
                // When the POD Enters an Error State, we no longer need to follow the decision tree
                // for where or not we can transition to a new state etc. The Only Goal For Error State is
                // to hopefully keep the Rpi connected to the desktop long enough to tell the desktop that
                // A failure was found and that the pod is working to shut down
                // println!("UDP THREAD: {} Bytes Read", bytes_received);
                if !self.current_pod_state.is_error_state() {
                    if let Ok(desktop_state_message) = udp_messages::DesktopStateMessage::from_json_bytes(&socket_buffer) {
                        if desktop_state_message.requested_state == self.current_pod_state {
                            if self.next_pod_state == self.current_pod_state {
                                self.handle_telemetry_timestamp(desktop_state_message.most_recent_timestamp);
                            } else {
                                return UdpWorkerState::Recovery(self.invalid_transition_recognized());
                            }
                        } else {
                            if self.current_pod_state.can_transition_to(&desktop_state_message.requested_state) {
                                if desktop_state_message.requested_state == self.next_pod_state {
                                    self.handle_telemetry_timestamp(desktop_state_message.most_recent_timestamp);
                                } else {
                                    if self.current_pod_state == self.next_pod_state {
                                        self.trigger_transition_to_new_state(desktop_state_message.requested_state);
                                        self.handle_telemetry_timestamp(desktop_state_message.most_recent_timestamp);
                                    } else {
                                        return UdpWorkerState::Recovery(self.invalid_transition_recognized());
                                    }
                                }
                            } else {
                                return UdpWorkerState::Recovery(self.invalid_transition_recognized());
                            }
                        }
                    } else {
                        // TODO: This needs to change once we've figured out how we want to handle an error
                        println!("UDP THREAD: Unable to read: {:?}", std::str::from_utf8(&socket_buffer).expect("To be able to convert to utf"));
                        panic!("UDP THREAD: Failed to Read DesktopStateMessage in UDP Handler while in Connected State");
                    }
                    self.timeout_counter = 0;
                }
            },
            Err(error) => {
                println!("Error: {:?}", error); // TODO: Figure out what error is returned for a timeout so that case can be handled separately
                match error.kind() {
                    std::io::ErrorKind::TimedOut => {
                        self.timeout_counter += 1; // Move this to the timeout  portion of the error handler
                        if self.timeout_counter >= self.udp_max_number_timeouts {
                            // TODO Enter into Recovery. Assume Desktop Disconnected
                            self.notify_recovery();
                            self.errno = UdpErrno::ControllerTimeout;
                            return UdpWorkerState::Recovery(self.EnterRecovery());
                        }
                    },
                    _ => {

                    }
                }

            }
        }
        // Check for new Messages from other threads
        while let Ok(message) = self.udp_message_receiver.try_recv() {
            match message {
                UDPMessage::PodStateChanged(new_state) => {
                    if new_state.is_error_state() {
                        self.errno = UdpErrno::GeneralPodFailure;
                    }
                    self.current_pod_state = new_state;
                },
                UDPMessage::TelemetryDataAvailable(new_data, timestamp) => {
                    self.current_pod_data = new_data;
                    self.current_telemetry_timestamp = timestamp;
                },
                UDPMessage::DisconnectFromHost => {
                    self.send_pod_state_message();
                    return UdpWorkerState::Recovery(self.EnterRecovery());
                }
                unrecognized_message => {
                    panic!("UnExpected Message Received on UDP mpsc channel while in Connected State: {:?}", unrecognized_message);
                }
            }
        }
        return UdpWorkerState::Connected(self);
    }
}

impl MainLoop<UdpWorkerState> for UdpWorker<Recovery> {
    fn main_loop(mut self) -> UdpWorkerState {
        self.send_pod_state_message();
        while let Ok(message) = self.udp_message_receiver.try_recv() {
            match message {
                UDPMessage::PodStateChanged(new_state) => {
                    if new_state.is_error_state() {
                        self.errno = UdpErrno::GeneralPodFailure;
                    }
                    self.current_pod_state = new_state;
                },
                UDPMessage::TelemetryDataAvailable(new_data, timestamp) => {
                    self.current_pod_data = new_data;
                    self.current_telemetry_timestamp = timestamp;
                },
                UDPMessage::DisconnectFromHost => {
                }
                unrecognized_message => {
                    panic!("UnExpected Message Received on UDP mpsc channel while in Connected State: {:?}", unrecognized_message);
                }
            }
        }
        match self.current_pod_state {
            pod_states::PodState::LowVoltage => {
                self.tcp_sender.send(TcpMessage::RecoveryComplete).expect("To be able to send message");
                return UdpWorkerState::Disconnected(self.EnterDisconnected());
            },
            pod_states::PodState::Armed => {
                // transision to lowVoltage
                if self.next_pod_state != pod_states::PodState::LowVoltage {
                    self.trigger_transition_to_new_state(pod_states::PodState::LowVoltage);
                }
            },
            pod_states::PodState::AutoPilot => {
                //transition to braking
                if self.next_pod_state != pod_states::PodState::Braking {
                    self.trigger_transition_to_new_state(pod_states::PodState::Braking);
                }
            },
            pod_states::PodState::Braking => {
                // wait till regression to lv
                if self.next_pod_state != pod_states::PodState::LowVoltage {
                    self.trigger_transition_to_new_state(pod_states::PodState::LowVoltage)
                }
            },
            state => {
                println!("Pod state mising in recovery procedure: {:?}", state);
            }
        }
        UdpWorkerState::Recovery(self)
    }
}
