use deserr::{deserialize, Deserr, JsonError};
use insta::{assert_debug_snapshot, assert_display_snapshot};
use serde_json::json;

#[test]
fn option_dont_use_default_by_default() {
    #[allow(unused)]
    #[derive(Debug, Deserr)]
    struct Struct {
        doggo: Option<String>,
    }

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": null })).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: None,
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": "bork" })).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: Some(
            "bork",
        ),
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({})).unwrap_err();

    assert_display_snapshot!(data, @"Json deserialize error: missing field `doggo` at ``");
}

#[test]
fn default_without_parameter() {
    #[allow(unused)]
    #[derive(Debug, Deserr)]
    struct Struct {
        #[deserr(default)]
        doggo: Option<String>,
    }

    let data = deserialize::<Struct, _, JsonError>(json!({})).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: None,
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": null })).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: None,
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": "bork" })).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: Some(
            "bork",
        ),
    }
    "###);
}

#[test]
fn default_with_a_parameter() {
    #[allow(unused)]
    #[derive(Debug, Deserr)]
    struct Struct {
        #[deserr(default = Some(String::from("BORK")))]
        doggo: Option<String>,
    }

    let data = deserialize::<Struct, _, JsonError>(json!({})).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: Some(
            "BORK",
        ),
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": null })).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: None,
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": "bork" })).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: Some(
            "bork",
        ),
    }
    "###);
}
