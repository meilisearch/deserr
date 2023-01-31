use crate::{
    take_cf_content, DeserializeError, Deserr, ErrorKind, IntoValue, Map, Sequence,
    Value, ValueKind, ValuePointerRef,
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    convert::TryFrom,
    hash::Hash,
    marker::PhantomData,
    ops::ControlFlow,
    str::FromStr,
};

impl<T, E> Deserr<E> for PhantomData<T>
where
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        _value: Value<V>,
        _location: ValuePointerRef,
    ) -> Result<Self, E> {
        Ok(Self)
    }
}

impl<T> Sequence for Vec<T>
where
    T: IntoValue,
{
    type Value = T;
    type Iter = <Self as IntoIterator>::IntoIter;

    fn len(&self) -> usize {
        self.len()
    }

    fn into_iter(self) -> Self::Iter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<E> Deserr<E> for ()
where
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Null => Ok(()),
            v => Err(take_cf_content(E::error(
                None,
                ErrorKind::IncorrectValueKind {
                    actual: v,
                    accepted: &[ValueKind::Null],
                },
                location,
            ))),
        }
    }
}

impl<E> Deserr<E> for bool
where
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Boolean(b) => Ok(b),
            v => Err(take_cf_content(E::error(
                None,
                ErrorKind::IncorrectValueKind {
                    actual: v,
                    accepted: &[ValueKind::Boolean],
                },
                location,
            ))),
        }
    }
}

macro_rules! deserialize_impl_integer {
    ($t:ty) => {
        impl<E> Deserr<E> for $t
        where
            E: DeserializeError,
        {
            fn deserialize_from_value<V: IntoValue>(
                value: Value<V>,
                location: ValuePointerRef,
            ) -> Result<Self, E> {
                use $crate::take_cf_content;

                let err = |value: Value<V>| {
                    E::error(
                        None,
                        ErrorKind::IncorrectValueKind {
                            actual: value,
                            accepted: &[ValueKind::Integer],
                        },
                        location,
                    )
                };

                match value {
                    Value::Integer(x) => <$t>::try_from(x).or_else(|_| {
                        Err(take_cf_content(E::error::<V>(
                            None,
                            ErrorKind::Unexpected {
                                msg: format!(
                                    "value: `{x}` is too large to be deserialized, maximum value authorized is `{}`",
                                    <$t>::MAX
                                ),
                            },
                            location,
                        )))
                    }),
                    v => Err(take_cf_content(err(v))),
                }
            }
        }
    };
}
deserialize_impl_integer!(u8);
deserialize_impl_integer!(u16);
deserialize_impl_integer!(u32);
deserialize_impl_integer!(u64);
deserialize_impl_integer!(usize);

macro_rules! deserialize_impl_negative_integer {
    ($t:ty) => {
        impl<E> Deserr<E> for $t
        where
            E: DeserializeError,
        {
            fn deserialize_from_value<V: IntoValue>(
                value: Value<V>,
                location: ValuePointerRef,
            ) -> Result<Self, E> {
                use $crate::take_cf_content;

                let err = |value: Value<V>| {
                    E::error(
                        None,
                        ErrorKind::IncorrectValueKind {
                            actual: value,
                            accepted: &[ValueKind::Integer, ValueKind::NegativeInteger],
                        },
                        location,
                    )
                };

                match value {
                    Value::Integer(x) => <$t>::try_from(x).or_else(|_| {
                        Err(take_cf_content(E::error::<V>(
                            None,
                            ErrorKind::Unexpected {
                                msg: format!(
                                    "value: `{x}` is too large to be deserialized, maximum value authorized is `{}`",
                                    <$t>::MAX
                                ),
                            },
                            location,
                        )))
                    }),
                    Value::NegativeInteger(x) => <$t>::try_from(x).or_else(|_| {
                        Err(take_cf_content(E::error::<V>(
                            None,
                            ErrorKind::Unexpected {
                                msg: format!(
                                    "value: `{x}` is too small to be deserialized, minimum value authorized is `{}`",
                                    <$t>::MIN
                                ),
                            },
                            location,
                        )))
                    }),
                    v => Err(take_cf_content(err(v))),
                }
            }
        }
    };
}

