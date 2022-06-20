/*!
Jayson is a crate for deserializing data, with the ability to return
custom, type-specific errors upon failure.

Unlike serde, Jayson does not parse the data in its serialization format itself,
but offload that work to other crates. Instead, it deserializes
the already-parsed serialized data into the final type. For example:

```ignore
// bytes of the serialized value
let s: &str = .. ;
// parsed serialized data
let json: serde_json::Value = serde_json::from_str(s)?;
// finally deserialize with Jayson
let data = T::deserialize_from_value(json.into_value())?;
```

Thus, Jayson
is a bit slower than crates that immediately deserialize a value while
parsing at the same time.

The main parts of Jayson are:
1. [`DeserializeFromValue<E>`] is the main trait for deserialization
2. [`IntoValue`] and [`Value`] describe the shape that the parsed serialized data must have
3. [`DeserializeError`] is the trait that all deserialization errors must conform to

If the feature `serde` is activated, then an implementation of [`IntoValue`] is provided
for the type `serde_json::Value`. This allows using Jayson to deserialize from JSON.
*/

#![allow(clippy::len_without_is_empty)]
mod impls;
#[cfg(feature = "serde_json")]
mod serde_json;

use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
};

pub use jayson_internal::DeserializeFromValue;

#[derive(Clone, Copy)]
pub enum ValuePointerRef<'a> {
    Origin,
    Key {
        key: &'a str,
        prev: &'a ValuePointerRef<'a>,
    },
    Index {
        index: usize,
        prev: &'a ValuePointerRef<'a>,
    },
}
impl<'a> Default for ValuePointerRef<'a> {
    fn default() -> Self {
        Self::Origin
    }
}
impl<'a> ValuePointerRef<'a> {
    #[must_use]
    pub fn push_key(&'a self, key: &'a str) -> Self {
        Self::Key { key, prev: self }
    }
    #[must_use]
    pub fn push_index(&'a self, index: usize) -> Self {
        Self::Index { index, prev: self }
    }
    pub fn to_owned(&'a self) -> ValuePointer {
        let mut cur = self;
        let mut components = vec![];
        loop {
            match cur {
                ValuePointerRef::Origin => break,
                ValuePointerRef::Key { key, prev } => {
                    components.push(ValuePointerComponent::Key(key.to_string()));
                    cur = prev;
                }
                ValuePointerRef::Index { index, prev } => {
                    components.push(ValuePointerComponent::Index(*index));
                    cur = prev;
                }
            }
        }
        ValuePointer { path: components }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValuePointerComponent {
    Key(String),
    Index(usize),
}

// TODO: custom Ord impl
/// Points to a subpart of a [`Value`].
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValuePointer {
    pub path: Vec<ValuePointerComponent>,
}
impl Display for ValuePointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for component in self.path.iter().rev() {
            match component {
                ValuePointerComponent::Index(i) => {
                    write!(f, ".{i}")?;
                }
                ValuePointerComponent::Key(s) => {
                    write!(f, ".{s}")?;
                }
            }
        }
        Ok(())
    }
}

/// Equivalent to [`Value`] but without the associated data.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Null,
    Boolean,
    Integer,
    NegativeInteger,
    Float,
    String,
    Sequence,
    Map,
}
impl Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueKind::Null => write!(f, "Null"),
            ValueKind::Boolean => write!(f, "Boolean"),
            ValueKind::Integer => write!(f, "Integer"),
            ValueKind::NegativeInteger => write!(f, "NegativeInteger"),
            ValueKind::Float => write!(f, "Float"),
            ValueKind::String => write!(f, "String"),
            ValueKind::Sequence => write!(f, "Sequence"),
            ValueKind::Map => write!(f, "Map"),
        }
    }
}
impl Debug for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

