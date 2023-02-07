//! This module implements the error messages of json deserialization errors.
//! We try to generate the best possible human-readable description of the error.
//!
//! We also provides some helpers if you need to reuse some component for your error
//! messages.

use std::{convert::Infallible, fmt::Display, ops::ControlFlow};

use deserr::{ErrorKind, IntoValue, ValueKind, ValuePointerRef};

use crate::{DeserializeError, MergeWithError};

use super::helpers::did_you_mean;

#[derive(Debug, Clone)]
pub struct JsonError(String);

impl Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl JsonError {
    fn new(msg: String) -> Self {
        JsonError(msg)
    }
}

/// Return a description of the given location in a Json, preceded by the given article.
/// e.g. `at .key1[8].key2`. If the location is the origin, the given article will not be
/// included in the description.
pub fn location_json_description(location: ValuePointerRef, article: &str) -> String {
    fn rec(location: ValuePointerRef) -> String {
        match location {
            ValuePointerRef::Origin => String::new(),
            ValuePointerRef::Key { key, prev } => rec(*prev) + "." + key,
            ValuePointerRef::Index { index, prev } => format!("{}[{index}]", rec(*prev)),
        }
    }
    match location {
        ValuePointerRef::Origin => String::new(),
        _ => {
            format!("{article} `{}`", rec(location))
        }
    }
}

/// Return a description of the list of value kinds for a Json payload.
pub fn value_kinds_description_json(kinds: &[ValueKind]) -> String {
    // Rank each value kind so that they can be sorted (and deduplicated)
    // Having a predictable order helps with pattern matching
    fn order(kind: &ValueKind) -> u8 {
        match kind {
            ValueKind::Null => 0,
            ValueKind::Boolean => 1,
            ValueKind::Integer => 2,
            ValueKind::NegativeInteger => 3,
            ValueKind::Float => 4,
            ValueKind::String => 5,
            ValueKind::Sequence => 6,
            ValueKind::Map => 7,
        }
    }
    // Return a description of a single value kind, preceded by an article
    fn single_description(kind: &ValueKind) -> &'static str {
        match kind {
            ValueKind::Null => "null",
            ValueKind::Boolean => "a boolean",
            ValueKind::Integer => "a positive integer",
            ValueKind::NegativeInteger => "a negative integer",
            ValueKind::Float => "a number",
            ValueKind::String => "a string",
            ValueKind::Sequence => "an array",
            ValueKind::Map => "an object",
        }
    }

    fn description_rec(kinds: &[ValueKind], count_items: &mut usize, message: &mut String) {
        let (msg_part, rest): (_, &[ValueKind]) = match kinds {
            [] => (String::new(), &[]),
            [ValueKind::Integer | ValueKind::NegativeInteger, ValueKind::Float, rest @ ..] => {
                ("a number".to_owned(), rest)
            }
            [ValueKind::Integer, ValueKind::NegativeInteger, ValueKind::Float, rest @ ..] => {
                ("a number".to_owned(), rest)
            }
            [ValueKind::Integer, ValueKind::NegativeInteger, rest @ ..] => {
                ("an integer".to_owned(), rest)
            }
            [a] => (single_description(a).to_owned(), &[]),
            [a, rest @ ..] => (single_description(a).to_owned(), rest),
        };

        if rest.is_empty() {
            if *count_items == 0 {
                message.push_str(&msg_part);
            } else if *count_items == 1 {
                message.push_str(&format!(" or {msg_part}"));
            } else {
                message.push_str(&format!(", or {msg_part}"));
            }
        } else {
            if *count_items == 0 {
                message.push_str(&msg_part);
            } else {
                message.push_str(&format!(", {msg_part}"));
            }

            *count_items += 1;
            description_rec(rest, count_items, message);
        }
    }

    let mut kinds = kinds.to_owned();
    kinds.sort_by_key(order);
    kinds.dedup();

    if kinds.is_empty() {
        // Should not happen ideally
        "a different value".to_owned()
    } else {
        let mut message = String::new();
        description_rec(kinds.as_slice(), &mut 0, &mut message);
        message
    }
}

