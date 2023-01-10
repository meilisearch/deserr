use std::num::ParseIntError;

use crate::*;

#[derive(Debug, PartialEq, Eq)]
pub enum DefaultError {
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
    ) -> Result<Self, Self> {
        Err(other)
    }
}

impl DeserializeError for DefaultError {
    fn error<V: IntoValue>(
        _self_: Option<Self>,
        error: ErrorKind<V>,
        _location: ValuePointerRef,
    ) -> Result<Self, Self> {
        Err(match error {
            ErrorKind::IncorrectValueKind {
                actual: _,
                accepted,
            } => Self::IncorrectValueKind {
                accepted: accepted.to_vec(),
            },
            ErrorKind::MissingField { field } => Self::MissingField(field.to_string()),
            ErrorKind::UnknownKey { key, accepted } => Self::UnknownKey {
                key: key.to_string(),
                accepted: accepted
                    .iter()
                    .map(|accepted| accepted.to_string())
                    .collect(),
            },
            ErrorKind::UnknownValue { value, accepted } => Self::UnknownValue {
                value: value.to_string(),
                accepted: accepted
                    .iter()
                    .map(|accepted| accepted.to_string())
                    .collect(),
            },
            ErrorKind::Unexpected { msg } => Self::Unexpected(msg),
        })
    }
}

impl From<ParseIntError> for DefaultError {
    fn from(value: ParseIntError) -> Self {
        Self::Unexpected(value.to_string())
    }
}
