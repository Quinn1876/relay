#[allow(unused_doc_comments)]

use std::time::Duration;
use std::sync::{
    mpsc::{
        channel,
        Receiver,
        Sender
    }
};

#[cfg(unix)]
use crate::can_extentions::prelude::*;

use crate::thread_managers::messages::{
    TcpMessage,
    UDPMessage,
    CanMessage as CANMessage,
    WorkerMessage
};
use crate::thread_managers;
use crate::error::Error;

pub fn run_threads<A: std::net::ToSocketAddrs +std::fmt::Debug + Send + 'static>(config: crate::config::Config<A>) -> Result<(), Error> {
    let (udp_message_sender, udp_message_receiver): (Sender<UDPMessage>, Receiver<UDPMessage>) = channel();
    #[allow(unused_variables)] // can_message_receiver is only used in unix, but needs to exist so that other parts of the code can send messages without crashing
    let (can_message_sender, can_message_receiver): (Sender<CANMessage>, Receiver<CANMessage>) = channel();
    #[cfg(unix)] // Worker does not need to be created if running outside of unix
    let (worker_message_sender, worker_message_receiver): (Sender<WorkerMessage>, Receiver<WorkerMessage>) = channel();
    let (tcp_sender, tcp_receiver): (Sender<TcpMessage>, Receiver<TcpMessage>) = channel();

    // Configuration Values
    let tcp_message_buffer_size = 128;
    // TODO - Figure out what these value should be
    let udp_socket_read_timeout = Duration::from_millis(500); // Amount of time the UDP Socket will wait for a message from the Controller
    let udp_max_number_timeouts = 10;
    // End Configuration Values

    // CAN Configuration
    #[cfg(unix)]
    let can_socket_read_timeout = Duration::from_millis(10000); // Amount of time the CAN Socket will wait for a message from the rest of the POD
    // End CAN Configuration

    // Thread Handles
    let tcp_handle = thread_managers::TcpManager::run(
        config.tcp_address,
        udp_message_sender.clone(),
        tcp_receiver,
        tcp_message_buffer_size
    );
    let udp_handle = thread_managers::UdpManager::run(
        can_message_sender.clone(),
        tcp_sender.clone(),
        udp_message_receiver,
        udp_max_number_timeouts,
        udp_socket_read_timeout,
        config.udp_address
    );

    #[cfg(unix)]
    let can_handle = thread_managers::CanManager::run(
        thread_managers::CanWorkerInitializer {
            can_interface: config.can_interface,
            worker_message_sender,
            can_message_receiver,
            can_socket_read_timeout,
            udp_message_sender: udp_message_sender.clone(),
        }
    );

    udp_message_sender.send(UDPMessage::StartupComplete).expect("To be able to complete startup");

    // Worker Thread
    // Initialization
    #[cfg(unix)]
    {
        let mut pod_data = crate::pod_data::PodData::new();
        loop {
            match worker_message_receiver.recv() {
                Ok(message) => {
                    match message {
                        WorkerMessage::CanFrameAndTimeStamp(frame, time) => {
                            // Handle CAN Frame in here
                            let mut new_data = true;
                            match frame.get_command() {
                                CanCommand::BmsHealthCheck{ battery_pack_current, cell_temperature } => {
                                    pod_data.battery_pack_current = Some(battery_pack_current);
                                    pod_data.average_cell_temperature = Some(cell_temperature);
                                },
                                CanCommand::MotorControllerHealthCheck{ igbt_temp, motor_voltage } => {
                                    pod_data.motor_voltage = Some(motor_voltage);
                                    pod_data.igbt_temp = Some(igbt_temp);
                                },
                                CanCommand::BmsData1{ battery_pack_voltage, state_of_charge } => {
                                    pod_data.battery_pack_voltage = Some(battery_pack_voltage);
                                    pod_data.state_of_charge = Some(state_of_charge);
                                },
                                CanCommand::BmsData2{ buck_temperature, bms_current } => {
                                    pod_data.buck_temperature = Some(buck_temperature);
                                    pod_data.bms_current = Some(bms_current);
                                },
                                CanCommand::BmsData3{ link_cap_voltage } => {
                                    pod_data.link_cap_voltage = Some(link_cap_voltage);
                                },
                                CanCommand::MotorControllerData1{ mc_pod_speed, motor_current } => {
                                    pod_data.mc_pod_speed = Some(mc_pod_speed);
                                    pod_data.motor_current = Some(motor_current);
                                },
                                CanCommand::MotorControllerData2{ battery_current, battery_voltage } => {
                                    pod_data.battery_current = Some(battery_current);
                                    pod_data.battery_voltage = Some(battery_voltage);
                                },
                                CanCommand::PodSpeed{ pod_speed } => {
                                    pod_data.speed = Some(pod_speed);
                                },
                                CanCommand::PressureHigh(pressure) => pod_data.pressure_high = Some(pressure),
                                CanCommand::PressureLow1(pressure) => pod_data.pressure_low_1 = Some(pressure),
                                CanCommand::PressureLow2(pressure) => pod_data.pressure_low_2 = Some(pressure),
                                CanCommand::Current5V(current) => pod_data.current_5v = Some(current),
                                CanCommand::Current12V(current) => pod_data.current_12v = Some(current),
                                CanCommand::Current24V(current) => pod_data.current_24v = Some(current),
                                CanCommand::Torchic1(data) => pod_data.torchic_1 = data,
                                CanCommand::Torchic2(data) => pod_data.torchic_2 = data,
                                _ => {
                                    new_data = false;
                                }
                            }
                            if new_data {
                                // println!("NEW DATA Parsed: {:?}", pod_data);
                                udp_message_sender.send(UDPMessage::TelemetryDataAvailable(pod_data, time)).expect("To be able to send telemetry data to udp from worker");
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
    #[cfg(unix)]
    can_handle.join().expect("Should be able to join at the end of the Program");

    Ok(())
}

