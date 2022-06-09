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

pub use jayson_internal::DeserializeFromValue;

/// Equivalent to [`Value`] but without the associated data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    fn deserialize_from_value<V: IntoValue>(value: Value<V>) -> Result<Self, E>;
    /// The value of `Self`, if any, when deserializing from a non-existent value.
    fn default() -> Option<Self> {
        None
    }
}

/// A trait for errors returned by [`deserialize_from_value`](DeserializeFromValue::deserialize_from_value).
pub trait DeserializeError {
    fn incorrect_value_kind(accepted: &[ValueKind]) -> Self;
    fn missing_field(field: &str) -> Self;
    fn unexpected(msg: &str) -> Self;
}

/// A default error type implementing [`DeserializeError`], for convenience.
#[derive(Debug)]
pub enum Error {
    IncorrectValueKind { accepted: Vec<ValueKind> },
    Unexpected(String),
    MissingField(String),
}

impl DeserializeError for Error {
    fn unexpected(s: &str) -> Self {
        Self::Unexpected(s.to_owned())
    }

    fn missing_field(field: &str) -> Self {
        Self::MissingField(field.to_owned())
    }

    fn incorrect_value_kind(accepted: &[ValueKind]) -> Self {
        Self::IncorrectValueKind {
            accepted: accepted.to_vec(),
        }
    }
}
