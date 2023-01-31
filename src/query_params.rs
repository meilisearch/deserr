//! This module implements the error messages of query parameters deserialization errors.
//! We try to generate the best possible human-readable description of the error.

use crate::{DeserializeError, MergeWithError};
use deserr::{ErrorKind, IntoValue, ValueKind, ValuePointerRef};
use std::{convert::Infallible, fmt::Display, ops::ControlFlow};

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
                    "Unknown parameter `{}`{location}: expected one of {}",
                    key,
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
                    "Unknown value `{}`{location}: expected one of {}",
                    value,
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
fn value_kinds_description_query_param(_accepted: &[ValueKind]) -> String {
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
