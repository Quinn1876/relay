use json::{ JsonValue, object, array };
type TorchicData = [Option<f32>; 2];
type PressureData = Option<f32>;

// All Pod data will be optional. None values will be converted to null in the JSON that is sent to the
// Desktop
#[derive(Clone, Copy, Debug)]
pub struct PodData {
    pub speed: Option<i32>,
    pub torchic_1: TorchicData,
    pub torchic_2: TorchicData,
    pub pressure_high: PressureData,
    pub pressure_low_1: PressureData,
    pub pressure_low_2: PressureData,
}

trait JsonHelper {
    fn to_json(&self) -> JsonValue;
}

impl JsonHelper for TorchicData {
    fn to_json(&self) -> JsonValue {
        array![self[0], self[1]]
    }
}


impl PodData {
    pub fn to_json(&self) -> JsonValue {
        object!{
            speed: self.speed,
            torchic_1: self.torchic_1.to_json(),
            torchic_2: self.torchic_2.to_json(),
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
