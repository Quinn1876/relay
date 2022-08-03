#[allow(unused_doc_comments)]

use std::time::Duration;
use std::{sync::{
    mpsc::{
        channel,
        Receiver,
        Sender
    }
}, convert::TryInto};
use std::fs::OpenOptions;
use std::io::prelude::*;

#[cfg(unix)]
use crate::can_extentions::prelude::*;

use crate::{thread_managers::messages::{
    TcpMessage,
    UDPMessage,
    CanMessage as CANMessage,
    WorkerMessage
}, utils::rpm_integrator, pod_data::{ PodData }};
use json::JsonValue;
use crate::thread_managers;
use crate::error::Error;

use crate::device_watchdog::{
    DeviceWatchdogMapFuncs,
    Device
};

use crate::utils::rpm_integrator::RpmIntegrator;

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

    let (send_data_to_logger, data_logger_receiver) = channel::<PodData>();
    std::thread::spawn(move || {
        let mut out_file = OpenOptions::new()
            .write(true)
            .append(true)
            .open("Logs.txt")
            .unwrap();
        loop {
            if let Ok(data) = data_logger_receiver.recv() {
                out_file.write(&serde_json::to_vec(&data).unwrap()).unwrap();
            }
        }
    });


    // Worker Thread
    // Initialization
    #[cfg(unix)]
    {
        let mut pod_data = crate::pod_data::PodData::new();
        let mut watchdog = crate::device_watchdog::DeviceWatchdogMap::with_all_devices(can_message_sender.clone(), CANMessage::DeviceLost, 400);
        let mut rpm_integrator = RpmIntegrator::default();
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
                                    watchdog.update_device_timestamp(Device::BMS, crate::device_watchdog::get_now());
                                },
                                CanCommand::MotorControllerHealthCheck{ igbt_temp, motor_voltage } => {
                                    pod_data.motor_voltage = Some(motor_voltage);
                                    pod_data.igbt_temp = Some(igbt_temp);
                                    watchdog.update_device_timestamp(Device::MC, crate::device_watchdog::get_now());
                                },
                                CanCommand::BmsData1{ battery_pack_voltage, state_of_charge } => {
                                    pod_data.battery_pack_voltage = Some(battery_pack_voltage);
                                    pod_data.state_of_charge = Some(state_of_charge);
                                    watchdog.update_device_timestamp(Device::BMS, crate::device_watchdog::get_now());
                                },
                                CanCommand::BmsData2{ buck_temperature, bms_current } => {
                                    pod_data.buck_temperature = Some(buck_temperature);
                                    pod_data.bms_current = Some(bms_current);
                                    watchdog.update_device_timestamp(Device::BMS, crate::device_watchdog::get_now());

                                },
                                CanCommand::BmsData3{ link_cap_voltage } => {
                                    pod_data.link_cap_voltage = Some(link_cap_voltage);
                                    watchdog.update_device_timestamp(Device::BMS, crate::device_watchdog::get_now());

                                },
                                CanCommand::MotorControllerData1{ mc_pod_speed, motor_current } => {
                                    pod_data.mc_pod_speed = Some(mc_pod_speed);
                                    pod_data.motor_current = Some(motor_current);
                                    watchdog.update_device_timestamp(Device::MC, crate::device_watchdog::get_now());

                                },
                                CanCommand::MotorControllerData2{ battery_current, battery_voltage } => {
                                    pod_data.battery_current = Some(battery_current);
                                    pod_data.battery_voltage = Some(battery_voltage);
                                    watchdog.update_device_timestamp(Device::MC, crate::device_watchdog::get_now());

                                },
                                CanCommand::PodSpeed{ pod_speed } => {
                                    pod_data.speed = Some(pod_speed);
                                    watchdog.update_device_timestamp(Device::MC, crate::device_watchdog::get_now());

                                },
                                CanCommand::PressureHigh(pressure) => {
                                    pod_data.pressure_high = Some(pressure);
                                    watchdog.update_device_timestamp(Device::PRESSURE_HIGH, crate::device_watchdog::get_now());
                                },
                                CanCommand::PressureLow1(pressure) => {
                                    pod_data.pressure_low_1 = Some(pressure);
                                    watchdog.update_device_timestamp(Device::PRESSURE_LOW_1, crate::device_watchdog::get_now());
                                },
                                CanCommand::PressureLow2(pressure) => {
                                    pod_data.pressure_low_2 = Some(pressure);
                                    watchdog.update_device_timestamp(Device::PRESSURE_LOW_2, crate::device_watchdog::get_now());
                                },
                                CanCommand::Current5V(current) => {
                                    pod_data.current_5v = Some(current);
                                    watchdog.update_device_timestamp(Device::ELEKID, crate::device_watchdog::get_now());
                                },
                                CanCommand::Current12V(current) => {
                                    pod_data.current_12v = Some(current);
                                    watchdog.update_device_timestamp(Device::ELEKID, crate::device_watchdog::get_now());
                                },
                                CanCommand::Current24V(current) => {
                                    pod_data.current_24v = Some(current);
                                    watchdog.update_device_timestamp(Device::ELEKID, crate::device_watchdog::get_now());
                                },
                                CanCommand::Torchic1(data) => {
                                    println!("TORCHIC1 DATA: {:?}", data);
                                    pod_data.torchic_1 = data;
                                    watchdog.update_device_timestamp(Device::TORCHIC_1, crate::device_watchdog::get_now());
                                },
                                CanCommand::Torchic2(data) => {
                                    pod_data.torchic_2 = data;
                                    watchdog.update_device_timestamp(Device::TORCHIC_2, crate::device_watchdog::get_now());
                                },
                                CanCommand::RoboteqBatteryAmpsResult{ motor_number, amps } => {
                                    if motor_number == 1 {
                                        pod_data.roboteq_motor_1_battery_amps = Some(amps);
                                    } else if motor_number == 2 {
                                        pod_data.roboteq_motor_1_battery_amps = Some(amps);
                                    } else {
                                        new_data = false;
                                    }
                                },
                                CanCommand::RoboteqMotorEncoderResult{ motor_number, speed } => {
                                    match motor_number {
                                        1 => { pod_data.roboteq_motor_1_speed = Some(RpmIntegrator::calc_speed(speed)); },
                                        2 => { pod_data.roboteq_motor_2_speed = Some(RpmIntegrator::calc_speed(speed));},
                                        _ => { new_data = false; }
                                    }
                                },
                                CanCommand::RoboteqTemperatureResult{ sub_index, temp } => {
                                    match sub_index {
                                        1 => { pod_data.roboteq_mcu_temp = Some(temp);},
                                        2 => { pod_data.roboteq_sensor_1_temp = Some(temp); },
                                        3 => { pod_data.roboteq_sensor_2_temp = Some(temp); },
                                        _ => { new_data = false; },
                                    }
                                }
                                _ => {
                                    new_data = false;
                                }
                            }
                            let devices = watchdog.check_devices();
                            for device in &devices {
                                println!("DEBUG: WATCHDOG DETECTED DEVICE LOST: {:?}", device);
                            }
                            if new_data {
                                // println!("NEW DATA Parsed: {:?}", pod_data);
                                if pod_data.ok() {
                                    send_data_to_logger.send(pod_data.clone()).unwrap();
                                    udp_message_sender.send(UDPMessage::TelemetryDataAvailable(pod_data, time)).expect("To be able to send telemetry data to udp from worker");
                                } else {
                                    //udp_message_sender.send(UDPMessage::SystemFault).expect("TO BE ABLE TO SEND MESSAGE");
                                }
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

