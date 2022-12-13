use crate::*;

#[derive(Debug, PartialEq, Eq)]
pub enum DefaultError {
    Unexpected(String),
    MissingField(String),
    IncorrectValueKind { accepted: Vec<ValueKind> },
    UnknownKey { key: String, accepted: Vec<String> },
    CustomMissingField(u8),
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
    fn location(&self) -> Option<ValuePointer> {
        None
    }

    fn incorrect_value_kind(
        _self_: Option<Self>,
        _actual: ValueKind,
        accepted: &[ValueKind],
        _location: ValuePointerRef,
    ) -> Result<Self, Self> {
        Err(Self::IncorrectValueKind {
            accepted: accepted.into(),
        })
    }

    fn missing_field(
        _self_: Option<Self>,
        field: &str,
        _location: ValuePointerRef,
    ) -> Result<Self, Self> {
        Err(Self::MissingField(field.to_string()))
    }

    fn unknown_key(
        _self_: Option<Self>,
        key: &str,
        accepted: &[&str],
        _location: ValuePointerRef,
    ) -> Result<Self, Self> {
        Err(Self::UnknownKey {
            key: key.to_string(),
            accepted: accepted.iter().map(<_>::to_string).collect(),
        })
    }

    fn unexpected(
        _self_: Option<Self>,
        msg: &str,
        _location: ValuePointerRef,
    ) -> Result<Self, Self> {
        Err(Self::Unexpected(msg.to_string()))
    }
}
