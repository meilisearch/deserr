use deserr::{deserialize, Deserr, JsonError};
use insta::assert_debug_snapshot;
use serde_json::json;

#[test]
fn skip() {
    #[allow(unused)]
    #[derive(Debug, Deserr)]
    struct Struct {
        #[deserr(skip)]
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
        doggo: None,
    }
    "###);
}

#[test]
fn skip_and_deny_unknown_fields() {
    #[allow(unused)]
    #[derive(Debug, Deserr)]
    #[deserr(deny_unknown_fields)]
    struct Struct {
        #[deserr(skip)]
        doggo: Option<String>,
    }

    let data = deserialize::<Struct, _, JsonError>(json!({})).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: None,
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": null })).unwrap_err();

    assert_debug_snapshot!(data, @r###"
    JsonError(
        "Unknown field `doggo`: expected one of ",
    )
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": "bork" })).unwrap_err();

    assert_debug_snapshot!(data, @r###"
    JsonError(
        "Unknown field `doggo`: expected one of ",
    )
    "###);
}

#[test]
fn skip_and_default() {
    #[allow(unused)]
    #[derive(Debug, Deserr)]
    struct Struct {
        #[deserr(skip, default = Some(String::from("bork")))]
        doggo: Option<String>,
    }

    let data = deserialize::<Struct, _, JsonError>(json!({})).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: Some(
            "bork",
        ),
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": null })).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: Some(
            "bork",
        ),
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
fn skip_and_default_and_deny_unknown_fields() {
    #[allow(unused)]
    #[derive(Debug, Deserr)]
    #[deserr(deny_unknown_fields)]
    struct Struct {
        #[deserr(skip, default = Some(String::from("bork")))]
        doggo: Option<String>,
    }

    let data = deserialize::<Struct, _, JsonError>(json!({})).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: Some(
            "bork",
        ),
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": null })).unwrap_err();

    assert_debug_snapshot!(data, @r###"
    JsonError(
        "Unknown field `doggo`: expected one of ",
    )
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": "bork" })).unwrap_err();

    assert_debug_snapshot!(data, @r###"
    JsonError(
        "Unknown field `doggo`: expected one of ",
    )
    "###);
}
