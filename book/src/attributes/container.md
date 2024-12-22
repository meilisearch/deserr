# Container attributes

### `#[deserr(rename_all = ...)]`

Rename all the fields (if this is a struct) or variants (if this is an enum) according to the given case convention.
The possible values are: `lowercase`, `camelCase`.

If you need more values please open an issue, it's easy to implement and was simply not implemented because it isn't required for Meilisearch at the moment.

<div class="warning">

Unlike `serde`, you don't need to put the double-quotes (`"`) around the name of the case, e.g.: `#[deserr(rename_all = camelCase)]`.

</div>

```rust
use deserr::{Deserr, deserialize, errors::JsonError};
use serde_json::json;

#[derive(Deserr, Debug, PartialEq, Eq)]
#[deserr(rename_all = camelCase)]
struct Search {
    query: String,
    attributes_to_retrieve: Vec<String>,
}

let data = deserialize::<Search, _, JsonError>(
    json!({ "query": "doggo", "attributesToRetrieve": ["age", "name"] }),
)
.unwrap();
assert_eq!(data, Search {
    query: String::from("doggo"),
    attributes_to_retrieve: vec![String::from("age"), String::from("name")],
});
```

### `#[deserr(deny_unknown_fields)]`

Always error during deserialization when encountering unknown fields.
When this attribute is not present, by default unknown fields are silently ignored.

```rust
use deserr::{Deserr, deserialize, errors::JsonError};
use serde_json::json;

#[derive(Deserr, Debug)]
#[deserr(deny_unknown_fields)]
struct Search {
    query: String,
}

let err = deserialize::<Search, _, JsonError>(
    json!({ "query": "doggo", "doggo": "bork" }),
)
.unwrap_err();

assert_eq!(err.to_string(), "Unknown field `doggo`: expected one of `query`");
```


<div class="warning">

Unlike `serde`, with `deserr` you can specify provide a custom function to handle the error.

```rust
use deserr::{Deserr, deserialize, ErrorKind, DeserializeError, ValuePointerRef, take_cf_content, errors::JsonError};
use std::convert::Infallible;
use serde_json::json;

#[derive(Deserr, Debug)]
#[deserr(deny_unknown_fields = unknown_fields_search)]
struct Search {
    query: String,
}

fn unknown_fields_search<E: DeserializeError>(
    field: &str,
    accepted: &[&str],
    location: ValuePointerRef,
) -> E {
    // `E::error` returns a `ControlFlow<E, E>`, which returns the error and indicates
    // whether we should keep accumulating errors or not. However, here we simply
    // want to retrieve the error's value. This is what `take_cf_content` does.
    match field {
        "doggo" => take_cf_content(E::error::<Infallible>(
                None,
                ErrorKind::Unexpected {
                    msg: String::from("can I pet the doggo? uwu")
                },
                location,
            )),
        _ => take_cf_content(E::error::<Infallible>(
            None,
            deserr::ErrorKind::UnknownKey { key: field, accepted },
            location,
        )),
    }
}

let err = deserialize::<Search, _, JsonError>(
    json!({ "query": "doggo", "doggo": "bork" }),
)
.unwrap_err();

assert_eq!(err.to_string(), "Invalid value: can I pet the doggo? uwu");

let err = deserialize::<Search, _, JsonError>(
    json!({ "query": "doggo", "catto": "jorts" }),
)
.unwrap_err();

assert_eq!(err.to_string(), "Unknown field `catto`: expected one of `query`");
```

</div>

### `#[deserr(tag)]`

Externally tag an enum.

<div class="warning">

Deserr does not support internally tagging your enum yet, which means you'll always need to use this attribute if you're deserializing an enum.

</div>

For complete unit enums, deserr can deserialize their value from a string, though.

```rust
use deserr::{Deserr, deserialize, errors::JsonError};
use serde_json::json;

#[derive(Deserr, Debug, PartialEq, Eq)]
struct Search {
    query: Query,
}

#[derive(Deserr, Debug, PartialEq, Eq)]
#[deserr(tag = "type")]
enum Query {
    Single {
        search: String,
    },
    Multi {
        searches: Vec<String>,
    }
}

let data = deserialize::<Search, _, JsonError>(
    json!({ "query": { "type": "Single", "search": "bork" } }),
)
.unwrap();
assert_eq!(data, Search {
    query: Query::Single {
        search: String::from("bork"),
    },
});
```

### `#[deserr(from)]`

