use std::str::FromStr;

use serde_cs::vec::CS;

use crate::{
    take_cf_content, DeserializeError, Deserr, ErrorKind, IntoValue, Value, ValueKind,
    ValuePointerRef,
};

impl<R, E, FE> Deserr<E> for CS<R>
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
                Err(e) => Err(take_cf_content(E::error::<V>(
                    None,
                    ErrorKind::Unexpected { msg: e.to_string() },
                    location,
                ))),
            },
            value => Err(take_cf_content(E::error::<V>(
                None,
                ErrorKind::IncorrectValueKind {
                    actual: value,
                    accepted: &[ValueKind::String],
                },
                location,
            ))),
        }
    }
}
