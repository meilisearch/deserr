use jayson::{DeserializeFromValue, Error, IntoValue};

#[derive(DeserializeFromValue)]
#[jayson(error = Error)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[test]
fn main() {
    let result = serde_json::from_str::<serde_json::Value>(r#"{"x": 1, "y": 2, "z": 3}"#).unwrap();
    let _ = Point::deserialize_from_value(result.into_value()).unwrap();
}