Deserializing a type from a function instead of a `Value`.
You need to provide the following information;
1. The input type of the function (here `&String`)
2. The path of the function (here, we're simply using the std `FromStr` implementation)

deserr will first try to deserialize the given type using its `Deserr<E>` implementation.
That means the input type of the `from` can be complex. Then deserr will call your
function.

- [If your function can fail, consider using `try_from` instead](#deserrtryfrom)
- [The field attribute may interests you as well](field.md#deserrfrom)

```rust
use deserr::{Deserr, deserialize, errors::JsonError};
use serde_json::json;

#[derive(Deserr, Debug, PartialEq, Eq)]
#[deserr(from(String) = From::from)]
enum Wildcard {
    Wildcard,
    Value(String),
}

impl From<String> for Wildcard {
    fn from(s: String) -> Self {
        if s == "*" {
            Wildcard::Wildcard
        } else {
            Wildcard::Value(s)
        }
    }
}

let data = deserialize::<Wildcard, _, JsonError>(
    json!("doggo"),
)
.unwrap();
assert_eq!(data, Wildcard::Value(String::from("doggo")));

let data = deserialize::<Wildcard, _, JsonError>(
    json!("*"),
)
.unwrap();
assert_eq!(data, Wildcard::Wildcard);
```

### `#[deserr(try_from)]`

Try deserializing a type from a function instead of a `Value`.
You need to provide the following information;
1. The input type of the function (here `&String`)
2. The path of the function (here, we're simply using the std `FromStr` implementation)
3. The error type that this function can return (here `AsciiStringError`)

deserr will first try to deserialize the given type using its `Deserr<E>` implementation.
That means the input type of the `try_from` can be complex. Then deserr will call your
function and accumulate the specified error against the error type of the caller.

- [If your function cannot fail, consider using `from` directly](#deserrfrom)
- [The field attribute may interests you as well](field.md#deserrtryfrom)

```rust
use deserr::{Deserr, deserialize, errors::JsonError};
use serde_json::json;
use std::str::FromStr;
use std::fmt;

// Notice how the `try_from` allows us to leverage the deserr limitation on tuple struct.
#[derive(Deserr, Debug, PartialEq, Eq)]
#[deserr(try_from(&String) = FromStr::from_str -> AsciiStringError)]
struct AsciiString(String);

#[derive(Debug)]
struct AsciiStringError(char);

impl fmt::Display for AsciiStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Encountered invalid character: `{}`, only ascii characters are accepted",
            self.0
        )
    }
}
impl std::error::Error for AsciiStringError {}

impl FromStr for AsciiString {
    type Err = AsciiStringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(c) = s.chars().find(|c| !c.is_ascii()) {
            Err(AsciiStringError(c))
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

let data = deserialize::<AsciiString, _, JsonError>(
    json!("doggo"),
)
.unwrap();
assert_eq!(data, AsciiString(String::from("doggo")));

let error = deserialize::<AsciiString, _, JsonError>(
    json!("👉👈"),
)
.unwrap_err();
assert_eq!(error.to_string(), "Invalid value: Encountered invalid character: `👉`, only ascii characters are accepted");
```

### `#[deserr(validate)]`

Validate a structure **after** it has been deserialized.
This is typically useful when your validation logic needs to take multiple fields into account.

```rust
use deserr::{Deserr, DeserializeError, ErrorKind, ValuePointerRef, deserialize, errors::JsonError};
use serde_json::json;
use std::convert::Infallible;

// `__Deserr_E` represents the Error returned by the generated `Deserr` implementation.
#[derive(Deserr, Debug, PartialEq, Eq)]
#[deserr(validate = validate_range -> __Deserr_E)]
struct Range {
    min: u8,
    max: u8,
}

fn validate_range<E: DeserializeError>(
    range: Range,
    location: ValuePointerRef,
) -> Result<Range, E> {
    if range.min > range.max {
        Err(deserr::take_cf_content(E::error::<Infallible>(
            None,
            ErrorKind::Unexpected {
                msg: format!(
                    "`max` (`{}`) should be greater than `min` (`{}`)",
                    range.max, range.min
                ),
            },
            location,
        )))
    } else {
        Ok(range)
    }
}

let data = deserialize::<Range, _, JsonError>(
    json!({ "min": 2, "max": 4 }),
)
.unwrap();
assert_eq!(data, Range { min: 2, max: 4 });

let error = deserialize::<Range, _, JsonError>(
    json!({ "min": 4, "max": 2 }),
)
.unwrap_err();
assert_eq!(error.to_string(), "Invalid value: `max` (`2`) should be greater than `min` (`4`)");
```

### `#[deserr(error)]`

Customize the error type that can be returned when deserializing this structure
instead of keeping it generic.

```rust
use deserr::{Deserr, DeserializeError, ValuePointerRef, ErrorKind, deserialize, errors::JsonError};
use serde_json::json;

#[derive(Deserr, Debug, PartialEq, Eq)]
#[deserr(error = JsonError)]
struct Search {
    query: String,
    limit: usize,
}

// As we can see, rust is able to infer the error type.
let data = deserialize::<Search, _, _>(
    json!({ "query": "doggo", "limit": 1 }),
)
.unwrap();
assert_eq!(data, Search { query: String::from("doggo"), limit: 1 });
```

### `#[deserr(where_predicate)]`

Let you add `where` clauses to the `Deserr` implementation that deserr will generate.

```rust
use deserr::{Deserr, DeserializeError, MergeWithError, deserialize, errors::JsonError};
use serde_json::json;

// Here we can constraint the generic `__Deserr_E` type used by deserr to implements `MergeWithError`.
// Now instead of constraining the final error type it stays generic if it's able to accumulate with
// with a `JsonError`.
#[derive(Deserr, Debug, PartialEq, Eq)]
#[deserr(where_predicate = __Deserr_E: MergeWithError<JsonError>, where_predicate = A: Deserr<JsonError>)]
struct Search<A> {
    #[deserr(error = JsonError)]
    query: A,
    limit: usize,
}
```

[For simple cases, see also the `needs_predicate` field attribute.](field.md#deserrneedspredicate)
