//! This module implements the error messages of query parameters deserialization errors.
//! We try to generate the best possible human-readable description of the error.
//!
//! We also provides some helpers if you need to reuse some component for your error
//! messages.

use crate::{DeserializeError, MergeWithError};
use deserr::{ErrorKind, IntoValue, ValueKind, ValuePointerRef};
use std::{convert::Infallible, fmt::Display, ops::ControlFlow};

use super::helpers::did_you_mean;

#[derive(Debug, Clone)]
pub struct QueryParamError(String);

impl Display for QueryParamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl QueryParamError {
    fn new(msg: String) -> Self {
        QueryParamError(msg)
    }
}

impl deserr::DeserializeError for QueryParamError {
    fn error<V: IntoValue>(
        _self_: Option<Self>,
        error: deserr::ErrorKind<V>,
        location: ValuePointerRef,
    ) -> ControlFlow<Self, Self> {
        let mut message = String::new();

        message.push_str(&match error {
            ErrorKind::IncorrectValueKind { actual, accepted } => {
                let expected = value_kinds_description_query_param(accepted);
                let received = value_description_with_kind_query_param(actual);

                let location = location_query_param_description(location, " for parameter");

                format!("Invalid value type{location}: expected {expected}, but found {received}")
            }
            ErrorKind::MissingField { field } => {
                let location = location_query_param_description(location, " inside");
                format!("Missing parameter `{field}`{location}")
            }
            ErrorKind::UnknownKey { key, accepted } => {
                let location = location_query_param_description(location, " inside");
                format!(
                    "Unknown parameter `{}`{location}: {}expected one of {}",
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
                let location = location_query_param_description(location, " for parameter");
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
                let location = location_query_param_description(location, " in parameter");
                format!("Invalid value{location}: {msg}")
            }
        });

        ControlFlow::Break(QueryParamError::new(message))
    }
}

/// Return a description of the list of value kinds for query parameters
/// Since query parameters are always treated as strings, we always return
/// "a string" for now.
pub fn value_kinds_description_query_param(_accepted: &[ValueKind]) -> String {
    "a string".to_owned()
}

fn value_description_with_kind_query_param<V: IntoValue>(actual: deserr::Value<V>) -> String {
    match actual {
        deserr::Value::Null => "null".to_owned(),
        deserr::Value::Boolean(x) => format!("a boolean: `{x}`"),
        deserr::Value::Integer(x) => format!("an integer: `{x}`"),
        deserr::Value::NegativeInteger(x) => {
            format!("an integer: `{x}`")
        }
        deserr::Value::Float(x) => {
            format!("a number: `{x}`")
        }
        deserr::Value::String(x) => {
            format!("a string: `{x}`")
        }
        deserr::Value::Sequence(_) => "multiple values".to_owned(),
        deserr::Value::Map(_) => "multiple parameters".to_owned(),
    }
}

/// Return a description of the given location in query parameters, preceded by the
/// given article. e.g. `at key5[2]`. If the location is the origin, the given article
/// will not be included in the description.
pub fn location_query_param_description(location: ValuePointerRef, article: &str) -> String {
    fn rec(location: ValuePointerRef) -> String {
        match location {
            ValuePointerRef::Origin => String::new(),
            ValuePointerRef::Key { key, prev } => {
                if matches!(prev, ValuePointerRef::Origin) {
                    key.to_owned()
                } else {
                    rec(*prev) + "." + key
                }
            }
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

impl MergeWithError<QueryParamError> for QueryParamError {
    fn merge(
        _self_: Option<Self>,
        other: QueryParamError,
        _merge_location: ValuePointerRef,
    ) -> ControlFlow<Self, Self> {
        ControlFlow::Break(other)
    }
}

impl<E: std::error::Error> MergeWithError<E> for QueryParamError {
    fn merge(
        self_: Option<Self>,
        other: E,
        merge_location: ValuePointerRef,
    ) -> ControlFlow<Self, Self> {
        QueryParamError::error::<Infallible>(
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
    fn test_value_kinds_description_query_param() {
        insta::assert_display_snapshot!(value_kinds_description_query_param(&[]), @"a string");

        insta::assert_display_snapshot!(value_kinds_description_query_param(&[ValueKind::Boolean]), @"a string");
        insta::assert_display_snapshot!(value_kinds_description_query_param(&[ValueKind::Integer]), @"a string");
        insta::assert_display_snapshot!(value_kinds_description_query_param(&[ValueKind::NegativeInteger]), @"a string");
        insta::assert_display_snapshot!(value_kinds_description_query_param(&[ValueKind::Integer]), @"a string");
        insta::assert_display_snapshot!(value_kinds_description_query_param(&[ValueKind::String]), @"a string");
        insta::assert_display_snapshot!(value_kinds_description_query_param(&[ValueKind::Sequence]), @"a string");
        insta::assert_display_snapshot!(value_kinds_description_query_param(&[ValueKind::Map]), @"a string");

        insta::assert_display_snapshot!(value_kinds_description_query_param(&[ValueKind::Integer, ValueKind::Boolean]), @"a string");
        insta::assert_display_snapshot!(value_kinds_description_query_param(&[ValueKind::Null, ValueKind::Integer]), @"a string");
        insta::assert_display_snapshot!(value_kinds_description_query_param(&[ValueKind::Sequence, ValueKind::NegativeInteger]), @"a string");
        insta::assert_display_snapshot!(value_kinds_description_query_param(&[ValueKind::Integer, ValueKind::Float]), @"a string");
        insta::assert_display_snapshot!(value_kinds_description_query_param(&[ValueKind::Integer, ValueKind::Float, ValueKind::NegativeInteger]), @"a string");
        insta::assert_display_snapshot!(value_kinds_description_query_param(&[ValueKind::Integer, ValueKind::Float, ValueKind::NegativeInteger, ValueKind::Null]), @"a string");
        insta::assert_display_snapshot!(value_kinds_description_query_param(&[ValueKind::Boolean, ValueKind::Integer, ValueKind::Float, ValueKind::NegativeInteger, ValueKind::Null]), @"a string");
        insta::assert_display_snapshot!(value_kinds_description_query_param(&[ValueKind::Null, ValueKind::Boolean, ValueKind::Integer, ValueKind::Float, ValueKind::NegativeInteger, ValueKind::Null]), @"a string");
    }

    #[test]
    fn error_msg_missing_field() {
        #[allow(dead_code)]
        #[derive(deserr::Deserr, Debug)]
        struct Missing {
            me: usize,
        }
        let value = json!({ "toto": 2 });
        let err = deserr::deserialize::<Missing, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Missing parameter `me`");
    }

    #[test]
    fn error_msg_incorrect() {
        #[allow(dead_code)]
        #[derive(deserr::Deserr, Debug)]
        struct Incorrect {
            me: usize,
        }
        let value = json!({ "me": [2] });
        let err = deserr::deserialize::<Incorrect, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Invalid value type for parameter `me`: expected a string, but found multiple values");

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
        let err = deserr::deserialize::<MultiIncorrect, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `la` for parameter `me`: expected one of `One`, `Two`, `Three`");

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
        let err =
            deserr::deserialize::<MultiIncorrectWithRename, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `la` for parameter `me`: expected one of `theobjectivecamelisnoice`, `bloup`");
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
        let err = deserr::deserialize::<SingleUnknownField, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown parameter `u`: expected one of `me`");

        #[allow(dead_code)]
        #[derive(deserr::Deserr, Debug)]
        #[deserr(deny_unknown_fields)]
        struct MultiUnknownField {
            me: usize,
            and: String,
        }
        let value = json!({ "me": 2, "and": "u", "uwu": "OwO" });
        let err = deserr::deserialize::<MultiUnknownField, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown parameter `uwu`: expected one of `me`, `and`");
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
        let err = deserr::deserialize::<UnexpectedTuple, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Invalid value in parameter `me`: the sequence should have exactly 2 elements");

        let value = json!({ "me": [2, 3, 4] });
        let err = deserr::deserialize::<UnexpectedTuple, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Invalid value in parameter `me`: the sequence should have exactly 2 elements");
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

        // did you mean triggered by an unknown key

        let value = json!({ "filler": "doggo" });
        let err = deserr::deserialize::<DidYouMean, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown parameter `filler`: did you mean `filter`? expected one of `q`, `filter`, `sort`, `attributesToHighlight`");

        let value = json!({ "sart": "doggo" });
        let err = deserr::deserialize::<DidYouMean, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown parameter `sart`: did you mean `sort`? expected one of `q`, `filter`, `sort`, `attributesToHighlight`");

        let value = json!({ "attributes_to_highlight": "doggo" });
        let err = deserr::deserialize::<DidYouMean, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown parameter `attributes_to_highlight`: did you mean `attributesToHighlight`? expected one of `q`, `filter`, `sort`, `attributesToHighlight`");

        let value = json!({ "attributesToHighloght": "doggo" });
        let err = deserr::deserialize::<DidYouMean, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown parameter `attributesToHighloght`: did you mean `attributesToHighlight`? expected one of `q`, `filter`, `sort`, `attributesToHighlight`");

        // doesn't match anything

        let value = json!({ "a": "doggo" });
        let err = deserr::deserialize::<DidYouMean, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown parameter `a`: expected one of `q`, `filter`, `sort`, `attributesToHighlight`");

        let value = json!({ "query": "doggo" });
        let err = deserr::deserialize::<DidYouMean, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown parameter `query`: expected one of `q`, `filter`, `sort`, `attributesToHighlight`");

        let value = json!({ "filterable": "doggo" });
        let err = deserr::deserialize::<DidYouMean, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown parameter `filterable`: expected one of `q`, `filter`, `sort`, `attributesToHighlight`");

        let value = json!({ "sortable": "doggo" });
        let err = deserr::deserialize::<DidYouMean, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown parameter `sortable`: expected one of `q`, `filter`, `sort`, `attributesToHighlight`");

        // did you mean triggered by an unknown value

        let value = json!({ "q": "filler" });
        let err = deserr::deserialize::<DidYouMean, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `filler` for parameter `q`: did you mean `filter`? expected one of `q`, `filter`, `sort`, `attributesToHighLight`");

        let value = json!({ "q": "sart" });
        let err = deserr::deserialize::<DidYouMean, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `sart` for parameter `q`: did you mean `sort`? expected one of `q`, `filter`, `sort`, `attributesToHighLight`");

        let value = json!({ "q": "attributes_to_highlight" });
        let err = deserr::deserialize::<DidYouMean, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `attributes_to_highlight` for parameter `q`: expected one of `q`, `filter`, `sort`, `attributesToHighLight`");

        let value = json!({ "q": "attributesToHighloght" });
        let err = deserr::deserialize::<DidYouMean, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `attributesToHighloght` for parameter `q`: did you mean `attributesToHighLight`? expected one of `q`, `filter`, `sort`, `attributesToHighLight`");

        // doesn't match anything

        let value = json!({ "q": "a" });
        let err = deserr::deserialize::<DidYouMean, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `a` for parameter `q`: expected one of `q`, `filter`, `sort`, `attributesToHighLight`");

        let value = json!({ "q": "query" });
        let err = deserr::deserialize::<DidYouMean, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `query` for parameter `q`: expected one of `q`, `filter`, `sort`, `attributesToHighLight`");

        let value = json!({ "q": "filterable" });
        let err = deserr::deserialize::<DidYouMean, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `filterable` for parameter `q`: expected one of `q`, `filter`, `sort`, `attributesToHighLight`");

        let value = json!({ "q": "sortable" });
        let err = deserr::deserialize::<DidYouMean, _, QueryParamError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Unknown value `sortable` for parameter `q`: expected one of `q`, `filter`, `sort`, `attributesToHighLight`");
    }
}
