use json::{ JsonValue, object, array }; // TODO Reimplement with serde json
type Float2 = [Option<f32>; 2];
type Float1 = Option<f32>;

// All Pod data will be optional. None values will be converted to null in the JSON that is sent to the
// Desktop
#[derive(Clone, Copy, Debug)]
pub struct PodData {
    pub battery_pack_current: Float1,
    pub average_cell_temperature: Float1,
    pub igbt_temp: Float1,
    pub motor_voltage: Float1,
    pub battery_pack_voltage: Float1,
    pub state_of_charge: Float1,
    pub buck_temperature: Float1,
    pub bms_current: Float1,
    pub link_cap_voltage: Float1,
    pub mc_pod_speed: Float1,
    pub motor_current: Float1,
    pub battery_current: Float1,
    pub battery_voltage: Float1,
    pub speed: Float1,
    pub current_5v: Float1,
    pub current_12v: Float1,
    pub current_24v: Float1,
    pub torchic_1: Float2,
    pub torchic_2: Float2,
    pub pressure_high: Float1,
    pub pressure_low_1: Float1,
    pub pressure_low_2: Float1,
    pub roboteq_motor_1_speed: Option<i32>,
    pub roboteq_motor_2_speed: Option<i32>,
    pub roboteq_motor_1_battery_amps: Option<i16>,
    pub roboteq_motor_2_battery_amps: Option<i16>,
    pub roboteq_mcu_temp: Option<i8>,
    pub roboteq_sensor_1_temp: Option<i8>,
    pub roboteq_sensor_2_temp: Option<i8>,


}

trait JsonHelper {
    fn to_json(&self) -> JsonValue;
}

impl JsonHelper for Float2 {
    fn to_json(&self) -> JsonValue {
        array![self[0], self[1]]
    }
}

impl Into<JsonValue> for PodData {
    fn into(self) -> JsonValue {
        object!{
            battery_pack_current: self.battery_pack_current,
            average_cell_temperature: self.average_cell_temperature,
            igbt_temp: self.igbt_temp,
            motor_voltage: self.motor_voltage,
            battery_pack_voltage: self.battery_pack_voltage,
            state_of_charge: self.state_of_charge,
            buck_temperature: self.buck_temperature,
            bms_current: self.bms_current,
            link_cap_voltage: self.link_cap_voltage,
            mc_pod_speed: self.mc_pod_speed,
            motor_current: self.motor_current,
            battery_current: self.battery_current,
            battery_voltage: self.battery_voltage,
            speed: self.speed,
            current_5v: self.current_5v,
            current_12v: self.current_12v,
            current_24v: self.current_24v,
            torchic_1: self.torchic_1.to_json(),
            torchic_2: self.torchic_2.to_json(),
            pressure_high: self.pressure_high,
            pressure_low_1: self.pressure_low_1,
            pressure_low_2: self.pressure_low_2,
            roboteq_motor_1_speed: self.roboteq_motor_1_speed,
            roboteq_motor_2_speed: self.roboteq_motor_2_speed,
            roboteq_motor_1_battery_amps: self.roboteq_motor_1_battery_amps,
            roboteq_motor_2_battery_amps: self.roboteq_motor_2_battery_amps,
            roboteq_mcu_temp: self.roboteq_mcu_temp,
            roboteq_sensor_1_temp: self.roboteq_sensor_1_temp,
            roboteq_sensor_2_temp: self.roboteq_sensor_2_temp,
        }
    }
}

impl From<JsonValue> for PodData {
    fn from(jv: JsonValue) -> PodData {
        let mut pod_data = PodData::new();
        for (key, value) in jv.entries() {
            match key {
                "battery_pack_current" => {
                    pod_data.battery_pack_current = value.as_f32();
                },
                "average_cell_temperature" => {
                    pod_data.average_cell_temperature = value.as_f32();
                },
                "igbt_temp" => {
                    pod_data.igbt_temp = value.as_f32();
                },
                "motor_voltage" => {
                    pod_data.motor_voltage = value.as_f32();
                },
                "battery_pack_voltage" => {
                    pod_data.battery_pack_voltage = value.as_f32();
                },
                "state_of_charge" => {
                    pod_data.state_of_charge = value.as_f32();
                },
                "buck_temperature" => {
                    pod_data.buck_temperature = value.as_f32();
                },
                "bms_current" => {
                    pod_data.bms_current = value.as_f32();
                },
                "link_cap_voltage" => {
                    pod_data.link_cap_voltage = value.as_f32();
                },
                "mc_pod_speed" => {
                    pod_data.mc_pod_speed = value.as_f32();
                },
                "motor_current" => {
                    pod_data.motor_current = value.as_f32();
                },
                "battery_current" => {
                    pod_data.battery_current = value.as_f32();
                },
                "battery_voltage" => {
                    pod_data.battery_voltage = value.as_f32();
                },
                "speed" => {
                    pod_data.speed = value.as_f32();
                },
                "current_5v" => {
                    pod_data.current_5v = value.as_f32();
                },
                "current_12v" => {
                    pod_data.current_12v = value.as_f32();
                },
                "current_24v" => {
                    pod_data.current_24v = value.as_f32();
                },
                "torchic_1" => {
                    let v: Vec<Option<f32>> = value.members().map(|x| x.as_f32()).collect();
                    if v.len() != 2 { continue }
                    pod_data.torchic_1 = [*v.get(0).unwrap(), *v.get(1).unwrap()];
                },
                "torchic_2" => {
                    let v: Vec<Option<f32>> = value.members().map(|x| x.as_f32()).collect();
                    if v.len() != 2 { continue }
                    pod_data.torchic_2 = [*v.get(0).unwrap(), *v.get(1).unwrap()];
                },
                "pressure_high" => {
                    pod_data.pressure_high = value.as_f32();
                },
                "pressure_low_1" => {
                    pod_data.pressure_low_1 = value.as_f32();
                },
                "pressure_low_2" => {
                    pod_data.pressure_low_2 = value.as_f32();
                },
                _ => {}
            }
        }
        pod_data
    }
}

