use std::time::Instant;
use std::f64::consts::PI;

pub struct RpmIntegrator {
  last_time_received: Instant,
  running_total_distance: f64,
}

const WHEEL_DIAMETER: f64 = 0.13335; // 0.13335m

impl RpmIntegrator {
  pub fn default() -> RpmIntegrator {
    RpmIntegrator {
      last_time_received: Instant::now(),
      running_total_distance: 0.0,
    }
  }

  pub fn calc_speed(rpm: i32) -> f64 {
    /* (r / m) * 1m / 60s * 1 wheel rpm / 18.5 motor rpm */
    let wheel_rps: f64 = (rpm as f64) / 1110.0;
    /* 1 revolution is equal to pi*d */
    wheel_rps * PI * WHEEL_DIAMETER
  }

  pub fn insert_reading(&mut self, rpm: i32) {
    let mut last_time_received = self.last_time_received;
    if self.running_total_distance == 0.0 {
      last_time_received = Instant::now();
    }
    /* This is a very rough integration technique. TODO: Look into implementing some better approximations */
    self.running_total_distance += RpmIntegrator::calc_speed(rpm) * last_time_received.elapsed().as_secs_f64();
  }

  pub fn running_distance(&self) -> f64 {
    return self.running_total_distance;
  }

  pub fn reset(&mut self) {
    self.running_total_distance = 0.0;
  }
}


