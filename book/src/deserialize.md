# Implementing deserialize manually

The [`Deserr`](https://docs.rs/deserr/latest/deserr/trait.Deserr.html) trait looks like this:

```rust
pub trait Deserr<E: deserr::DeserializeError>: Sized {
    fn deserialize_from_value<V: deserr::IntoValue>(
        value: deserr::Value<V>,
        location: deserr::ValuePointerRef<'_>,
    ) -> Result<Self, E>;
}
```

The method's job is to deserialize a value to the concrete type you're implementing this trait on.
It's useful when the derive macro is not powerful enough for you.
For example you'll often need to implement the type yourself while working with enums since deserr
only supports unit enums.

One of the most common type you might need while working with json is a type that represents if a value
is `Set` (specified by the user), `NotSet` (the field is not present) or `Reset` (the field is set to `null`).
Instead of working with an `Option<Option<Value>>` we may want to introduce the following enum and implement `Deserr` on it:
```rust
use deserr::{DeserializeError, Deserr};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Setting<T> {
    Set(T),
    Reset,
    NotSet,
}

// If the value is missing we're going to rely on its default implementation of `NotSet`.
impl<T> Default for Setting<T> {
    fn default() -> Self {
        Self::NotSet
    }
}

impl<T, E> Deserr<E> for Setting<T>
where
    T: Deserr<E>,
    E: DeserializeError,
{
    fn deserialize_from_value<V: deserr::IntoValue>(
        value: deserr::Value<V>,
        location: deserr::ValuePointerRef<'_>,
    ) -> Result<Self, E> {
        match value {
            deserr::Value::Null => Ok(Setting::Reset),
            // If the value contains something, we let the inner type deserialize it
            _ => T::deserialize_from_value(value, location).map(Setting::Set),
        }
    }
}
```
