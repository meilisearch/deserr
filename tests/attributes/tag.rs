use deserr::{deserialize, errors::JsonError, Deserr};
use insta::{assert_debug_snapshot, assert_display_snapshot};
use serde_json::json;

#[test]
fn tagged_enum() {
    #[allow(unused)]
    #[derive(Debug, Deserr)]
    struct Struct {
        either: Either,
    }

    #[allow(unused)]
    #[derive(Debug, Deserr)]
    #[deserr(tag = "type")]
    enum Either {
        Left { doggo: String },
        Right { doggo: bool, catto: String },
    }

    let data = deserialize::<Struct, _, JsonError>(
        json!({ "either": { "type": "Left", "doggo": "bork" } }),
    )
    .unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        either: Left {
            doggo: "bork",
        },
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(
        json!({ "either": { "type": "Right", "doggo": false, "catto": "jorts" } }),
    )
    .unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        either: Right {
            doggo: false,
            catto: "jorts",
        },
    }
    "###);

    let data =
        deserialize::<Struct, _, JsonError>(json!({ "either": { "doggo": "bork" } })).unwrap_err();

    assert_display_snapshot!(data, @"Missing field `type` inside `.either`");

    let data = deserialize::<Struct, _, JsonError>(
        json!({ "either": { "doggo": false, "catto": "jorts" } }),
    )
    .unwrap_err();

    assert_display_snapshot!(data, @"Missing field `type` inside `.either`");
}
#[test]
fn tagged_enum_plus_rename() {
    #[allow(unused)]
    #[derive(Debug, Deserr)]
    struct Struct {
        either: Either,
    }

    #[allow(unused)]
    #[derive(Debug, Deserr)]
    #[deserr(tag = "type", rename_all = lowercase)]
    enum Either {
        Left {
            doggo: String,
        },
        #[deserr(rename = "RIGHT")]
        Right {
            doggo: bool,
            catto: String,
        },
    }

    let data = deserialize::<Struct, _, JsonError>(
        json!({ "either": { "type": "left", "doggo": "bork" } }),
    )
    .unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        either: Left {
            doggo: "bork",
        },
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(
        json!({ "either": { "type": "RIGHT", "doggo": false, "catto": "jorts" } }),
    )
    .unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        either: Right {
            doggo: false,
            catto: "jorts",
        },
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(
        json!({ "either": { "type": "Left", "doggo": "bork" } }),
    )
    .unwrap_err();

    assert_debug_snapshot!(data, @r###"
    JsonError(
        "Invalid value at `.either`: Incorrect tag value",
    )
    "###);

    let data = deserialize::<Struct, _, JsonError>(
        json!({ "either": { "type": "Right", "doggo": false, "catto": "jorts" } }),
    )
    .unwrap_err();

    assert_debug_snapshot!(data, @r###"
    JsonError(
        "Invalid value at `.either`: Incorrect tag value",
    )
    "###);
}
