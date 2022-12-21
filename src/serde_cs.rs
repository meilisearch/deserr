use std::str::FromStr;

use serde_cs::vec::CS;

use crate::{
    DeserializeError, DeserializeFromValue, ErrorKind, IntoValue, Value, ValueKind, ValuePointerRef,
};

impl<R, E, FE> DeserializeFromValue<E> for CS<R>
where
    R: FromStr<Err = FE>,
    FE: std::error::Error,
    E: DeserializeError,
{
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::String(s) => match CS::from_str(&s) {
                Ok(ret) => Ok(ret),
                Err(e) => Err(E::error::<V>(
                    None,
                    ErrorKind::Unexpected { msg: e.to_string() },
                    location,
                )?),
            },
            value => Err(E::error::<V>(
                None,
                ErrorKind::IncorrectValueKind {
                    actual: value,
                    accepted: &[ValueKind::String],
                },
                location,
            )?),
        }
    }
}
