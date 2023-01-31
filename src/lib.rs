#![doc = include_str!("../README.md")]

#[cfg(feature = "actix-web")]
pub mod actix_web;
mod default_error;
mod impls;
mod json;
mod query_params;
#[cfg(feature = "serde-cs")]
pub mod serde_cs;
#[cfg(feature = "serde-json")]
pub mod serde_json;
mod value;

pub use default_error::DefaultError;
pub use default_error::DefaultErrorContent;
pub use json::JsonError;
pub use query_params::QueryParamError;
extern crate self as deserr;

/**
It is possible to derive the `Deserr` trait for structs and enums with named fields.
The derive proc macro accept many arguments, explained below:

The basic usage is as follows:
```
use deserr::Deserr;

#[derive(Deserr)]
struct MyStruct {
    x: bool,
    y: u8,
}
```
This will implement `impl<E> Deserr<E> MyStruct` for all `E: DeserializeError`.

To use it on enums, the attribute `tag` must be added:
```
use deserr::Deserr;

#[derive(Deserr)]
#[deserr(tag = "my_enum_tag")]
enum MyEnum {
    A,
    B { x: bool, y: u8 }
}
```
This will correctly deserialize the given enum for values of this shape:
```json
{
    "my_enum_tag": "A"
}
// or
{
    "my_enum_tag": "B",
    "x": true,
    "y": 1
}
```

It is possible to change the name of the keys corresponding to each field using the `rename` and `rename_all`
attributes:

```rust
use deserr::Deserr;
#[derive(Deserr)]
#[deserr(rename_all = camelCase)]
struct MyStruct {
    my_field: bool,
    #[deserr(rename = "goodbye_world")]
    hello_world: u8,
}
```
will parse the following:
```json
{
    "myField": 1,
    "goodbye_world": 2
}
```
*/
pub use deserr_internal::Deserr;
pub use value::{IntoValue, Map, Sequence, Value, ValueKind, ValuePointer, ValuePointerRef};

use std::fmt::Debug;
use std::ops::ControlFlow;

/// A trait for types that can be deserialized from a [`Value`]. The generic type
/// parameter `E` is the custom error that is returned when deserialization fails.
pub trait Deserr<E: DeserializeError>: Sized {
    /// Attempts to deserialize `Self` from the given value. Note that this method is an
    /// implementation detail. You probably want to use the [`deserialize`] function directly instead.
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E>;
}

/// Deserialize the given value.
///
/// This function has three generic arguments, two of which can often be inferred.
/// 1. `Ret` is the type we want to deserialize to. For example: `MyStruct`
/// 2. `Val` is the type of the value given as argument. For example: `serde_json::Value`
/// 3. `E` is the error type we want to get when deserialization fails. For example: `MyError`
pub fn deserialize<Ret, Val, E>(value: Val) -> Result<Ret, E>
where
    Ret: Deserr<E>,
    Val: IntoValue,
    E: DeserializeError,
{
    Ret::deserialize_from_value(value.into_value(), ValuePointerRef::Origin)
}

/// A trait which describes how to combine two errors together.
pub trait MergeWithError<T>: Sized {
    /// Merge two errors together.
    ///
    /// ## Arguments:
    /// - `self_`: the existing error, if any
    /// - `other`: the new error
    /// - `merge_location`: the location where the merging happens.
    ///
    /// ## Return value
    /// It should return the merged error inside a `Result`.
    ///
    /// The variant of the returned result should be `Ok(e)` to signal that the deserialization
    /// should continue (to accumulate more errors), or `Err(e)` to stop the deserialization immediately.
    ///
    /// Note that in both cases, the deserialization should eventually fail.
    ///
    /// ## Example
    /// Imagine you have the following json:
    /// ```json
    /// {
    ///    "w": true,
    ///    "x" : { "y": 1 }
    /// }
    /// ```
    /// It may be that deserializing the first field, `w`, fails with error `suberror: E`. This is the
    /// first deserialization error we encounter, so the current error value is `None`. The function `Self::merge`
    /// is called as follows:
    /// ```ignore
    /// // let mut error = None;
    /// // let mut location : ValuePointerRef::Origin;
    /// error = Some(Self::merge(error, suberror, location.push_key("w"))?);
    /// // if the returned value was Err(e), then we returned early from the deserialize method
    /// // otherwise, `error` is now set
    /// ```
    /// Later on, we encounter a new suberror originating from `x.y`. The `merge` function is called again:
    /// ```ignore
    /// // let mut error = Some(..);
    /// // let mut location : ValuePointerRef::Origin;
    /// error = Some(Self::merge(error, suberror, location.push_key("x"))?);
    /// // if the returned value was Err(e), then we returned early from the deserialize method
    /// // otherwise, `error` is now the result of its merging with suberror.
    /// ```
    /// Note that even though the suberror originated at `x.y`, the `merge_location` argument was `x`
    /// because that is where the merge happened.
    fn merge(
        self_: Option<Self>,
        other: T,
        merge_location: ValuePointerRef,
    ) -> ControlFlow<Self, Self>;
}

pub enum ErrorKind<'a, V: IntoValue> {
    IncorrectValueKind {
        actual: Value<V>,
        accepted: &'a [ValueKind],
    },
    MissingField {
        field: &'a str,
    },
    UnknownKey {
        key: &'a str,
        accepted: &'a [&'a str],
    },
    UnknownValue {
        value: &'a str,
        accepted: &'a [&'a str],
    },
    Unexpected {
        msg: String,
    },
}

/// A trait for errors returned by [`deserialize_from_value`](Deserr::deserialize_from_value).
pub trait DeserializeError: Sized + MergeWithError<Self> {
    fn error<V: IntoValue>(
        self_: Option<Self>,
        error: ErrorKind<V>,
        location: ValuePointerRef,
    ) -> ControlFlow<Self, Self>;
}

/// Used by the derive proc macro. Do not use.
#[doc(hidden)]
pub enum FieldState<T> {
    Missing,
    Err,
    Some(T),
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

    #[track_caller]
    pub fn unwrap_or(self, value: T) -> T {
        match self {
            FieldState::Some(x) => x,
            FieldState::Missing => value,
            FieldState::Err => value,
        }
    }

    #[track_caller]
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            FieldState::Some(x) => Ok(x),
            FieldState::Missing => Err(err),
            FieldState::Err => Err(err),
        }
    }

    pub fn map<U>(self, f: impl Fn(T) -> U) -> FieldState<U> {
        match self {
            FieldState::Some(x) => FieldState::Some(f(x)),
            FieldState::Missing => FieldState::Missing,
            FieldState::Err => FieldState::Err,
        }
    }
}

/// Extract the `ControlFlow` result if it's the same type.
pub fn take_cf_content<T>(r: ControlFlow<T, T>) -> T {
    match r {
        ControlFlow::Continue(x) => x,
        ControlFlow::Break(x) => x,
    }
}
