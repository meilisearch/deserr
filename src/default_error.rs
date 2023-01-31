use std::num::ParseIntError;

use crate::*;

#[derive(Debug, PartialEq, Eq)]
pub struct DefaultError {
    pub location: ValuePointer,
    pub content: DefaultErrorContent,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DefaultErrorContent {
    Unexpected(String),
    MissingField(String),
    IncorrectValueKind {
        accepted: Vec<ValueKind>,
    },
    UnknownKey {
        key: String,
        accepted: Vec<String>,
    },
    UnknownValue {
        value: String,
        accepted: Vec<String>,
    },
    CustomMissingField(usize),
    Validation,
}

impl MergeWithError<DefaultError> for DefaultError {
    fn merge(
        _self_: Option<Self>,
        other: DefaultError,
        _merge_location: ValuePointerRef,
    ) -> ControlFlow<Self, Self> {
        ControlFlow::Break(other)
    }
}

impl DeserializeError for DefaultError {
    fn error<V: IntoValue>(
        _self_: Option<Self>,
        error: ErrorKind<V>,
        location: ValuePointerRef,
    ) -> ControlFlow<Self, Self> {
        let content = match error {
            ErrorKind::IncorrectValueKind {
                actual: _,
                accepted,
            } => DefaultErrorContent::IncorrectValueKind {
                accepted: accepted.to_vec(),
            },
            ErrorKind::MissingField { field } => {
                DefaultErrorContent::MissingField(field.to_string())
            }
            ErrorKind::UnknownKey { key, accepted } => DefaultErrorContent::UnknownKey {
                key: key.to_string(),
                accepted: accepted
                    .iter()
                    .map(|accepted| accepted.to_string())
                    .collect(),
            },
            ErrorKind::UnknownValue { value, accepted } => DefaultErrorContent::UnknownValue {
                value: value.to_string(),
                accepted: accepted
                    .iter()
                    .map(|accepted| accepted.to_string())
                    .collect(),
            },
            ErrorKind::Unexpected { msg } => DefaultErrorContent::Unexpected(msg),
        };
        ControlFlow::Break(Self {
            location: location.to_owned(),
            content,
        })
    }
}

impl From<ParseIntError> for DefaultError {
    fn from(value: ParseIntError) -> Self {
        Self {
            location: ValuePointerRef::Origin.to_owned(),
            content: DefaultErrorContent::Unexpected(value.to_string()),
        }
    }
}
