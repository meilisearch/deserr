use jayson::{DeserializeError, DeserializeFromValue, MergeWithError};

#[derive(Debug)]
pub struct MyError;
impl DeserializeError for MyError {
    fn location(&self) -> Option<jayson::ValuePointer> {
        todo!()
    }

    fn incorrect_value_kind(
        _self_: Option<Self>,
        _actual: jayson::ValueKind,
        _accepted: &[jayson::ValueKind],
        _location: jayson::ValuePointerRef,
    ) -> Result<Self, Self> {
        todo!()
    }

    fn missing_field(
        _self_: Option<Self>,
        _field: &str,
        _location: jayson::ValuePointerRef,
    ) -> Result<Self, Self> {
        todo!()
    }

    fn unknown_key(
        _self_: Option<Self>,
        _key: &str,
        _accepted: &[&str],
        _location: jayson::ValuePointerRef,
    ) -> Result<Self, Self> {
        todo!()
    }

    fn unexpected(
        _self_: Option<Self>,
        _msg: &str,
        _location: jayson::ValuePointerRef,
    ) -> Result<Self, Self> {
        todo!()
    }
}
impl MergeWithError<MyError> for MyError {
    fn merge(
        _self_: Option<Self>,
        _other: MyError,
        _merge_location: jayson::ValuePointerRef,
    ) -> Result<Self, Self> {
        todo!()
    }
}

#[derive(DeserializeFromValue)]
#[jayson(error = MyError)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[test]
fn main() {
    let result = serde_json::from_str::<serde_json::Value>(r#"{"x": 1, "y": 2, "z": 3}"#).unwrap();
    let _: Point = jayson::deserialize(result).unwrap();
}
