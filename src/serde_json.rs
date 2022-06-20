use crate::{
    DeserializeError, DeserializeFromValue, IntoValue, Map, Sequence, Value, ValueKind,
    ValuePointerRef,
};
use serde_json::{Map as JMap, Number, Value as JValue};

impl Map for JMap<String, JValue> {
    type Value = JValue;
    type Iter = <Self as IntoIterator>::IntoIter;

    fn len(&self) -> usize {
        self.len()
    }
    fn remove(&mut self, key: &str) -> Option<Self::Value> {
        self.remove(key)
    }
    fn into_iter(self) -> Self::Iter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl IntoValue for JValue {
    type Sequence = Vec<JValue>;
    type Map = JMap<String, JValue>;

    fn into_value(self) -> Value<Self> {
        match self {
            JValue::Null => Value::Null,
            JValue::Bool(b) => Value::Boolean(b),
            JValue::Number(n) => {
                if let Some(n) = n.as_u64() {
                    Value::Integer(n)
                } else if let Some(n) = n.as_i64() {
                    Value::NegativeInteger(n)
                } else if let Some(n) = n.as_f64() {
                    Value::Float(n)
                } else {
                    panic!();
                }
            }
            JValue::String(x) => Value::String(x),
            JValue::Array(x) => Value::Sequence(x),
            JValue::Object(x) => Value::Map(x),
        }
    }

    fn kind(&self) -> ValueKind {
        match self {
            JValue::Null => ValueKind::Null,
            JValue::Bool(_) => ValueKind::Boolean,
            JValue::Number(n) => {
                if n.is_u64() {
                    ValueKind::Integer
                } else if n.is_i64() {
                    ValueKind::NegativeInteger
                } else if n.is_f64() {
                    ValueKind::Float
                } else {
                    panic!();
                }
            }
            JValue::String(_) => ValueKind::String,
            JValue::Array(_) => ValueKind::Sequence,
            JValue::Object(_) => ValueKind::Map,
        }
    }
}

impl<E: DeserializeError> DeserializeFromValue<E> for JValue {
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E> {
        let mut error: Option<E> = None;
        Ok(match value {
            Value::Null => JValue::Null,
            Value::Boolean(b) => JValue::Bool(b),
            Value::Integer(x) => JValue::Number(Number::from(x)),
            Value::NegativeInteger(x) => JValue::Number(Number::from(x)),
            Value::Float(f) => match Number::from_f64(f) {
                Some(n) => JValue::Number(n),
                None => {
                    return Err(E::unexpected(
                        error,
                        &format!("The float {f} is not representable in JSON"),
                        location,
                    )?);
                }
            },
            Value::String(s) => JValue::String(s),
            Value::Sequence(seq) => {
                let mut jseq = Vec::with_capacity(seq.len());
                for (index, value) in seq.into_iter().enumerate() {
                    let result = Self::deserialize_from_value(
                        value.into_value(),
                        location.push_index(index),
                    );
                    match result {
                        Ok(value) => {
                            jseq.push(value);
                        }
                        Err(e) => {
                            error = Some(E::merge(error, e, location.push_index(index))?);
                        }
                    }
                }
                if let Some(e) = error {
                    return Err(e);
                } else {
                    JValue::Array(jseq)
                }
            }
            Value::Map(map) => {
                let mut jmap = JMap::with_capacity(map.len());
                for (key, value) in map.into_iter() {
                    let result =
                        Self::deserialize_from_value(value.into_value(), location.push_key(&key));
                    match result {
                        Ok(value) => {
                            jmap.insert(key, value);
                        }
                        Err(e) => {
                            error = Some(E::merge(error, e, location.push_key(&key))?);
                        }
                    }
                }
                if let Some(e) = error {
                    return Err(e);
                } else {
                    JValue::Object(jmap)
                }
            }
        })
    }
}
