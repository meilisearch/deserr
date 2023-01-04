use std::collections::{BTreeMap, HashMap};

use deserr::{
    serde_json::JsonError, DeserializeError, ErrorKind, IntoValue, MergeWithError, ValuePointerRef,
};
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

impl From<std::convert::Infallible> for JsonPointer {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
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

#[test]
fn test_default_error_message() {
    #[allow(dead_code)]
    #[derive(Debug, deserr::DeserializeFromValue, serde::Deserialize)]
    #[deserr(deny_unknown_fields)]
    #[serde(deny_unknown_fields)]
    struct Test {
        top: usize,
        right: String,
        left: (isize, isize),
        bottom: BTreeMap<char, usize>,
    }

    // this should deserialize correctly
    let ret = deserr::deserialize::<Test, _, JsonError>(
        json!({ "top": 2, "right": "42", "left": [3, 3], "bottom": { "a": 4, "b": 5 } }),
    )
    .unwrap();
    insta::assert_debug_snapshot!(ret, @r###"
    Test {
        top: 2,
        right: "42",
        left: (
            3,
            3,
        ),
        bottom: {
            'a': 4,
            'b': 5,
        },
    }
    "###);

    // can't deserialize a negative integer into an usize
    let value = json!({ "top": -2, "right": "42", "left": [3, 3], "bottom": { "a": 4, "b": 5 } });
    let deser = deserr::deserialize::<Test, _, JsonError>(value.clone()).unwrap_err();
    let serde = serde_json::from_value::<Test>(value).unwrap_err();
    insta::assert_display_snapshot!(deser, @"invalid value: `-2` expected `usize` at `.top`.");
    insta::assert_display_snapshot!(serde, @"invalid value: integer `-2`, expected usize");

    // can't deserialize an integer into a string
    let value = json!({ "top": 2, "right": 42, "left": [3, 3], "bottom": { "a": 4, "b": 5 } });
    let deser = deserr::deserialize::<Test, _, JsonError>(value.clone()).unwrap_err();
    let serde = serde_json::from_value::<Test>(value).unwrap_err();
    insta::assert_display_snapshot!(deser, @"invalid type: Integer `42`, expected a String at `.right`.");
    insta::assert_display_snapshot!(serde, @"invalid type: integer `42`, expected a string");

    // can't deserialize an integer into an array
    let value = json!({ "top": 2, "right": "hello", "left": 2, "bottom": { "a": 4, "b": 5 } });
    let deser = deserr::deserialize::<Test, _, JsonError>(value.clone()).unwrap_err();
    let serde = serde_json::from_value::<Test>(value).unwrap_err();
    insta::assert_display_snapshot!(deser, @"invalid type: Integer `2`, expected a Sequence at `.left`.");
    insta::assert_display_snapshot!(serde, @"invalid type: integer `2`, expected a tuple of size 2");

    // array of wrong length
    let value = json!({ "top": 2, "right": "hello", "left": [2], "bottom": { "a": 4, "b": 5 } });
    let deser = deserr::deserialize::<Test, _, JsonError>(value.clone()).unwrap_err();
    let serde = serde_json::from_value::<Test>(value).unwrap_err();
    insta::assert_display_snapshot!(deser, @"the sequence should have exactly 2 elements at `.left`.");
    insta::assert_display_snapshot!(serde, @"invalid length 1, expected a tuple of size 2");

    // array of wrong length
    let value =
        json!({ "top": 2, "right": "hello", "left": [2, 3, 4], "bottom": { "a": 4, "b": 5 } });
    let deser = deserr::deserialize::<Test, _, JsonError>(value.clone()).unwrap_err();
    let serde = serde_json::from_value::<Test>(value).unwrap_err();
    insta::assert_display_snapshot!(deser, @"the sequence should have exactly 2 elements at `.left`.");
    insta::assert_display_snapshot!(serde, @"invalid length 3, expected fewer elements in array");

    // string instead of object
    let value = json!({ "top": 2, "right": "hello", "left": [2, 3], "bottom": "hello" });
    let deser = deserr::deserialize::<Test, _, JsonError>(value.clone()).unwrap_err();
    let serde = serde_json::from_value::<Test>(value).unwrap_err();
    insta::assert_display_snapshot!(deser, @r###"invalid type: String `"hello"`, expected a Map at `.bottom`."###);
    insta::assert_display_snapshot!(serde, @r###"invalid type: string "hello", expected a map"###);

    // string instead of char IN the object
    let value = json!({ "top": 2, "right": "hello", "left": [2, 3], "bottom": { "a": "hello" }});
    let deser = deserr::deserialize::<Test, _, JsonError>(value.clone()).unwrap_err();
    let serde = serde_json::from_value::<Test>(value).unwrap_err();
    insta::assert_display_snapshot!(deser, @r###"invalid type: String `"hello"`, expected a Integer at `.bottom.a`."###);
    insta::assert_display_snapshot!(serde, @r###"invalid type: string "hello", expected usize"###);
}
