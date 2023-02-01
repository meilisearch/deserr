use deserr::{
    deserialize, take_cf_content, DeserializeError, Deserr, ErrorKind, JsonError, QueryParamError,
    Sequence, Value, ValueKind,
};
use serde_json::json;

#[allow(dead_code)]
#[derive(Debug, Deserr)]
#[deserr(deny_unknown_fields)]
struct Query {
    name: String,
    filter: Filter,
}

/// A filter is a recursive structure, but it's always composed of arrays or direct value.
/// Thus a valid filter can be on of the following:
/// - `"jorts"`
/// - `["jorts", "jean"]`
/// - `["jorts", ["bilbo", "bob"], "jean"]`
#[derive(Debug)]
enum Filter {
    Array(Vec<Filter>),
    Direct(String),
}

impl<E: DeserializeError> Deserr<E> for Filter {
    fn deserialize_from_value<V: deserr::IntoValue>(
        value: deserr::Value<V>,
        location: deserr::ValuePointerRef,
    ) -> Result<Self, E> {
        match value {
            Value::String(s) => Ok(Filter::Direct(s)),
            Value::Sequence(seq) => Ok(Filter::Array(
                seq.into_iter()
                    .enumerate()
                    .map(|(index, value)| {
                        Self::deserialize_from_value(value.into_value(), location.push_index(index))
                    })
                    .collect::<Result<Vec<Self>, E>>()?,
            )),
            value => {
                // we're not interested in the `ControlFlow`, we just want to use the error message defined
                // by the error type.
                Err(take_cf_content(E::error(
                    None,
                    ErrorKind::IncorrectValueKind {
                        actual: value,
                        accepted: &[ValueKind::String, ValueKind::Sequence],
                    },
                    location,
                )))
            }
        }
    }
}

fn main() {
    let filter = deserialize::<Filter, serde_json::Value, JsonError>(json!("jorts")).unwrap();
    insta::assert_debug_snapshot!(filter, @r###"
    Direct(
        "jorts",
    )
    "###);

    // As you can see, we're effectively able to deserialize all kind of valid filters.
    let filter = deserialize::<Filter, _, JsonError>(json!([
        "jorts",
        "the",
        ["most", ["famous", "catto"], "in", "the"],
        "world"
    ]))
    .unwrap();
    insta::assert_debug_snapshot!(filter, @r###"
    Array(
        [
            Direct(
                "jorts",
            ),
            Direct(
                "the",
            ),
            Array(
                [
                    Direct(
                        "most",
                    ),
                    Array(
                        [
                            Direct(
                                "famous",
                            ),
                            Direct(
                                "catto",
                            ),
                        ],
                    ),
                    Direct(
                        "in",
                    ),
                    Direct(
                        "the",
                    ),
                ],
            ),
            Direct(
                "world",
            ),
        ],
    )
    "###);

    // And when an error arise, we get the nice error from the json error handler.
    let error = deserialize::<Filter, _, JsonError>(json!(["jorts", "is", ["a", 10]])).unwrap_err();
    insta::assert_display_snapshot!(error, @"Invalid value type at `[2][1]`: expected a string or an array, but found a positive integer: `10`");

    // But since we're generic over the error type we can as well switch to query parameter error!
    let error =
        deserialize::<Filter, _, QueryParamError>(json!(["jorts", "is", "a", 10])).unwrap_err();
    insta::assert_display_snapshot!(error, @"Invalid value type for parameter `[3]`: expected a string, but found an integer: `10`");

    // And as expected, using this `Filter` type from another struct that got its `Deserr` implementation from the derive macro just works.
    let filter =
        deserialize::<Query, _, JsonError>(json!({ "name": "jorts", "filter": "catto" })).unwrap();
    insta::assert_debug_snapshot!(filter, @r###"
    Query {
        name: "jorts",
        filter: Direct(
            "catto",
        ),
    }
    "###);
}
