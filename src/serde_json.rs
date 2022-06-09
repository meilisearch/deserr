use crate::{IntoValue, Map, Value, ValueKind};
use serde_json::{Map as JMap, Value as JValue};

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
