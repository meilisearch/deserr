use std::{
    convert::Infallible,
    fmt::{self, Display},
    ops::ControlFlow,
    str::FromStr,
};

use deserr::{
    deserialize, take_cf_content, DeserializeError, Deserr, ErrorKind, JsonError, MergeWithError,
    ValuePointerRef,
};
use insta::{assert_debug_snapshot, assert_display_snapshot};
use serde_json::json;

// For the next tests we're going to deserialize a string that can't contains any non-ascii char
// Since we need a custom error type to accumulate onto in both function it's declared here.

struct AsciiStringError(char);

impl Display for AsciiStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Encountered invalid character: `{}`, only ascii characters are accepted",
            self.0
        )
    }
}

impl MergeWithError<AsciiStringError> for JsonError {
    fn merge(
        _self_: Option<Self>,
        other: AsciiStringError,
        merge_location: ValuePointerRef,
    ) -> ControlFlow<Self, Self> {
        ControlFlow::Break(take_cf_content(JsonError::error::<Infallible>(
            None,
            ErrorKind::Unexpected {
                msg: other.to_string(),
            },
            merge_location,
        )))
    }
}

#[test]
fn from_container_attribute() {
    #[allow(unused)]
    #[derive(Debug, Deserr)]
    #[deserr(try_from(&String) = FromStr::from_str -> AsciiStringError)]
    struct AsciiString(String);

    impl FromStr for AsciiString {
        type Err = AsciiStringError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if let Some(c) = s.chars().find(|c| !c.is_ascii()) {
                Err(AsciiStringError(c))
            } else {
                Ok(Self(s.to_string()))
            }
        }
    }

    let data = deserialize::<AsciiString, _, JsonError>(json!("doggo")).unwrap();

    assert_debug_snapshot!(data, @r###"
    AsciiString(
        "doggo",
    )
    "###);

    let data = deserialize::<AsciiString, _, JsonError>(json!("🥺")).unwrap_err();

    assert_display_snapshot!(data, @"Invalid value: Encountered invalid character: `🥺`, only ascii characters are accepted");

    #[allow(unused)]
    #[derive(Debug, Deserr)]
    struct Struct {
        #[deserr(needs_predicate)]
        doggo: AsciiString,
    }

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": "BORK" })).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: AsciiString(
            "BORK",
        ),
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": "👉 👈"})).unwrap_err();

    assert_display_snapshot!(data, @"Invalid value at `.doggo`: Encountered invalid character: `👉`, only ascii characters are accepted");
}

#[test]
fn from_field_attribute() {
    #[allow(unused)]
    #[derive(Debug)]
    struct AsciiString(String);

    impl FromStr for AsciiString {
        type Err = AsciiStringError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if let Some(c) = s.chars().find(|c| !c.is_ascii()) {
                Err(AsciiStringError(c))
            } else {
                Ok(Self(s.to_string()))
            }
        }
    }

    #[allow(unused)]
    #[derive(Debug, Deserr)]
    struct Struct {
        #[deserr(try_from(&String) = FromStr::from_str -> AsciiStringError)]
        doggo: AsciiString,
    }

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": "BORK" })).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: AsciiString(
            "BORK",
        ),
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": "👉 👈"})).unwrap_err();

    assert_display_snapshot!(data, @"Invalid value at `.doggo`: Encountered invalid character: `👉`, only ascii characters are accepted");
}