deserialize_impl_negative_integer!(i8);
deserialize_impl_negative_integer!(i16);
deserialize_impl_negative_integer!(i32);
deserialize_impl_negative_integer!(i64);
deserialize_impl_negative_integer!(isize);

macro_rules! deserialize_impl_float {
    ($t:ty) => {
        impl<E> Deserr<E> for $t
        where
            E: DeserializeError,
        {
            fn deserialize_from_value<V: IntoValue>(
                value: Value<V>,
                location: ValuePointerRef,
            ) -> Result<Self, E> {
                match value {
                    Value::Integer(x) => Ok(x as $t),
                    Value::NegativeInteger(x) => Ok(x as $t),
                    Value::Float(x) => Ok(x as $t),
                    v => Err($crate::take_cf_content(E::error(
                        None,
                        ErrorKind::IncorrectValueKind {
                            actual: v,
                            accepted: &[
                                ValueKind::Float,
                                ValueKind::Integer,
                                ValueKind::NegativeInteger,
                            ],
                        },
                        location,
                    ))),
                }
            }
        }
    };
}
deserialize_impl_float!(f32);
deserialize_impl_float!(f64);

impl<E> Deserr<E> for String
where
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::String(x) => Ok(x),
            v => Err(take_cf_content(E::error(
                None,
                ErrorKind::IncorrectValueKind {
                    actual: v,
                    accepted: &[ValueKind::String],
                },
                location,
            ))),
        }
    }
}

impl<T, E> Deserr<E> for Vec<T>
where
    T: Deserr<E>,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Sequence(seq) => {
                let mut error = None;
                let mut vec = Vec::with_capacity(seq.len());
                for (index, value) in seq.into_iter().enumerate() {
                    let result =
                        T::deserialize_from_value(value.into_value(), location.push_index(index));
                    match result {
                        Ok(value) => {
                            vec.push(value);
                        }
                        Err(e) => {
                            error = match E::merge(error, e, location.push_index(index)) {
                                ControlFlow::Continue(e) => Some(e),
                                ControlFlow::Break(e) => return Err(e),
                            };
                        }
                    }
                }
                if let Some(e) = error {
                    Err(e)
                } else {
                    Ok(vec)
                }
            }
            v => Err(take_cf_content(E::error(
                None,
                ErrorKind::IncorrectValueKind {
                    actual: v,
                    accepted: &[ValueKind::Sequence],
                },
                location,
            ))),
        }
    }
}

impl<T, E> Deserr<E> for Option<T>
where
    T: Deserr<E>,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Null => Ok(None),
            value => T::deserialize_from_value(value, location).map(Some),
        }
    }
}

impl<T, E> Deserr<E> for Box<T>
where
    T: Deserr<E>,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        T::deserialize_from_value(value, location).map(Box::new)
    }
}

impl<Key, T, E> Deserr<E> for HashMap<Key, T>
where
    Key: FromStr + Hash + Eq,
    T: Deserr<E>,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Map(map) => {
                let mut error = None;
                let mut res = HashMap::with_capacity(map.len());
                for (string_key, value) in map.into_iter() {
                    match Key::from_str(&string_key) {
                        Ok(key) => {
                            match T::deserialize_from_value(
                                value.into_value(),
                                location.push_key(&string_key),
                            ) {
                                Ok(value) => {
                                    res.insert(key, value);
                                }
                                Err(e) => {
                                    error = match E::merge(error, e, location.push_key(&string_key))
                                    {
                                        ControlFlow::Continue(e) => Some(e),
                                        ControlFlow::Break(e) => return Err(e),
                                    };
                                }
                            }
                        }
                        Err(_) => {
                            error = match E::error::<V>(
                                error,
                                ErrorKind::Unexpected {
                                    msg: format!(
                                    "the key \"{string_key}\" could not be deserialized into the key type `{}`",
                                    std::any::type_name::<Key>())
                                },
                                location) {
                                    ControlFlow::Continue(e) => Some(e),
                                    ControlFlow::Break(e) => return Err(e),
                                };
                        }
                    }
                }
                Ok(res)
            }
            v => Err(take_cf_content(E::error(
                None,
                ErrorKind::IncorrectValueKind {
                    actual: v,
                    accepted: &[ValueKind::Map],
                },
                location,
            ))),
        }
    }
}

