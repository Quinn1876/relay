use json::{ JsonValue, object };
type TorchicData = [Option<f32>; 2];
type PressureData = Option<f32>;
// All Pod data will be optional. These values will be converted to null in the JSON that is sent to the
// Desktop
#[derive(Clone, Copy, Debug)]
pub struct PodData {
    speed: Option<i32>,
    torchic_1: TorchicData,
    torchic_2: TorchicData,
    pressure_high: PressureData,
    pressure_low_1: PressureData,
    pressure_low_2: PressureData,
}

impl PodData {
    pub fn to_json(&self) -> JsonValue {
        object!{
            speed: self.speed,
            torchic_1: self.torchic_1,
            torchic_2: self.torchic_2,
            pressure_high: self.pressure_high,
            pressure_low_1: self.pressure_low_1,
            pressure_low_2: self.pressure_low_2
        }
    }

    pub fn new() -> PodData {
        PodData {
            speed: None,
            torchic_1: [None, None],
            torchic_2: [None, None],
            pressure_high: None,
            pressure_low_1: None,
            pressure_low_2: None,
        }
    }
}
