# Deserr

## Introduction

Deserr is a crate for deserializing data, with the ability to return
custom, type-specific errors upon failure.

Unlike serde, Deserr does not parse the data in its serialization format itself,
but offload the work to other crates. Instead, it deserializes
the already-parsed serialized data into the final type. For example:

```rust,ignore
// bytes of the serialized value
let s: &str = ".." ;
// parse serialized data using another crate, such as `serde_json`
let json: serde_json::Value = serde_json::from_str(s).unwrap();
// finally deserialize with deserr
let data = T::deserialize_from_value(json.into_value()).unwrap();
// `T` must implements `DeserializeFromValue`.
```

Thus, Deserr is slower than crates that immediately deserialize a value while
parsing at the same time.

The main parts of Deserr are:
1. [`DeserializeFromValue<E>`] is the main trait for deserialization
2. [`IntoValue`] and [`Value`] describe the shape that the parsed serialized data must have
3. [`DeserializeError`] is the trait that all deserialization errors must conform to
4. [`MergeWithError<E>`] describes how to combine multiple errors together. It allows Deserr
to return multiple deserialization errors at once.
5. [`ValuePointerRef`] and [`ValuePointer`] point to locations within the value. They are
used to locate the origin of an error.
6. [`deserialize`] is the main function to use to deserialize a value
7. The [`DeserializeFromValue`](derive@DeserializeFromValue) derive proc macro

If the feature `serde` is activated, then an implementation of [`IntoValue`] is provided
for the type `serde_json::Value`. This allows using Deserr to deserialize from JSON.

## Example

### Implementing deserialize for a custom type
```rust
use deserr::{DeserializeError, DeserializeFromValue, DefaultError, Value, ValueKind, IntoValue, MergeWithError, ValuePointerRef, ValuePointer};

enum MyError {
    ForbiddenName,
    Other(DefaultError)
}

impl DeserializeError for MyError {
    /// Return the origin of the error, if it can be found
    fn location(&self) -> Option<ValuePointer> {
        None
    }

    /// Create a new error due to an unexpected value kind.
    ///
    /// Return `Ok` to continue deserializing or `Err` to fail early.
    fn incorrect_value_kind<V: IntoValue>(_self_: Option<Self>, actual: Value<V>, accepted: &[ValueKind], location: ValuePointerRef) -> Result<Self, Self> {
        Err(Self::Other(DefaultError::incorrect_value_kind(None, actual, accepted, location)?))
    }

    /// Create a new error due to a missing key.
    ///
    /// Return `Ok` to continue deserializing or `Err` to fail early.
    fn missing_field(_self_: Option<Self>, field: &str, location: ValuePointerRef) -> Result<Self, Self> {
        Err(Self::Other(DefaultError::missing_field(None, field, location)?))
    }

    /// Create a new error due to finding an unknown key.
    ///
    /// Return `Ok` to continue deserializing or `Err` to fail early.
    fn unknown_key(_self_: Option<Self>, key: &str, accepted: &[&str], location: ValuePointerRef) -> Result<Self, Self> {
        Err(Self::Other(DefaultError::unknown_key(None, key, accepted, location)?))
    }

    /// Create a new error with the custom message.
    ///
    /// Return `Ok` to continue deserializing or `Err` to fail early.
    fn unexpected(_self_: Option<Self>, field: &str, location: ValuePointerRef) -> Result<Self, Self> {
        Err(Self::Other(DefaultError::unexpected(None, field, location)?))
    }
}

impl From<DefaultError> for MyError {
    fn from(error: DefaultError) -> Self {
        Self::Other(error)
    }
}

impl MergeWithError<MyError> for MyError {
    fn merge(self_: Option<Self>, other: MyError, merge_location: ValuePointerRef) -> Result<Self, Self> {
        Err(other)
    }
}

struct Name(String);

impl DeserializeFromValue<MyError> for Name {
    fn deserialize_from_value<V: IntoValue>(value: Value<V>, location: ValuePointerRef) -> Result<Self, MyError> {
        match value {
            Value::String(s) => {
                if s == "Robert '); DROP TABLE Students; --" {
                    Err(MyError::ForbiddenName)
                } else {
                    Ok(Name(s))
                }
            }
            value => {
                match MyError::incorrect_value_kind(None, value, &[ValueKind::String], location) {
                    Ok(_) => unreachable!(),
                    Err(e) => Err(e),
                }
            }
        }
    }
}
```

### Using macros

```rust,ignore
#[derive(DeserializeFromValue)]
#[deserr(error = MyError)]
struct User {
	name: Name,
}
```

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
