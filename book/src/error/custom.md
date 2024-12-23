# Defining your own error

Defining your own error type comes down to implementing the [`DeserrError`](https://docs.rs/deserr/latest/deserr/trait.DeserializeError.html) trait that looks like that:
```rust
pub trait DeserializeError: Sized + deserr::MergeWithError<Self> {
    fn error<V: deserr::IntoValue>(
        self_: Option<Self>,
        error: deserr::ErrorKind<'_, V>,
        location: deserr::ValuePointerRef<'_>,
    ) -> std::ops::ControlFlow<Self, Self>;
}
```

The method's job is to build your custom error type from an error kind and a location.
deserr will call this method everytime it encounter an error while deserializing the specified payload and your job will be
to craft your own error type from the parameters, and let deserr know if it should continue to explore the payload looking
for more errors or stop immediately.
- `_self` contains the previous version of your error if you told deserr to accumulate errors.
- `error` the error encountered by deserr whil deserializing the value. 
- `location` the location of the error
- [`ControlFlow`](https://doc.rust-lang.org/stable/std/ops/enum.ControlFlow.html) is your way to tell deserr to continue accumulating errors or to stop.

And you may have noticed that your type must also implements the [`MergeWithError`](https://docs.rs/deserr/latest/deserr/trait.MergeWithError.html) trait.
This trait describe error type that can be merged together to return only one final type.
It also gives you the opportunity to tell deserr to stop deserializing the structure.

```rust
pub trait MergeWithError<T>: Sized {
    fn merge(
        self_: Option<Self>,
        other: T,
        merge_location: deserr::ValuePointerRef<'_>,
    ) -> std::ops::ControlFlow<Self, Self>;
}
```

This trait also gives you the opportunity to merge **an other** error type with your error type.
