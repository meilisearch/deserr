use deserr::{deserialize, errors::JsonError, Deserr};
use insta::assert_debug_snapshot;
use serde_json::json;

#[test]
fn from_container_attribute() {
    #[allow(unused)]
    #[derive(Debug, Deserr)]
    #[deserr(from(String) = From::from)]
    enum AsciiString {
        Valid(String),
        Invalid(String),
    }

    impl From<String> for AsciiString {
        fn from(s: String) -> Self {
            if s.chars().any(|c| !c.is_ascii()) {
                Self::Invalid(s)
            } else {
                Self::Valid(s)
            }
        }
    }

    let data = deserialize::<AsciiString, _, JsonError>(json!("doggo")).unwrap();

    assert_debug_snapshot!(data, @r###"
    Valid(
        "doggo",
    )
    "###);

    let data = deserialize::<AsciiString, _, JsonError>(json!("🥺")).unwrap();

    assert_debug_snapshot!(data, @r###"
    Invalid(
        "🥺",
    )
    "###);

    #[allow(unused)]
    #[derive(Debug, Deserr)]
    struct Struct {
        #[deserr(needs_predicate)]
        doggo: AsciiString,
    }

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": "BORK" })).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: Valid(
            "BORK",
        ),
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": "👉 👈"})).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: Invalid(
            "👉 👈",
        ),
    }
    "###);
}

#[test]
fn from_field_attribute() {
    #[allow(unused)]
    #[derive(Debug)]
    enum AsciiString {
        Valid(String),
        Invalid(String),
    }

    impl From<String> for AsciiString {
        fn from(s: String) -> Self {
            if s.chars().any(|c| !c.is_ascii()) {
                Self::Invalid(s)
            } else {
                Self::Valid(s)
            }
        }
    }

    #[allow(unused)]
    #[derive(Debug, Deserr)]
    struct Struct {
        #[deserr(from(String) = From::from)]
        doggo: AsciiString,
    }

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": "BORK" })).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: Valid(
            "BORK",
        ),
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": "👉 👈"})).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: Invalid(
            "👉 👈",
        ),
    }
    "###);
}