impl PodData {
    pub fn new() -> PodData {
        PodData {
            battery_pack_current: None,
            average_cell_temperature: None,
            igbt_temp: None,
            motor_voltage: None,
            battery_pack_voltage: None,
            state_of_charge: None,
            buck_temperature: None,
            bms_current: None,
            link_cap_voltage: None,
            mc_pod_speed: None,
            motor_current: None,
            battery_current: None,
            battery_voltage: None,
            speed: None,
            current_5v: None,
            current_12v: None,
            current_24v: None,
            torchic_1: [None, None],
            torchic_2: [None, None],
            pressure_high: None,
            pressure_low_1: None,
            pressure_low_2: None,
            roboteq_motor_1_speed: None,
            roboteq_motor_2_speed: None,
            roboteq_motor_1_battery_amps: None,
            roboteq_motor_2_battery_amps: None,
            roboteq_mcu_temp: None,
            roboteq_sensor_1_temp: None,
            roboteq_sensor_2_temp: None,
        }
    }

    /**
     * @brief ok()
     * Check if the board data is okay
     * !! Important. This function uses unsafe code to unwrap Options. This is okay so long as the none check short circuits the execution path when there is no data
     * !! IT IS UNDEFINED BEHAVIOUR TO NOT CHECK NONE BEFORE Calling the unsafe blocks
     */
    pub fn ok(&self) -> bool {
        (self.battery_pack_current.is_none() || unsafe { self.battery_pack_current.unwrap_unchecked() < 50.0})
    &&  (self.average_cell_temperature.is_none() || unsafe { self.average_cell_temperature.unwrap_unchecked() < 45.0 && self.average_cell_temperature.unwrap_unchecked() > 10.0 })
    &&  (self.igbt_temp.is_none() || unsafe { self.igbt_temp.unwrap_unchecked() < 125.0 && self.igbt_temp.unwrap_unchecked() > -40.0 })
    &&  (self.motor_voltage.is_none() || unsafe { self.motor_voltage.unwrap_unchecked() < 37.0 && self.motor_voltage.unwrap_unchecked() > 28.0 })
    &&  (self.battery_pack_voltage.is_none() || unsafe { self.battery_pack_voltage.unwrap_unchecked() > 43.0 })
    &&  (self.state_of_charge.is_none() || unsafe { self.state_of_charge.unwrap_unchecked() > 10.0 })
    &&  (self.buck_temperature.is_none() || true) // We will be using an off the shelf buck because Elekid does not provide enough current. It will monitor the temp itself.__rust_force_expr!
    &&  (self.bms_current.is_none() || unsafe {self.bms_current.unwrap_unchecked() < 0.05 }) // 50 miliamps
    &&  (self.link_cap_voltage.is_none()) // !! NO MC RIGHT NOW!!!
    &&  (self.mc_pod_speed.is_none()) // !! NO MC RIGHT NOW !!
    &&  (self.motor_current.is_none()) // !! NO MC RIGHT NOW
    &&  (self.battery_current.is_none()) // !! NO MC RIGHT NOW
    &&  (self.battery_voltage.is_none()) // !! NO MC RIGHT NOW
    &&  (self.speed.is_none() || unsafe { self.speed.unwrap_unchecked() >= -1.0 && self.speed.unwrap_unchecked() < 44.0})
    &&  (self.current_5v.is_none()) // OFF THE SHELF BUCK. IF WE NEED TO BE CHECKING THIS, IT WILL BE UPDATED
    &&  (self.current_12v.is_none()) // OFF THE SHELF BUCK. IF WE NEED TO BE CHECKING THIS, IT WILL BE UPDATED
    &&  (self.current_24v.is_none()) // OFF THE SHELF BUCK. IF WE NEED TO BE CHECKING THIS, IT WILL BE UPDATED
    &&  (self.torchic_1[0].is_none() || unsafe { self.torchic_1[0].unwrap_unchecked() < 100.0})
    &&  (self.torchic_1[1].is_none() || unsafe { self.torchic_1[1].unwrap_unchecked() < 100.0})
    &&  (self.torchic_2[0].is_none() || unsafe { self.torchic_2[0].unwrap_unchecked() < 100.0})
    &&  (self.torchic_2[1].is_none() || unsafe { self.torchic_2[1].unwrap_unchecked() < 100.0})
    &&  (self.pressure_high.is_none() || unsafe { self.pressure_high.unwrap_unchecked() < 400.0 })
    &&  (self.pressure_low_1.is_none() || unsafe { self.pressure_low_1.unwrap_unchecked() < 100.0 })
    &&  (self.pressure_low_2.is_none() || unsafe { self.pressure_low_2.unwrap_unchecked() < 100.0 })
    }
}
