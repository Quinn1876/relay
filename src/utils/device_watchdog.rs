/**
 * @brief The device watchdog is responsible for keeping track
 * of the last message received from each board over the can bus.
 * It will be configured with an expected period for the messages
 * and if twice that period is missed then a notification is sent.
 * */


use std::sync::mpsc::Sender;
use std::convert::TryInto;

use std::time::{
  SystemTime,
  UNIX_EPOCH
};
use chrono::NaiveDateTime;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Device {
  BMS,
  MC,
  PRESSURE_HIGH,
  PRESSURE_LOW_1,
  PRESSURE_LOW_2,
  ELEKID,
  TORCHIC_1,
  TORCHIC_2
}

fn get_now() -> NaiveDateTime {
  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap();
  NaiveDateTime::from_timestamp(
    now.as_secs().try_into().unwrap(),
    (now.as_nanos() % (1e9 as u128)) as u32 // Nano seconds since second
  )
}

pub struct DeviceWatchdog<T> {
  sender: Sender<T>,
  notification: T,
  last_message: Option<NaiveDateTime>, /* If none, then no message has been received and assumed disconnected */
  period: i64 /* Time in millis  */
}


impl<T: Clone + Send> DeviceWatchdog<T> {
  fn new(sender: Sender<T>, notification: T, period: i64 ) -> DeviceWatchdog<T> {
    DeviceWatchdog {
      sender,
      notification,
      last_message: None,
      period
    }
  }

  pub fn update_last_message(&mut self, timestamp: NaiveDateTime) {
    self.last_message = Some(timestamp);
  }

  pub fn is_device_functioning(&self, now: &NaiveDateTime) -> bool {
    if let Some(last_message) = self.last_message {
      if now.signed_duration_since(last_message).num_milliseconds() > 2 * self.period {
        self.notify();
        false
      } else {
        true
      }
    } else {
      /* Vacuously True case. We set this true because the absence of a device on the bus should not impact the functionality of the rest of the pod.
        This allows us to deprecate a device without updating or recompiling the code. On the other hand, adding a new device requires logic to handle that device,
        so recompilation will be required either way.
      */
      true
    }
  }

  fn notify(&self) {
    self.sender.send(self.notification.clone()).unwrap();
  }
}

pub type DeviceWatchdogMap<T> = std::collections::HashMap<Device, DeviceWatchdog<T>>;

trait DeviceWatchdogMapFuncs {
  type Notification: Clone + Send;

  /* Create a map with all devices */
  fn with_all_devices(sender: Sender<Self::Notification>, notification: Self::Notification, period: i64)
  -> DeviceWatchdogMap<Self::Notification>;

  /* Checks each of the devices timestamps. Returns a list of fail */
  fn check_devices(&self) -> Vec<Device>;

  /* Update Device Time */
  fn update_device_timestamp(&mut self, device: Device, timestamp: NaiveDateTime);
}


impl<T: Send + Clone> DeviceWatchdogMapFuncs for DeviceWatchdogMap<T> {
  type Notification = T;
  fn with_all_devices(sender: Sender<T>, notification: T, period: i64)
  -> DeviceWatchdogMap<T> {
    let mut map = DeviceWatchdogMap::<T>::new();
    map.insert(Device::BMS, DeviceWatchdog::<T>::new(sender.clone(), notification.clone(), period));
    map.insert(Device::MC, DeviceWatchdog::<T>::new(sender.clone(), notification.clone(), period));
    map.insert(Device::PRESSURE_HIGH, DeviceWatchdog::<T>::new(sender.clone(), notification.clone(), period));
    map.insert(Device::PRESSURE_LOW_1, DeviceWatchdog::<T>::new(sender.clone(), notification.clone(), period));
    map.insert(Device::PRESSURE_LOW_2, DeviceWatchdog::<T>::new(sender.clone(), notification.clone(), period));
    map.insert(Device::ELEKID, DeviceWatchdog::<T>::new(sender.clone(), notification.clone(), period));
    map.insert(Device::TORCHIC_1, DeviceWatchdog::<T>::new(sender.clone(), notification.clone(), period));
    map.insert(Device::TORCHIC_2, DeviceWatchdog::<T>::new(sender.clone(), notification.clone(), period));
    map
  }

