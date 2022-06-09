use crate::{DeserializeError, DeserializeFromValue, IntoValue, Map, Sequence, Value, ValueKind};
use std::{
    collections::{BTreeMap, HashMap},
    convert::TryFrom,
    hash::Hash,
    str::FromStr,
};

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

impl<E: DeserializeError> DeserializeFromValue<E> for () {
    fn deserialize_from_value<V: IntoValue>(value: Value<V>) -> Result<Self, E> {
        match value {
            Value::Null => Ok(()),
            _ => Err(E::incorrect_value_kind(&[ValueKind::Null])),
        }
    }
}

impl<E: DeserializeError> DeserializeFromValue<E> for bool {
    fn deserialize_from_value<V: IntoValue>(value: Value<V>) -> Result<Self, E> {
        match value {
            Value::Boolean(b) => Ok(b),
            _ => Err(E::incorrect_value_kind(&[ValueKind::Boolean])),
        }
    }
}

macro_rules! deserialize_impl_integer {
    ($t:ty) => {
        impl<E: DeserializeError> DeserializeFromValue<E> for $t {
            fn deserialize_from_value<V: IntoValue>(value: Value<V>) -> Result<Self, E> {
                let err = || E::incorrect_value_kind(&[ValueKind::Integer]);

                match value {
                    Value::Integer(x) => <$t>::try_from(x).ok(),
                    Value::NegativeInteger(x) => <$t>::try_from(x).ok(),
                    _ => return Err(err()),
                }
                .ok_or_else(err)
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
        impl<E: DeserializeError> DeserializeFromValue<E> for $t {
            fn deserialize_from_value<V: IntoValue>(value: Value<V>) -> Result<Self, E> {
                let err =
                    || E::incorrect_value_kind(&[ValueKind::Integer, ValueKind::NegativeInteger]);

                match value {
                    Value::Integer(x) => <$t>::try_from(x).ok(),
                    Value::NegativeInteger(x) => <$t>::try_from(x).ok(),
                    _ => return Err(err()),
                }
                .ok_or_else(err)
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
        impl<E: DeserializeError> DeserializeFromValue<E> for $t {
            fn deserialize_from_value<V: IntoValue>(value: Value<V>) -> Result<Self, E> {
                match value {
                    Value::Integer(x) => {
                        return Ok(x as $t);
                    }
                    Value::NegativeInteger(x) => {
                        return Ok(x as $t);
                    }
                    Value::Float(x) => {
                        return Ok(x as $t);
                    }
                    _ => {
                        return Err(E::incorrect_value_kind(&[
                            ValueKind::Float,
                            ValueKind::Integer,
                            ValueKind::NegativeInteger,
                        ]))
                    }
                };
            }
        }
    };
}
deserialize_impl_float!(f32);
deserialize_impl_float!(f64);

impl<E: DeserializeError> DeserializeFromValue<E> for String {
    fn deserialize_from_value<V: IntoValue>(value: Value<V>) -> Result<Self, E> {
        match value {
            Value::String(x) => Ok(x),
            _ => Err(E::incorrect_value_kind(&[ValueKind::String])),
        }
    }
}

impl<T, E: DeserializeError> DeserializeFromValue<E> for Vec<T>
where
    T: DeserializeFromValue<E>,
{
    fn deserialize_from_value<V: IntoValue>(value: Value<V>) -> Result<Self, E> {
        match value {
            Value::Sequence(seq) => seq
                .into_iter()
                .map(V::into_value)
                .map(T::deserialize_from_value)
                .collect(),
            _ => Err(E::incorrect_value_kind(&[ValueKind::Sequence])),
        }
    }
}

impl<T, E: DeserializeError> DeserializeFromValue<E> for Option<T>
where
    T: DeserializeFromValue<E>,
{
    fn deserialize_from_value<V: IntoValue>(value: Value<V>) -> Result<Self, E> {
        match value {
            Value::Null => Ok(None),
            value => T::deserialize_from_value(value).map(Some),
        }
    }
    fn default() -> Option<Self> {
        Some(None)
    }
}

impl<T, E: DeserializeError> DeserializeFromValue<E> for Box<T>
where
    T: DeserializeFromValue<E>,
{
    fn deserialize_from_value<V: IntoValue>(value: Value<V>) -> Result<Self, E> {
        T::deserialize_from_value(value).map(Box::new)
    }
}

impl<Key, T, E: DeserializeError> DeserializeFromValue<E> for HashMap<Key, T>
where
    Key: FromStr + Hash + Eq,
    T: DeserializeFromValue<E>,
{
    fn deserialize_from_value<V: IntoValue>(value: Value<V>) -> Result<Self, E> {
        match value {
            Value::Map(map) => {
                let mut res = HashMap::with_capacity(map.len());
                for (key, value) in map.into_iter() {
                    let key = Key::from_str(&key).map_err(|_| E::unexpected("todo"))?;
                    let value = T::deserialize_from_value(value.into_value())?;
                    res.insert(key, value);
                }
                Ok(res)
            }
            _ => Err(E::incorrect_value_kind(&[ValueKind::Map])),
        }
    }
}

impl<Key, T, E: DeserializeError> DeserializeFromValue<E> for BTreeMap<Key, T>
where
    Key: FromStr + Ord,
    T: DeserializeFromValue<E>,
{
    fn deserialize_from_value<V: IntoValue>(value: Value<V>) -> Result<Self, E> {
        match value {
            Value::Map(map) => {
                let mut res = BTreeMap::new();
                for (key, value) in map.into_iter() {
                    let key = Key::from_str(&key).map_err(|_| E::unexpected("todo"))?;
                    let value = T::deserialize_from_value(value.into_value())?;
                    res.insert(key, value);
                }
                Ok(res)
            }
            _ => Err(E::incorrect_value_kind(&[ValueKind::Map])),
        }
    }
}

impl<A, B, E: DeserializeError> DeserializeFromValue<E> for (A, B)
where
    A: DeserializeFromValue<E>,
    B: DeserializeFromValue<E>,
{
    fn deserialize_from_value<V: IntoValue>(value: Value<V>) -> Result<Self, E> {
        match value {
            Value::Sequence(seq) => {
                let len = seq.len();
                if len < 2 {
                    return Err(E::unexpected("todo"));
                }
                if len > 2 {
                    return Err(E::unexpected("todo"));
                }
                let mut iter = seq.into_iter();

                let a = A::deserialize_from_value(iter.next().unwrap().into_value())?;
                let b = B::deserialize_from_value(iter.next().unwrap().into_value())?;

                Ok((a, b))
            }
            _ => Err(E::incorrect_value_kind(&[ValueKind::Sequence])),
        }
    }
}

impl<A, B, C, E: DeserializeError> DeserializeFromValue<E> for (A, B, C)
where
    A: DeserializeFromValue<E>,
    B: DeserializeFromValue<E>,
    C: DeserializeFromValue<E>,
{
    fn deserialize_from_value<V: IntoValue>(value: Value<V>) -> Result<Self, E> {
        match value {
            Value::Sequence(seq) => {
                let len = seq.len();
                if len < 3 {
                    return Err(E::unexpected("todo"));
                }
                if len > 3 {
                    return Err(E::unexpected("todo"));
                }
                let mut iter = seq.into_iter();

                let a = A::deserialize_from_value(iter.next().unwrap().into_value())?;
                let b = B::deserialize_from_value(iter.next().unwrap().into_value())?;
                let c = C::deserialize_from_value(iter.next().unwrap().into_value())?;

                Ok((a, b, c))
            }
            _ => Err(E::incorrect_value_kind(&[ValueKind::Sequence])),
        }
    }
}
