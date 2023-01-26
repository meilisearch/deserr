use std::convert::Infallible;

use deserr::{
    deserialize, serde_json::JsonError, take_result_content, DeserializeError,
    DeserializeFromValue, ErrorKind, ValuePointerRef,
};
use insta::{assert_debug_snapshot, assert_display_snapshot};
use serde_json::json;

#[test]
fn default_deny_unknown_fields() {
    #[allow(unused)]
    #[derive(Debug, DeserializeFromValue)]
    #[deserr(deny_unknown_fields)]
    struct Struct {
        word: String,
    }

    let data = deserialize::<Struct, _, JsonError>(json!({ "word": "doggo" })).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        word: "doggo",
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({ "word": "doggo", "turbo": "doggo" }))
        .unwrap_err();

    assert_display_snapshot!(data, @"Json deserialize error: unknown field `turbo`, expected one of `word` at ``.");
}

#[test]
fn custom_deny_unknown_fields() {
    #[allow(unused)]
    #[derive(Debug, DeserializeFromValue)]
    #[deserr(deny_unknown_fields = custom_function)]
    struct Struct {
        word: String,
    }

    fn custom_function<E: DeserializeError>(
        field: &str,
        accepted: &[&str],
        location: ValuePointerRef,
    ) -> E {
        match field {
            "doggo" => take_result_content(E::error::<Infallible>(
                None,
                ErrorKind::Unexpected {
                    msg: format!("The word is doggo, not the opposite"),
                },
                location,
            )),
            _ => take_result_content(E::error::<Infallible>(
                None,
                deserr::ErrorKind::UnknownKey {
                    key: field,
                    accepted,
                },
                location,
            )),
        }
    }

    let data = deserialize::<Struct, _, JsonError>(json!({ "word": "doggo" })).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        word: "doggo",
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({ "word": "doggo", "turbo": "doggo" }))
        .unwrap_err();

    assert_display_snapshot!(data, @"Json deserialize error: unknown field `turbo`, expected one of `word` at ``.");

    let data = deserialize::<Struct, _, JsonError>(json!({ "word": "doggo", "doggo": "word" }))
        .unwrap_err();

    assert_display_snapshot!(data, @"The word is doggo, not the opposite at ``.");
}
