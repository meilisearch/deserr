use std::convert::Infallible;

use deserr::{
    deserialize, serde_json::JsonError, take_result_content, DeserializeError,
    Deserr, ErrorKind, ValuePointerRef,
};
use insta::{assert_debug_snapshot, assert_display_snapshot};
use serde_json::json;

#[test]
fn missing_field_error() {
    #[allow(unused)]
    #[derive(Debug, Deserr)]
    struct Struct {
        doggo: String,
        #[deserr(missing_field_error = custom_function)]
        catto: String,
    }

    fn custom_function<E: DeserializeError>(_field_name: &str, location: ValuePointerRef) -> E {
        take_result_content(E::error::<Infallible>(
            None,
            ErrorKind::Unexpected {
                msg: String::from("I really need the query field, please give it to me uwu"),
            },
            location,
        ))
    }

    let data =
        deserialize::<Struct, _, JsonError>(json!({ "doggo": "bork", "catto": "jorts" })).unwrap();
    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: "bork",
        catto: "jorts",
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({ "catto": "jorts" })).unwrap_err();
    assert_display_snapshot!(data, @"Json deserialize error: missing field `doggo` at ``");

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": "bork" })).unwrap_err();
    assert_display_snapshot!(data, @"I really need the query field, please give it to me uwu at ``.");
}