/// `Value<V>` is a view into the parsed serialization data (of type `V`) that
/// is readable by Jayson.
///
/// It is an enum with a variant for each possible value kind. The content of the variants
/// is either a simple value, such as `bool` or `String`, or an abstract [`Sequence`] or
/// [`Map`], which are views into the rest of the serialized data.
#[derive(Debug)]
pub enum Value<V: IntoValue> {
    Null,
    Boolean(bool),
    Integer(u64),
    NegativeInteger(i64),
    Float(f64),
    String(String),
    Sequence(V::Sequence),
    Map(V::Map),
}
impl<V: IntoValue> Value<V> {
    pub fn kind(&self) -> ValueKind {
        match self {
            Value::Null => ValueKind::Null,
            Value::Boolean(_) => ValueKind::Boolean,
            Value::Integer(_) => ValueKind::Integer,
            Value::NegativeInteger(_) => ValueKind::NegativeInteger,
            Value::Float(_) => ValueKind::Float,
            Value::String(_) => ValueKind::String,
            Value::Sequence(_) => ValueKind::Sequence,
            Value::Map(_) => ValueKind::Map,
        }
    }
}

/// A trait for a value that can be deserialized via [`DeserializeFromValue`].
pub trait IntoValue: Sized {
    type Sequence: Sequence<Value = Self>;
    type Map: Map<Value = Self>;

    fn kind(&self) -> ValueKind;
    fn into_value(self) -> Value<Self>;
}

/// A sequence of values conforming to [`IntoValue`].
pub trait Sequence {
    type Value: IntoValue;
    type Iter: Iterator<Item = Self::Value>;

    fn len(&self) -> usize;
    fn into_iter(self) -> Self::Iter;
}

/// A keyed map of values conforming to [`IntoValue`].
pub trait Map {
    type Value: IntoValue;
    type Iter: Iterator<Item = (String, Self::Value)>;

    fn len(&self) -> usize;
    fn remove(&mut self, key: &str) -> Option<Self::Value>;
    fn into_iter(self) -> Self::Iter;
}

/// A trait for types that can be deserialized from a [`Value`]. The generic type
/// parameter `E` is the custom error that is returned when deserialization fails.
pub trait DeserializeFromValue<E: DeserializeError>: Sized {
    /// Attempts to deserialize `Self` from the given value.
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E>;
    /// The value of `Self`, if any, when deserializing from a non-existent value.
    fn default() -> Option<Self> {
        None
    }
}

pub fn deserialize<Ret, Val, E>(value: Val) -> Result<Ret, E>
where
    Ret: DeserializeFromValue<E>,
    Val: IntoValue,
    E: DeserializeError,
{
    Ret::deserialize_from_value(value.into_value(), ValuePointerRef::Origin)
}

pub trait SingleDeserializeError {
    fn location(&self) -> Option<ValuePointer>;
    #[must_use]
    fn incorrect_value_kind(
        actual: ValueKind,
        accepted: &[ValueKind],
        location: ValuePointerRef,
    ) -> Self;
    #[must_use]
    fn missing_field(field: &str, location: ValuePointerRef) -> Self;
    #[must_use]
    fn unknown_key(key: &str, accepted: &[&str], location: ValuePointerRef) -> Self;
    #[must_use]
    fn unexpected(msg: &str, location: ValuePointerRef) -> Self;
}

pub trait MergeWithError<T>: Sized {
    fn merge(self_: Option<Self>, other: T, merge_location: ValuePointerRef) -> Result<Self, Self>;
}

