# Deserr

## Introduction

Deserr is a crate for deserializing data, with the ability to return
custom, type-specific errors upon failure.

Unlike serde, Deserr does not parse the data in its serialization format itself,
but offload the work to other crates. Instead, it deserializes
the already-parsed serialized data into the final type. For example:

```rust
// bytes of the serialized value
let s: &str = .. ;
// parse serialized data using another crate, such as serde_json
let json: serde_json::Value = serde_json::from_str(s)?;
// finally deserialize with Deserr
let data = T::deserialize_from_value(json.into_value())?;
```

## Example

### Implementing deserialize for a custom type
```rust
use deserr::{DeserializeError, DeserializeFromValue, Error};
enum MyError {
    ForbiddenName,
    Other(Error)
}
impl DeserializeError for MyError {
    fn unexpected(s: &str) -> Self {
        Self::Other(Error::unexpected(s))
    }
    fn missing_field(field: &str) -> Self {
        Self::Other(Error::missing_field(field))
    }
    fn incorrect_value_kind(accepted: &[ValueKind]) -> Self {
        Self::Other(Error::incorrect_value_kind(accepted))
    }
}

struct Name(String);

impl DeserializeFromValue<MyError> for Name {
    fn deserialize_from_value<V: IntoValue>(value: Value<V>) -> Result<Self, MyError> {
        match value {
            Value::String(s) => {
                if s == "Robert '); DROP TABLE Students; --" {
                    Err(MyError::ForbiddenName)
                } else {
                    Ok(Name(s))
                }
            }
            _ => {
                Err(MyError::incorrect_value_kind(&[ValueKind::String]))
            }
        }
    }
}
```

### Using macros

```rust
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
