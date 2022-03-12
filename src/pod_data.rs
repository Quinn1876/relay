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
        }
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
        }
    }
}
