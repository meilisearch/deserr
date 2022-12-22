use deserr::{DeserializeError, ErrorKind, IntoValue, MergeWithError, ValuePointerRef};
use serde_json::json;

pub struct JsonPointer(String);

impl MergeWithError<JsonPointer> for JsonPointer {
    fn merge(
        _self_: Option<Self>,
        other: JsonPointer,
        _merge_location: ValuePointerRef,
    ) -> Result<Self, Self> {
        Err(other)
    }
}

impl DeserializeError for JsonPointer {
    fn error<V: IntoValue>(
        _self_: Option<Self>,
        _error: ErrorKind<V>,
        location: ValuePointerRef,
    ) -> Result<Self, Self> {
        Err(JsonPointer(location.as_json()))
    }
}
#[test]
fn test_pointer_as_json() {
    #[allow(dead_code)]
    #[derive(Debug, deserr::DeserializeFromValue)]
    #[deserr(deny_unknown_fields)]
    struct Test {
        top: usize,
    }

    // error at origin.
    let ret = deserr::deserialize::<Test, _, JsonPointer>(json!({})).unwrap_err();
    insta::assert_display_snapshot!(ret.0, @"");

    // can't deserialize top
    let ret = deserr::deserialize::<Test, _, JsonPointer>(json!({ "top": "hello" })).unwrap_err();
    insta::assert_display_snapshot!(ret.0, @".top");

    // can't deserialize top
    let ret = deserr::deserialize::<Test, _, JsonPointer>(json!({ "top": -2 })).unwrap_err();
    insta::assert_display_snapshot!(ret.0, @".top");

    // unknown field bottom at origin
    let ret =
        deserr::deserialize::<Test, _, JsonPointer>(json!({ "top": 2, "bottom": 3 })).unwrap_err();
    insta::assert_display_snapshot!(ret.0, @"");

    #[allow(dead_code)]
    #[derive(Debug, deserr::DeserializeFromValue)]
    #[deserr(deny_unknown_fields)]
    struct Test2 {
        left: Vec<usize>,
        right: Vec<Test>,
    }

    // left was supposed to be an array
    let ret =
        deserr::deserialize::<Test2, _, JsonPointer>(json!({ "left": 2, "right": [{ "top": 2 }] }))
            .unwrap_err();
    insta::assert_display_snapshot!(ret.0, @".left");

    // can't deserialize the first (0) element of the array
    let ret = deserr::deserialize::<Test2, _, JsonPointer>(
        json!({ "left": [-2], "right": [{ "top": 2 }] }),
    )
    .unwrap_err();
    insta::assert_display_snapshot!(ret.0, @".left[0]");

    // can't deserialize the third element of the array
    let ret = deserr::deserialize::<Test2, _, JsonPointer>(
        json!({ "left": [2, 3, -2], "right": [{ "top": 2 }] }),
    )
    .unwrap_err();
    insta::assert_display_snapshot!(ret.0, @".left[2]");

    // can't deserialize the second element of the array on the top field
    let ret = deserr::deserialize::<Test2, _, JsonPointer>(
        json!({ "left": [], "right": [{ "top": 2 }, { "top": -2 }] }),
    )
    .unwrap_err();
    insta::assert_display_snapshot!(ret.0, @".right[1].top");
}
