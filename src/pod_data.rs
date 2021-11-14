use json::{ JsonValue, object };

pub struct PodData {

}

impl PodData {
    pub fn to_json(&self) -> JsonValue {
        object!{}
    }
}
