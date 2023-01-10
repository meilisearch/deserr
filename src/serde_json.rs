use std::fmt::Display;

use crate::{
    DeserializeError, DeserializeFromValue, ErrorKind, IntoValue, Map, MergeWithError, Sequence,
    Value, ValueKind, ValuePointerRef,
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
                    return Err(E::error::<V>(
                        error,
                        ErrorKind::Unexpected {
                            msg: format!("the float {f} is not representable in JSON"),
                        },
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

impl<V: IntoValue> From<Value<V>> for JValue {
    fn from(value: Value<V>) -> Self {
        match value {
            Value::Null => JValue::Null,
            Value::Boolean(b) => JValue::Bool(b),
            Value::Integer(n) => JValue::Number(Number::from(n)),
            Value::NegativeInteger(i) => JValue::Number(Number::from(i)),
            // if we can't parse the float then its set to `null`
            Value::Float(f) => Number::from_f64(f)
                .map(JValue::Number)
                .unwrap_or(JValue::Null),
            Value::String(s) => JValue::String(s),
            Value::Sequence(s) => JValue::Array(
                s.into_iter()
                    .map(IntoValue::into_value)
                    .map(JValue::from)
                    .collect(),
            ),
            Value::Map(m) => m
                .into_iter()
                .map(|(k, v)| (k, JValue::from(v.into_value())))
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct JsonError(pub String);

impl Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl MergeWithError<JsonError> for JsonError {
    fn merge(
        _self_: Option<Self>,
        other: JsonError,
        _merge_location: ValuePointerRef,
    ) -> Result<Self, Self> {
        Err(other)
    }
}

impl DeserializeError for JsonError {
    fn error<V: IntoValue>(
        _self_: Option<Self>,
        error: ErrorKind<V>,
        location: ValuePointerRef,
    ) -> Result<Self, Self> {
        let location = location.as_json();

        match error {
            ErrorKind::IncorrectValueKind { actual, accepted } => {
                let expected = match accepted.len() {
                    0 => String::new(),
                    1 => format!(", expected a {}", accepted[0]),
                    _ => format!(
                        ", expected one of {}",
                        accepted
                            .iter()
                            .map(|accepted| accepted.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    ),
                };

                let kind = actual.kind();
                // if we're not able to get the value as a string then we print nothing.
                let received = match serde_json::to_string(&serde_json::Value::from(actual)) {
                    Ok(value) => format!("`{}`", value),
                    Err(_) => String::new(),
                };

                let format = format!("invalid type: {kind} {received}{expected} at `{location}`.",);
                Err(JsonError(format))
            }
            ErrorKind::MissingField { field } => {
                // serde_json original message:
                // Json deserialize error: missing field `lol` at line 1 column 2

                Err(JsonError(format!(
                    "Json deserialize error: missing field `{field}` at `{location}`"
                )))
            }
            ErrorKind::UnknownKey { key, accepted } => {
                let format = format!(
                    "Json deserialize error: unknown field `{}`, expected one of {} at `{}`.",
                    key,
                    accepted
                        .iter()
                        .map(|accepted| format!("`{}`", accepted))
                        .collect::<Vec<String>>()
                        .join(", "),
                    location
                );

                Err(JsonError(format))
            }
            ErrorKind::Unexpected { msg } => {
                // serde_json original message:
                // The json payload provided is malformed. `trailing characters at line 1 column 19`.
                Err(JsonError(format!("{msg} at `{location}`.")))
            }
        }
    }
}

impl ValuePointerRef<'_> {
    // if the error happened in the root, then an empty string is returned.
    pub fn as_json(&self) -> String {
        match self {
            ValuePointerRef::Origin => String::new(),
            ValuePointerRef::Key { key, prev } => prev.as_json() + "." + key,
            ValuePointerRef::Index { index, prev } => format!("{}[{index}]", prev.as_json()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn from_value_to_deserr_and_back() {
        let value = json!({ "The": "best", "doggos": ["are"], "the": { "bernese": "mountain" }});
        let deserr = value.clone().into_value();

        insta::assert_debug_snapshot!(deserr, @r###"
        Map(
            {
                "The": String("best"),
                "doggos": Array [
                    String("are"),
                ],
                "the": Object {
                    "bernese": String("mountain"),
                },
            },
        )
        "###);

        let deserr: JValue = deserr.into();
        insta::assert_debug_snapshot!(deserr, @r###"
        Object {
            "The": String("best"),
            "doggos": Array [
                String("are"),
            ],
            "the": Object {
                "bernese": String("mountain"),
            },
        }
        "###);

        assert_eq!(value, deserr);
    }

    #[test]
    fn error_msg_missing_field() {
        #[allow(dead_code)]
        #[derive(deserr::DeserializeFromValue, Debug)]
        struct Missing {
            me: usize,
        }
        let value = json!({ "toto": 2 });
        let err = deserr::deserialize::<Missing, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Json deserialize error: missing field `me` at ``");
    }

    #[test]
    fn error_msg_incorrect() {
        #[allow(dead_code)]
        #[derive(deserr::DeserializeFromValue, Debug)]
        struct Incorrect {
            me: usize,
        }
        let value = json!({ "me": [2] });
        let err = deserr::deserialize::<Incorrect, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"invalid type: Sequence `[2]`, expected a Integer at `.me`.");

        #[allow(dead_code)]
        #[derive(deserr::DeserializeFromValue, Debug)]
        enum Variants {
            One,
            Two,
            Three,
        }

        #[allow(dead_code)]
        #[derive(deserr::DeserializeFromValue, Debug)]
        struct MultiIncorrect {
            me: Variants,
        }
        let value = json!({ "me": "la" });
        let err = deserr::deserialize::<MultiIncorrect, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Incorrect tag value at `.me`.");
    }

    #[test]
    fn error_msg_unknown_key() {
        #[allow(dead_code)]
        #[derive(deserr::DeserializeFromValue, Debug)]
        #[deserr(deny_unknown_fields)]
        struct SingleUnknownField {
            me: usize,
        }
        let value = json!({ "me": 2, "u": "uwu" });
        let err = deserr::deserialize::<SingleUnknownField, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Json deserialize error: unknown field `u`, expected one of `me` at ``.");

        #[allow(dead_code)]
        #[derive(deserr::DeserializeFromValue, Debug)]
        #[deserr(deny_unknown_fields)]
        struct MultiUnknownField {
            me: usize,
            and: String,
        }
        let value = json!({ "me": 2, "and": "u", "uwu": "OwO" });
        let err = deserr::deserialize::<MultiUnknownField, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"Json deserialize error: unknown field `uwu`, expected one of `me`, `and` at ``.");
    }

    #[test]
    fn error_msg_unexpected() {
        #[allow(dead_code)]
        #[derive(deserr::DeserializeFromValue, Debug)]
        #[deserr(deny_unknown_fields)]
        struct UnexpectedTuple {
            me: (usize, String),
        }
        let value = json!({ "me": [2] });
        let err = deserr::deserialize::<UnexpectedTuple, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"the sequence should have exactly 2 elements at `.me`.");

        let value = json!({ "me": [2, 3, 4] });
        let err = deserr::deserialize::<UnexpectedTuple, _, JsonError>(value).unwrap_err();
        insta::assert_display_snapshot!(err, @"the sequence should have exactly 2 elements at `.me`.");
    }
}
