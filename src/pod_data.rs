use json::{ JsonValue, object };

// All Pod data will be optional. These values will be converted to null in the JSON that is sent to the
// Desktop
#[derive(Clone, Copy, Debug)]
pub struct PodData {
    speed: Option<i32>,
}

impl PodData {
    pub fn to_json(&self) -> JsonValue {
        object!{
            speed: self.speed,
        }
    }

    pub fn new() -> PodData {
        PodData {
            speed: None
        }
    }
}