impl<Key, T, E> Deserr<E> for BTreeMap<Key, T>
where
    Key: FromStr + Ord,
    T: Deserr<E>,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Map(map) => {
                let mut error = None;
                let mut res = BTreeMap::new();
                for (string_key, value) in map.into_iter() {
                    match Key::from_str(&string_key) {
                        Ok(key) => {
                            match T::deserialize_from_value(
                                value.into_value(),
                                location.push_key(&string_key),
                            ) {
                                Ok(value) => {
                                    res.insert(key, value);
                                }
                                Err(e) => {
                                    error = match E::merge(error, e, location.push_key(&string_key))
                                    {
                                        ControlFlow::Continue(e) => Some(e),
                                        ControlFlow::Break(e) => return Err(e),
                                    };
                                }
                            }
                        }
                        Err(_) => {
                            error = match E::error::<V>(
                                error,
                                ErrorKind::Unexpected {
                                    msg: format!("the key \"{string_key}\" could not be deserialized into the key type `{}`",
                                    std::any::type_name::<Key>())
                                },
                                location
                            ) {
                                ControlFlow::Continue(e) => Some(e),
                                ControlFlow::Break(e) => return Err(e),
                            };
                        }
                    }
                }
                Ok(res)
            }
            v => Err(take_cf_content(E::error(
                None,
                ErrorKind::IncorrectValueKind {
                    actual: v,
                    accepted: &[ValueKind::Map],
                },
                location,
            ))),
        }
    }
}

impl<T, E> Deserr<E> for HashSet<T>
where
    T: Deserr<E> + Hash + Eq,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Sequence(seq) => {
                let mut error = None;
                let mut set = HashSet::with_capacity(seq.len());
                for (index, value) in seq.into_iter().enumerate() {
                    let result =
                        T::deserialize_from_value(value.into_value(), location.push_index(index));
                    match result {
                        Ok(value) => {
                            set.insert(value);
                        }
                        Err(e) => {
                            error = match E::merge(error, e, location.push_index(index)) {
                                ControlFlow::Continue(e) => Some(e),
                                ControlFlow::Break(e) => return Err(e),
                            };
                        }
                    }
                }
                if let Some(e) = error {
                    Err(e)
                } else {
                    Ok(set)
                }
            }
            v => Err(take_cf_content(E::error(
                None,
                ErrorKind::IncorrectValueKind {
                    actual: v,
                    accepted: &[ValueKind::Sequence],
                },
                location,
            ))),
        }
    }
}

impl<T, E> Deserr<E> for BTreeSet<T>
where
    T: Deserr<E> + Ord,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Sequence(seq) => {
                let mut error = None;
                let mut set = BTreeSet::new();
                for (index, value) in seq.into_iter().enumerate() {
                    let result =
                        T::deserialize_from_value(value.into_value(), location.push_index(index));
                    match result {
                        Ok(value) => {
                            set.insert(value);
                        }
                        Err(e) => {
                            error = match E::merge(error, e, location.push_index(index)) {
                                ControlFlow::Continue(e) => Some(e),
                                ControlFlow::Break(e) => return Err(e),
                            };
                        }
                    }
                }
                if let Some(e) = error {
                    Err(e)
                } else {
                    Ok(set)
                }
            }
            v => Err(take_cf_content(E::error(
                None,
                ErrorKind::IncorrectValueKind {
                    actual: v,
                    accepted: &[ValueKind::Sequence],
                },
                location,
            ))),
        }
    }
}

