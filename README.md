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
// `T` must implements `Deserr`.
```

Thus, Deserr is slower than crates that immediately deserialize a value while
parsing at the same time.

The main parts of Deserr are:
1. [`Deserr<E>`] is the main trait for deserialization
2. [`IntoValue`] and [`Value`] describe the shape that the parsed serialized data must have
3. [`DeserializeError`] is the trait that all deserialization errors must conform to
4. [`MergeWithError<E>`] describes how to combine multiple errors together. It allows Deserr
to return multiple deserialization errors at once.
5. [`ValuePointerRef`] and [`ValuePointer`] point to locations within the value. They are
used to locate the origin of an error.
6. [`deserialize`] is the main function to use to deserialize a value
7. The [`Deserr`](derive@Deserr) derive proc macro

If the feature `serde` is activated, then an implementation of [`IntoValue`] is provided
for the type `serde_json::Value`. This allows using Deserr to deserialize from JSON.

## Example

### Implementing deserialize for a custom type
```rust
use deserr::{DeserializeError, Deserr, ErrorKind, JsonError, Value, ValueKind, IntoValue, take_cf_content, MergeWithError, ValuePointerRef, ValuePointer};
use std::ops::ControlFlow;

enum MyError {
    ForbiddenName,
    Other(JsonError)
}

impl DeserializeError for MyError {
    /// Create a new error with the custom message.
    ///
    /// Return `Ok` to continue deserializing or `Err` to fail early.
    fn error<V: IntoValue>(_self_: Option<Self>, error: ErrorKind<V>, location: ValuePointerRef) -> ControlFlow<Self, Self> {
        ControlFlow::Break(Self::Other(take_cf_content(JsonError::error(None, error, location))))
    }
}

impl From<JsonError> for MyError {
    fn from(error: JsonError) -> Self {
        Self::Other(error)
    }
}

impl MergeWithError<MyError> for MyError {
    fn merge(self_: Option<Self>, other: MyError, merge_location: ValuePointerRef) -> ControlFlow<Self, Self> {
        ControlFlow::Break(other)
    }
}

struct Name(String);

impl Deserr<MyError> for Name {
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
                match MyError::error(None, ErrorKind::IncorrectValueKind { actual: value, accepted: &[ValueKind::String] }, location) {
                    ControlFlow::Continue(_) => unreachable!(),
                    ControlFlow::Break(e) => Err(e),
                }
            }
        }
    }
}
```

### Using macros

```rust,ignore
#[derive(Deserr)]
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