impl<T, U> MergeWithError<U> for T
where
    T: SingleDeserializeError,
    T: From<U>,
{
    fn merge(
        self_: Option<Self>,
        other: U,
        _merge_location: ValuePointerRef,
    ) -> Result<Self, Self> {
        assert!(self_.is_none());
        Err(other.into())
    }
}
impl<T> DeserializeError for T
where
    T: SingleDeserializeError,
{
    fn incorrect_value_kind(
        self_: Option<Self>,
        actual: ValueKind,
        accepted: &[ValueKind],
        location: ValuePointerRef,
    ) -> Result<Self, Self> {
        assert!(self_.is_none());
        Err(Self::incorrect_value_kind(actual, accepted, location))
    }

    fn missing_field(
        self_: Option<Self>,
        field: &str,
        location: ValuePointerRef,
    ) -> Result<Self, Self> {
        assert!(self_.is_none());
        Err(Self::missing_field(field, location))
    }

    fn unknown_key(
        self_: Option<Self>,
        key: &str,
        accepted: &[&str],
        location: ValuePointerRef,
    ) -> Result<Self, Self> {
        assert!(self_.is_none());
        Err(Self::unknown_key(key, accepted, location))
    }

    fn unexpected(self_: Option<Self>, msg: &str, location: ValuePointerRef) -> Result<Self, Self> {
        assert!(self_.is_none());
        Err(Self::unexpected(msg, location))
    }

    fn location(&self) -> Option<ValuePointer> {
        <Self as SingleDeserializeError>::location(self)
    }
}

/// A trait for errors returned by [`deserialize_from_value`](DeserializeFromValue::deserialize_from_value).
pub trait DeserializeError: Sized + MergeWithError<Self> {
    fn location(&self) -> Option<ValuePointer>;

    #[must_use]
    fn incorrect_value_kind(
        self_: Option<Self>,
        actual: ValueKind,
        accepted: &[ValueKind],
        location: ValuePointerRef,
    ) -> Result<Self, Self>;
    #[must_use]
    fn missing_field(
        self_: Option<Self>,
        field: &str,
        location: ValuePointerRef,
    ) -> Result<Self, Self>;
    #[must_use]
    fn unknown_key(
        self_: Option<Self>,
        key: &str,
        accepted: &[&str],
        location: ValuePointerRef,
    ) -> Result<Self, Self>;
    #[must_use]
    fn unexpected(self_: Option<Self>, msg: &str, location: ValuePointerRef) -> Result<Self, Self>;
}

#[derive(Clone, Debug)]
pub enum StandardError {
    IncorrectValueKind {
        actual: ValueKind,
        accepted: Vec<ValueKind>,
    },
    MissingField {
        field: String,
    },
    UnknownKey {
        key: String,
        accepted: Vec<String>,
    },
    Unexpected {
        message: String,
    },
}
impl Display for StandardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StandardError::IncorrectValueKind { actual, accepted } => {
                writeln!(f, "Found a {actual} but expected one of {accepted:?}")
            }
            StandardError::MissingField { field } => {
                writeln!(f, "Missing field {field}")
            }
            StandardError::UnknownKey { key, accepted } => {
                writeln!(f, "Unknown key {key}. Expected one of {accepted:?}")
            }
            StandardError::Unexpected { message } => {
                writeln!(f, "{message}")
            }
        }
    }
}
impl std::error::Error for StandardError {}

impl SingleDeserializeError for StandardError {
    fn incorrect_value_kind(
        actual: ValueKind,
        accepted: &[ValueKind],
        _location: ValuePointerRef,
    ) -> Self {
        Self::IncorrectValueKind {
            actual,
            accepted: accepted.to_vec(),
        }
    }

    fn missing_field(field: &str, _location: ValuePointerRef) -> Self {
        Self::MissingField {
            field: field.to_string(),
        }
    }

    fn unknown_key(key: &str, accepted: &[&str], _location: ValuePointerRef) -> Self {
        Self::UnknownKey {
            key: key.to_string(),
            accepted: accepted.into_iter().map(<_>::to_string).collect(),
        }
    }

    fn unexpected(msg: &str, _location: ValuePointerRef) -> Self {
        Self::Unexpected {
            message: msg.to_string(),
        }
    }

    fn location(&self) -> Option<ValuePointer> {
        None
    }
}

#[derive(Debug)]
pub struct AccumulatedErrors<E> {
    pub locations: BTreeMap<ValuePointer, Vec<E>>,
}
impl<E> Default for AccumulatedErrors<E> {
    fn default() -> Self {
        Self {
            locations: Default::default(),
        }
    }
}

