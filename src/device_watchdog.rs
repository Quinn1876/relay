/**
 * @brief The device watchdog is responsible for keeping track
 * of the last message received from each board over the can bus.
 * It will be configured with an expected period for the messages
 * and if twice that period is missed then a notification is sent.
 * */


use std::sync::mpsc::Sender;
use std::time::{
  SystemTime,
  UNIX_EPOCH
};
use chrono::NaiveDateTime;

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

pub struct DeviceWatchdog<T> {
  sender: Sender<T>,
  notification: T,
  last_message: Option<NaiveDateTime>, /* If none, then no message has been received and assumed disconnected */
  period
}


impl DeviceWatchdog<T> {
  fn new(sender: Sender<T>, notification: T, period: u128 ) -> DeviceWatchdog<T> {
    DeviceWatchdog {
      sender,
      notification,
      last_message: None,
      period
    }
  }

  pub fn update_last_message(&mut self, timestamp: NaiveDateTime) {
    self.last_message = timestamp;
  }

  pub fn isDeviceFunctioning(&self, now: &NaiveDateTime) -> bool {
    if let Some(last_message) = self.last_message {
      now.signed_duration_since(last_message) > 2 * period
    } else {
      true
    }
  }
}

pub type DeviceWatchdogMap<T> = HashMap<Device, DeviceWatchdog<T>>
pub trait DeviceWatchdogMapFuncs {
  /* Create a map with all devices */
  fn withAllDevices<T>(sender: Sender<T>, notification: T, period: u128)
  -> DeviceWatchdogMap
  where T: Clone;
  /* Checks each of the devices timestamps. Returns a list of fail */
  fn checkTimes(&self) -> Vec<Device>;
}


impl DeviceWatchdogMapFuncs for DeviceWatchdogMap<T> {
  fn withAllDevices<T>(sender: Sender<T>, notification: T, period: u128)
  -> DeviceWatchdogMap
  where T: Clone {
    let map = DeviceWatchdogMap<T>::new();
    map.insert(BMS, DeviceWatchdog<T>::new(sender.clone(), notification.clone(), period));
    map.insert(MC, DeviceWatchdog<T>::new(sender.clone(), notification.clone(), period));
    map.insert(PRESSURE_HIGH, DeviceWatchdog<T>::new(sender.clone(), notification.clone(), period));
    map.insert(PRESSURE_LOW_1, DeviceWatchdog<T>::new(sender.clone(), notification.clone(), period));
    map.insert(PRESSURE_LOW_2, DeviceWatchdog<T>::new(sender.clone(), notification.clone(), period));
    map.insert(ELEKID, DeviceWatchdog<T>::new(sender.clone(), notification.clone(), period));
    map.insert(TORCHIC_1, DeviceWatchdog<T>::new(sender.clone(), notification.clone(), period));
    map.insert(TORCHIC_2, DeviceWatchdog<T>::new(sender.clone(), notification.clone(), period));
    map
  }

  fn checkTimes(&self) -> Vec<Device> {
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap();
    let now = NaiveDateTime::from_timestamp(
        now.as_secs(),
        now.as_nanos() % (1e9 as u128) // Nano seconds since second
    );
    let result_vec: Vec<Device> = Vec::new();
    for (device, watchdog) in &self {
      if !watchdog.isDeviceFunctioning(now) {
        result_vec.push(device);
      }
    }
    result_vec
  }
}


/********************
 *      TESTS
 ********************/
mod test {
  #[test]
  fn isDeviceFunctioning() {
      unimplemented!();
  }

}