/// Return the JSON string of the value preceded by a description of its kind
pub fn value_description_with_kind_json(v: &serde_json::Value) -> String {
    match v.kind() {
        ValueKind::Null => "null".to_owned(),
        kind => {
            format!(
                "{}: `{}`",
                value_kinds_description_json(&[kind]),
                serde_json::to_string(v).unwrap()
            )
        }
    }
}

impl DeserializeError for JsonError {
    fn error<V: IntoValue>(
        _self_: Option<Self>,
        error: deserr::ErrorKind<V>,
        location: ValuePointerRef,
    ) -> ControlFlow<Self, Self> {
        let mut message = String::new();

        message.push_str(&match error {
            ErrorKind::IncorrectValueKind { actual, accepted } => {
                let expected = value_kinds_description_json(accepted);
                let received = value_description_with_kind_json(&serde_json::Value::from(actual));

                let location = location_json_description(location, " at");

                format!("Invalid value type{location}: expected {expected}, but found {received}")
            }
            ErrorKind::MissingField { field } => {
                let location = location_json_description(location, " inside");
                format!("Missing field `{field}`{location}")
            }
            ErrorKind::UnknownKey { key, accepted } => {
                let location = location_json_description(location, " inside");

                format!(
                    "Unknown field `{}`{location}: {}expected one of {}",
                    key,
                    did_you_mean(key, accepted),
                    accepted
                        .iter()
                        .map(|accepted| format!("`{}`", accepted))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            ErrorKind::UnknownValue { value, accepted } => {
                let location = location_json_description(location, " at");
                format!(
                    "Unknown value `{}`{location}: {}expected one of {}",
                    value,
                    did_you_mean(value, accepted),
                    accepted
                        .iter()
                        .map(|accepted| format!("`{}`", accepted))
                        .collect::<Vec<String>>()
                        .join(", "),
                )
            }
            ErrorKind::Unexpected { msg } => {
                let location = location_json_description(location, " at");
                format!("Invalid value{location}: {msg}")
            }
        });

        ControlFlow::Break(JsonError::new(message))
    }
}

impl MergeWithError<JsonError> for JsonError {
    fn merge(
        _self_: Option<Self>,
        other: JsonError,
        _merge_location: ValuePointerRef,
    ) -> ControlFlow<Self, Self> {
        ControlFlow::Break(other)
    }
}

impl<E: std::error::Error> MergeWithError<E> for JsonError {
    fn merge(
        self_: Option<Self>,
        other: E,
        merge_location: ValuePointerRef,
    ) -> ControlFlow<Self, Self> {
        JsonError::error::<Infallible>(
            self_,
            ErrorKind::Unexpected {
                msg: other.to_string(),
            },
            merge_location,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deserr::ValueKind;
    use serde_json::json;

    #[test]
    fn test_value_kinds_description_json() {
        insta::assert_display_snapshot!(value_kinds_description_json(&[]), @"a different value");

        insta::assert_display_snapshot!(value_kinds_description_json(&[ValueKind::Boolean]), @"a boolean");
        insta::assert_display_snapshot!(value_kinds_description_json(&[ValueKind::Integer]), @"a positive integer");
        insta::assert_display_snapshot!(value_kinds_description_json(&[ValueKind::NegativeInteger]), @"a negative integer");
        insta::assert_display_snapshot!(value_kinds_description_json(&[ValueKind::Integer]), @"a positive integer");
        insta::assert_display_snapshot!(value_kinds_description_json(&[ValueKind::String]), @"a string");
        insta::assert_display_snapshot!(value_kinds_description_json(&[ValueKind::Sequence]), @"an array");
        insta::assert_display_snapshot!(value_kinds_description_json(&[ValueKind::Map]), @"an object");

        insta::assert_display_snapshot!(value_kinds_description_json(&[ValueKind::Integer, ValueKind::Boolean]), @"a boolean or a positive integer");
        insta::assert_display_snapshot!(value_kinds_description_json(&[ValueKind::Null, ValueKind::Integer]), @"null or a positive integer");
        insta::assert_display_snapshot!(value_kinds_description_json(&[ValueKind::Sequence, ValueKind::NegativeInteger]), @"a negative integer or an array");
        insta::assert_display_snapshot!(value_kinds_description_json(&[ValueKind::Integer, ValueKind::Float]), @"a number");
        insta::assert_display_snapshot!(value_kinds_description_json(&[ValueKind::Integer, ValueKind::Float, ValueKind::NegativeInteger]), @"a number");
        insta::assert_display_snapshot!(value_kinds_description_json(&[ValueKind::Integer, ValueKind::Float, ValueKind::NegativeInteger, ValueKind::Null]), @"null or a number");
        insta::assert_display_snapshot!(value_kinds_description_json(&[ValueKind::Boolean, ValueKind::Integer, ValueKind::Float, ValueKind::NegativeInteger, ValueKind::Null]), @"null, a boolean, or a number");
        insta::assert_display_snapshot!(value_kinds_description_json(&[ValueKind::Null, ValueKind::Boolean, ValueKind::Integer, ValueKind::Float, ValueKind::NegativeInteger, ValueKind::Null]), @"null, a boolean, or a number");
    }

    #[test]
    fn error_msg_missing_field() {
        #[allow(dead_code)]
        #[derive(deserr::Deserr, Debug)]
        struct Missing {
            me: usize,
        }
        let value = json!({ "toto": 2 });
        let err = deserr::deserialize::<Missing, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Missing field `me`");
    }

    #[test]
    fn error_msg_incorrect() {
        #[allow(dead_code)]
        #[derive(deserr::Deserr, Debug)]
        struct Incorrect {
            me: usize,
        }
        let value = json!({ "me": [2] });
        let err = deserr::deserialize::<Incorrect, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Invalid value type at `.me`: expected a positive integer, but found an array: `[2]`");

        #[allow(dead_code)]
        #[derive(deserr::Deserr, Debug)]
        enum Variants {
            One,
            Two,
            Three,
        }

        #[allow(dead_code)]
        #[derive(deserr::Deserr, Debug)]
        struct MultiIncorrect {
            me: Variants,
        }
        let value = json!({ "me": "la" });
        let err = deserr::deserialize::<MultiIncorrect, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `la` at `.me`: expected one of `One`, `Two`, `Three`");

        #[allow(dead_code)]
        #[derive(deserr::Deserr, Debug)]
        #[deserr(rename_all = lowercase)]
        enum CamelCaseVariants {
            TheObjectiveCamelIsNOICE,
            Bloup,
        }

        #[allow(dead_code)]
        #[derive(deserr::Deserr, Debug)]
        struct MultiIncorrectWithRename {
            me: CamelCaseVariants,
        }
        let value = json!({ "me": "la" });
        let err = deserr::deserialize::<MultiIncorrectWithRename, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `la` at `.me`: expected one of `theobjectivecamelisnoice`, `bloup`");
    }

    #[test]
    fn error_msg_unknown_key() {
        #[allow(dead_code)]
        #[derive(deserr::Deserr, Debug)]
        #[deserr(deny_unknown_fields)]
        struct SingleUnknownField {
            me: usize,
        }
        let value = json!({ "me": 2, "u": "uwu" });
        let err = deserr::deserialize::<SingleUnknownField, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown field `u`: expected one of `me`");

        #[allow(dead_code)]
        #[derive(deserr::Deserr, Debug)]
        #[deserr(deny_unknown_fields)]
        struct MultiUnknownField {
            me: usize,
            and: String,
        }
        let value = json!({ "me": 2, "and": "u", "uwu": "OwO" });
        let err = deserr::deserialize::<MultiUnknownField, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown field `uwu`: expected one of `me`, `and`");
    }

    #[test]
    fn error_msg_unexpected() {
        #[allow(dead_code)]
        #[derive(deserr::Deserr, Debug)]
        #[deserr(deny_unknown_fields)]
        struct UnexpectedTuple {
            me: (usize, String),
        }
        let value = json!({ "me": [2] });
        let err = deserr::deserialize::<UnexpectedTuple, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Invalid value at `.me`: the sequence should have exactly 2 elements");

        let value = json!({ "me": [2, 3, 4] });
        let err = deserr::deserialize::<UnexpectedTuple, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Invalid value at `.me`: the sequence should have exactly 2 elements");
    }

    #[test]
    fn error_did_you_mean() {
        #[allow(dead_code)]
        #[derive(deserr::Deserr, Debug)]
        #[deserr(deny_unknown_fields, rename_all = camelCase)]
        struct DidYouMean {
            q: Values,
            filter: String,
            sort: String,
            attributes_to_highlight: String,
        }

        #[derive(deserr::Deserr, Debug)]
        #[deserr(rename_all = camelCase)]
        enum Values {
            Q,
            Filter,
            Sort,
            AttributesToHighLight,
        }

        let value = json!({ "filler": "doggo" });
        let err = deserr::deserialize::<DidYouMean, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown field `filler`: did you mean `filter`? expected one of `q`, `filter`, `sort`, `attributesToHighlight`");

        let value = json!({ "sart": "doggo" });
        let err = deserr::deserialize::<DidYouMean, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown field `sart`: did you mean `sort`? expected one of `q`, `filter`, `sort`, `attributesToHighlight`");

        let value = json!({ "attributes_to_highlight": "doggo" });
        let err = deserr::deserialize::<DidYouMean, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown field `attributes_to_highlight`: did you mean `attributesToHighlight`? expected one of `q`, `filter`, `sort`, `attributesToHighlight`");

        let value = json!({ "attributesToHighloght": "doggo" });
        let err = deserr::deserialize::<DidYouMean, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown field `attributesToHighloght`: did you mean `attributesToHighlight`? expected one of `q`, `filter`, `sort`, `attributesToHighlight`");

        // doesn't match anything

        let value = json!({ "a": "doggo" });
        let err = deserr::deserialize::<DidYouMean, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown field `a`: expected one of `q`, `filter`, `sort`, `attributesToHighlight`");

        let value = json!({ "query": "doggo" });
        let err = deserr::deserialize::<DidYouMean, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown field `query`: expected one of `q`, `filter`, `sort`, `attributesToHighlight`");

        let value = json!({ "filterable": "doggo" });
        let err = deserr::deserialize::<DidYouMean, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown field `filterable`: expected one of `q`, `filter`, `sort`, `attributesToHighlight`");

        let value = json!({ "sortable": "doggo" });
        let err = deserr::deserialize::<DidYouMean, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown field `sortable`: expected one of `q`, `filter`, `sort`, `attributesToHighlight`");

        // did you mean triggered by an unknown value

        let value = json!({ "q": "filler" });
        let err = deserr::deserialize::<DidYouMean, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `filler` at `.q`: did you mean `filter`? expected one of `q`, `filter`, `sort`, `attributesToHighLight`");

        let value = json!({ "q": "sart" });
        let err = deserr::deserialize::<DidYouMean, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `sart` at `.q`: did you mean `sort`? expected one of `q`, `filter`, `sort`, `attributesToHighLight`");

        let value = json!({ "q": "attributes_to_highlight" });
        let err = deserr::deserialize::<DidYouMean, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `attributes_to_highlight` at `.q`: expected one of `q`, `filter`, `sort`, `attributesToHighLight`");

        let value = json!({ "q": "attributesToHighloght" });
        let err = deserr::deserialize::<DidYouMean, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `attributesToHighloght` at `.q`: did you mean `attributesToHighLight`? expected one of `q`, `filter`, `sort`, `attributesToHighLight`");

        // doesn't match anything

        let value = json!({ "q": "a" });
        let err = deserr::deserialize::<DidYouMean, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `a` at `.q`: expected one of `q`, `filter`, `sort`, `attributesToHighLight`");

        let value = json!({ "q": "query" });
        let err = deserr::deserialize::<DidYouMean, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `query` at `.q`: expected one of `q`, `filter`, `sort`, `attributesToHighLight`");

        let value = json!({ "q": "filterable" });
        let err = deserr::deserialize::<DidYouMean, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `filterable` at `.q`: expected one of `q`, `filter`, `sort`, `attributesToHighLight`");

        let value = json!({ "q": "sortable" });
        let err = deserr::deserialize::<DidYouMean, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `sortable` at `.q`: expected one of `q`, `filter`, `sort`, `attributesToHighLight`");
    }
}