  fn check_devices(&self) -> Vec<Device> {
    let now = get_now();
    let mut result_vec: Vec<Device> = Vec::new();
    for (device, watchdog) in self {
      if !watchdog.is_device_functioning(&now) {
        result_vec.push(device.clone());
      }
    }
    result_vec
  }

  fn update_device_timestamp(&mut self, device: Device, timestamp: NaiveDateTime) {
    self.get_mut(&device).expect("Device Missing from map. Consider checking the with_all_devices function to ensure the device is initialized.").update_last_message(timestamp);
  }
}


/********************
 *      TESTS
 ********************/
mod test {
  use super::*;
  #[test]
  fn is_device_functioning_0() {
    /* Vacuously true case  */
    /* Setup */
    let (sender, receiver) = std::sync::mpsc::channel::<u8>();
    let dut = DeviceWatchdog::new(sender, 1, 200); /* 20 second period to ensure that the timing doesn't interfere with the test */
    /* Test */
    assert!(dut.is_device_functioning(&get_now()));
    assert!(receiver.try_recv().is_err()); /* There should not be a notification here */
  }

  #[test]
  fn is_device_functioning_1() {
    /* Ensure that messages */
    /* Setup */
    let (sender, receiver) = std::sync::mpsc::channel::<u8>();
    let mut dut = DeviceWatchdog::new(sender, 1, 200); /* 20 second period to ensure that the timing doesn't interfere with the test */
    /* Test */
    dut.update_last_message(get_now());
    assert!(dut.is_device_functioning(&get_now()));
    assert!(receiver.try_recv().is_err()); /* There should not be a notification here */
  }

  #[test]
  fn is_device_functioning_2() {
    /* Setup */
    let (sender, receiver) = std::sync::mpsc::channel::<u8>();
    let mut dut = DeviceWatchdog::new(sender, 1, 200); /* 20 second period to ensure that the timing doesn't interfere with the test */
    /* Test */
    dut.update_last_message(get_now());
    std::thread::sleep(std::time::Duration::from_millis(401)); /* Sleep just past hte 2x period time */
    assert!(receiver.try_recv().is_err()); /* There should not be a notification here */
    assert!(!dut.is_device_functioning(&get_now()));
    assert!(receiver.try_recv().is_ok()); /* There should  be a notification here */
  }

  #[test]
  fn check_devices_0() {
    /* Setup */
    let (sender, receiver) = std::sync::mpsc::channel::<u8>();
    let mut device_watchdog_map_dut = DeviceWatchdogMap::with_all_devices(sender, 1, 200);
    device_watchdog_map_dut.update_device_timestamp(Device::BMS, get_now());
    device_watchdog_map_dut.update_device_timestamp(Device::ELEKID, get_now());
    assert_eq!(device_watchdog_map_dut.check_devices().len(), 0);
    assert!(receiver.try_recv().is_err()); /* There should not be a notification here */
  }

  #[test]
  fn check_devices_1() {
    /* Setup */
    let (sender, receiver) = std::sync::mpsc::channel::<u8>();
    let mut device_watchdog_map_dut = DeviceWatchdogMap::with_all_devices(sender, 1, 200);
    device_watchdog_map_dut.update_device_timestamp(Device::BMS, get_now());
    device_watchdog_map_dut.update_device_timestamp(Device::ELEKID, get_now());
    std::thread::sleep(std::time::Duration::from_millis(401)); /* Sleep just past hte 2x period time */
    assert!(receiver.try_recv().is_err()); /* There should not be a notification here */
    assert_eq!(device_watchdog_map_dut.check_devices().len(), 2);
    assert!(receiver.try_recv().is_ok()); /* There should  be a notification here */
    assert!(receiver.try_recv().is_ok()); /* There should  be a notification here */
    assert!(receiver.try_recv().is_err()); /* There should not be a notification here */

  }

}
