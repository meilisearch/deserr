use jayson::{DeserializeError, DeserializeFromValue};

#[derive(Debug)]
pub struct SimpleError;
impl DeserializeError for SimpleError {
    fn incorrect_value_kind(
        _accepted: &[jayson::ValueKind],
        _location: jayson::ValuePointerRef,
    ) -> Self {
        SimpleError
    }

    fn missing_field(_field: &str, _location: jayson::ValuePointerRef) -> Self {
        SimpleError
    }

    fn unexpected(_msg: &str, _location: jayson::ValuePointerRef) -> Self {
        SimpleError
    }
}

#[derive(DeserializeFromValue)]
#[jayson(error = SimpleError)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[test]
fn main() {
    let result = serde_json::from_str::<serde_json::Value>(r#"{"x": 1, "y": 2, "z": 3}"#).unwrap();
    let _: Point = jayson::deserialize(result).unwrap();
}