impl<A, B, E> Deserr<E> for (A, B)
where
    A: Deserr<E>,
    B: Deserr<E>,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Sequence(seq) => {
                let len = seq.len();
                if len != 2 {
                    return Err(take_cf_content(E::error::<V>(
                        None,
                        ErrorKind::Unexpected {
                            msg: String::from("the sequence should have exactly 2 elements"),
                        },
                        location,
                    )));
                }
                let mut error = None;
                let mut iter = seq.into_iter();

                let a = A::deserialize_from_value(
                    iter.next().unwrap().into_value(),
                    location.push_index(0),
                );
                let a = match a {
                    Ok(a) => Some(a),
                    Err(e) => {
                        error = match E::merge(error, e, location.push_index(0)) {
                            ControlFlow::Continue(e) => Some(e),
                            ControlFlow::Break(e) => return Err(e),
                        };
                        None
                    }
                };
                let b = B::deserialize_from_value(
                    iter.next().unwrap().into_value(),
                    location.push_index(1),
                );
                let b = match b {
                    Ok(b) => Some(b),
                    Err(e) => {
                        error = match E::merge(error, e, location.push_index(1)) {
                            ControlFlow::Continue(e) => Some(e),
                            ControlFlow::Break(e) => return Err(e),
                        };
                        None
                    }
                };

                if let Some(error) = error {
                    Err(error)
                } else {
                    Ok((a.unwrap(), b.unwrap()))
                }
            }
            v => Err(take_cf_content(E::error(
                None,
                ErrorKind::IncorrectValueKind {
                    actual: v,
                    accepted: &[ValueKind::Sequence],
                },
                location,
            ))),
        }
    }
}

impl<A, B, C, E> Deserr<E> for (A, B, C)
where
    A: Deserr<E>,
    B: Deserr<E>,
    C: Deserr<E>,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::Sequence(seq) => {
                let len = seq.len();
                if len != 2 {
                    return Err(take_cf_content(E::error::<V>(
                        None,
                        ErrorKind::Unexpected {
                            msg: String::from("the sequence should have exactly 2 elements"),
                        },
                        location,
                    )));
                }
                let mut error = None;
                let mut iter = seq.into_iter();

                let a = A::deserialize_from_value(
                    iter.next().unwrap().into_value(),
                    location.push_index(0),
                );
                let a = match a {
                    Ok(a) => Some(a),
                    Err(e) => {
                        error = match E::merge(error, e, location.push_index(0)) {
                            ControlFlow::Continue(e) => Some(e),
                            ControlFlow::Break(e) => return Err(e),
                        };
                        None
                    }
                };
                let b = B::deserialize_from_value(
                    iter.next().unwrap().into_value(),
                    location.push_index(1),
                );
                let b = match b {
                    Ok(b) => Some(b),
                    Err(e) => {
                        error = match E::merge(error, e, location.push_index(1)) {
                            ControlFlow::Continue(e) => Some(e),
                            ControlFlow::Break(e) => return Err(e),
                        };
                        None
                    }
                };
                let c = C::deserialize_from_value(
                    iter.next().unwrap().into_value(),
                    location.push_index(2),
                );
                let c = match c {
                    Ok(c) => Some(c),
                    Err(e) => {
                        error = match E::merge(error, e, location.push_index(2)) {
                            ControlFlow::Continue(e) => Some(e),
                            ControlFlow::Break(e) => return Err(e),
                        };
                        None
                    }
                };

                if let Some(error) = error {
                    Err(error)
                } else {
                    Ok((a.unwrap(), b.unwrap(), c.unwrap()))
                }
            }
            v => Err(take_cf_content(E::error(
                None,
                ErrorKind::IncorrectValueKind {
                    actual: v,
                    accepted: &[ValueKind::Sequence],
                },
                location,
            ))),
        }
    }
}