impl<E> MergeWithError<Self> for AccumulatedErrors<E>
where
    E: DeserializeError,
{
    fn merge(
        self_: Option<Self>,
        other: Self,
        _merge_location: ValuePointerRef,
    ) -> Result<Self, Self> {
        let mut self_ = self_.unwrap_or_default();
        for (key, value) in other.locations {
            self_.locations.entry(key).or_default().extend(value);
        }
        Ok(self_)
    }
}
#[allow(unused)]
impl<E> MergeWithError<E> for AccumulatedErrors<E>
where
    E: DeserializeError,
{
    fn merge(self_: Option<Self>, other: E, merge_location: ValuePointerRef) -> Result<Self, Self> {
        let mut self_ = self_.unwrap_or_default();
        // If the added error has no location, we add it to the origin
        let location = other.location().unwrap_or_default();
        self_.locations.entry(location).or_default().push(other);
        Ok(self_)
    }
}
#[allow(unused)]
impl<E> DeserializeError for AccumulatedErrors<E>
where
    E: DeserializeError,
{
    fn location(&self) -> Option<ValuePointer> {
        if let Some((_, value)) = self.locations.iter().next() {
            value[0].location()
        } else {
            None
        }
    }

    fn incorrect_value_kind(
        self_: Option<Self>,
        actual: ValueKind,
        accepted: &[ValueKind],
        location: ValuePointerRef,
    ) -> Result<Self, Self> {
        let mut self_ = self_.unwrap_or_default();
        let new_err =
            take_result_content(E::incorrect_value_kind(None, actual, accepted, location));
        let location = location.to_owned();
        self_.locations.entry(location).or_default().push(new_err);
        Ok(self_)
    }

    fn missing_field(
        self_: Option<Self>,
        field: &str,
        location: ValuePointerRef,
    ) -> Result<Self, Self> {
        let mut self_ = self_.unwrap_or_default();
        let new_err = take_result_content(E::missing_field(None, field, location));
        let location = location.to_owned();
        self_.locations.entry(location).or_default().push(new_err);
        Ok(self_)
    }

    fn unknown_key(
        self_: Option<Self>,
        key: &str,
        accepted: &[&str],
        location: ValuePointerRef,
    ) -> Result<Self, Self> {
        let mut self_ = self_.unwrap_or_default();
        let new_err = take_result_content(E::unknown_key(None, key, accepted, location));
        let location = location.to_owned();
        self_.locations.entry(location).or_default().push(new_err);
        Ok(self_)
    }

    fn unexpected(self_: Option<Self>, msg: &str, location: ValuePointerRef) -> Result<Self, Self> {
        let mut self_ = self_.unwrap_or_default();
        let new_err = take_result_content(E::unexpected(None, msg, location));
        let location = location.to_owned();
        self_.locations.entry(location).or_default().push(new_err);
        Ok(self_)
    }
}
impl<E> Display for AccumulatedErrors<E>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (location, errors) in self.locations.iter() {
            writeln!(f, "Errors at {location}:")?;
            for e in errors {
                writeln!(f, "{e}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl<E> std::error::Error for AccumulatedErrors<E> where E: std::error::Error {}
#[doc(hidden)]
pub enum FieldState<T> {
    Missing,
    Err,
    Some(T),
}
impl<T> From<Option<T>> for FieldState<T> {
    fn from(x: Option<T>) -> Self {
        match x {
            Some(x) => FieldState::Some(x),
            None => FieldState::Missing,
        }
    }
}
impl<T> FieldState<T> {
    pub fn is_missing(&self) -> bool {
        matches!(self, FieldState::Missing)
    }
    #[track_caller]
    pub fn unwrap(self) -> T {
        match self {
            FieldState::Some(x) => x,
            _ => panic!("Unwrapping an empty field state"),
        }
    }
}

#[doc(hidden)]
pub fn take_result_content<T>(r: Result<T, T>) -> T {
    match r {
        Ok(x) => x,
        Err(x) => x,
    }
}
