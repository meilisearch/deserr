# Deserr

## Introduction

Deserr is a crate for deserializing data, with the ability to return
custom, type-specific errors upon failure. It was also designed with
user-facing APIs in mind and thus provides better defaults than serde for
this use case.

Unlike serde, deserr does not parse the data in its serialization format itself
but offloads the work to other crates. Instead, it deserializes
the already-parsed serialized data into the final type. For example:

```rust,ignore
// bytes of the serialized value
let s: &str = ".." ;
// parse serialized data using another crate, such as `serde_json`
let json: serde_json::Value = serde_json::from_str(s).unwrap();
// finally deserialize with deserr
let data = T::deserialize_from_value(json.into_value()).unwrap();
// `T` must implement `Deserr`.
```

## Why would I use it

The main place where you should use deserr is on your user-facing API,
especially if it's supposed to be read by a human.
Since deserr gives you full control over your error types, you can improve
the quality of your error messages.
Here is a little preview of what you can do with deserr:


Let's say I sent this payload to update my [Meilisearch](https://docs.meilisearch.com/reference/api/settings.html#settings) settings:
```json
{
  "filterableAttributes": ["doggo.age", "catto.age"],
  "sortableAttributes": ["uploaded_at"],
  "typoTolerance": {
    "minWordSizeForTypos": {
      "oneTypo": 1000, "twoTypo": 80
    },
    "enabled": true
  },
  "displayedAttributes": ["*"],
  "searchableAttributes": ["doggo.name", "catto.name"]
}
```

#### With serde

With serde, we don't have much customization; this is the typical kind of message we would get in return:

```json
{
  "message": "Json deserialize error: invalid value: integer `1000`, expected u8 at line 6 column 21",
  "code": "bad_request",
  "type": "invalid_request",
  "link": "https://docs.meilisearch.com/errors#bad_request"
}
```

##### The message

> Json deserialize error: invalid value: integer `1000`, expected u8 at line 6 column 21

- The message uses the word `u8`, which definitely won't help a user who doesn't know rust or is unfamiliar with types.
- The location is provided in terms of lines and columns. While this is generally good, when most of your users
  read this message in their terminal, it doesn't actually help much.

#### The rest of the payload

Since serde returned this error, we cannot know what happened or on which field it happened. Thus, the best we
can do is generate a code `bad_request` that is common for our whole API. We then use this code to generate
a link to our documentation to help our users. But such a generic link does not help our users because it
can be thrown by every single route of Meilisearch.

#### With deserr

```json
{
  "message": "Invalid value at `.typoTolerance.minWordSizeForTypos.oneTypo`: value: `1000` is too large to be deserialized, maximum value authorized is `255`",
  "code": "invalid_settings_typo_tolerance",
  "type": "invalid_request",
  "link": "https://docs.meilisearch.com/errors#invalid-settings-typo-tolerance"
}
```

##### The message

> Invalid value at `.typoTolerance.minWordSizeForTypos.oneTypo`: value: `1000` is too large to be deserialized, maximum value authorized is `255`

- We get a more human-readable location; `.typoTolerance.minWordSizeForTypos.oneTypo`. It gives us the faulty field.
- We also get a non-confusing and helpful message this time; it explicitly tells us that the maximum value authorized is `255`.

##### The rest of the payload

Since deserr called one of our functions in the process, we were able to use a custom error code + link to redirect
our user to the documentation specific to this feature and this field.

#### More possibilities with deserr that were impossible with serde

##### Adding constraints on multiples fields

In Meilisearch, there is another constraint on this `minWordSizeForTypos`, the `twoTypo` field **must be** greater than
the `oneType` field.

Serde doesn't provide any feature to do that. You could write your own implementation of `Deserialize` for the
entire sub-object `minWordSizeForTypos`, but that's generally hard and wouldn't even let you customize the
error type.
Thus, that's the kind of thing you're going to check by hand in your code later on. This is error-prone and
may bring inconsistencies between most of the deserialization error messages and your error message.

With deserr, we provide attributes that allow you to validate your structure once it's deserialized.

##### When a field is missing

It's possible to provide your own function when a field is missing.

```rust,ignore
pub fn missing_field<E: DeserializeError>(field: &str, location: ValuePointerRef) -> E {
    todo!()
}
```

At Meilisearch, we use this function to specify a custom error code, but we keep the default error message which is pretty accurate.

##### When an unknown field is encountered

It's possible to provide your own function when a field is missing.

```rust,ignore
fn unknown_field<E: DeserializeError>(
    field: &str,
    accepted: &[&str],
    location: ValuePointerRef,
) -> E {
    todo!()
}
```

Here is a few ideas we have or would like to implement at Meilisearch;
- In the case of a resource you can `PUT` with some fields, but can't `PATCH` all its fields. We can throw a special `immutable field x` error instead of an `unknown field x`.
- Detecting when you use the field name of an alternative; for example, we use `q` to make a `query` while some Meilisearch alternatives use `query`.
  We could help our users with a `did you mean?` message that corrects the field to its proper name in Meilisearch.
- Trying to guess what the user was trying to say by computing the [levenstein distance](https://en.wikipedia.org/wiki/Levenshtein_distance)
  between what the user typed and what is accepted to provide a `did you mean?` message that attempts to correct typos.

##### When multiple errors are encountered

Deserr lets you accumulate multiple errors with its `MergeWithError` trait while trying to deserialize the value into your type.
This is a good way to improve your user experience by reducing the number of interactions
a user needs to have to fix an invalid payload.

-----------

The main parts of deserr are:
1. `Deserr<E>` is the main trait for deserialization, unlike Serde, it's very easy to deserialize this trait manually, see the `implements_deserr_manually.rs` file in our examples directory.
2. `IntoValue` and `Value` describes the shape that the parsed serialized data must have
3. `DeserializeError` is the trait that all deserialization errors must conform to
4. `MergeWithError<E>` describe how to combine multiple errors together. It allows deserr
to return multiple deserialization errors at once.
5. `ValuePointerRef` and `ValuePointer` point to locations within the value. They are
used to locate the origin of an error.
6. `deserialize<Ret, Val, E>` is the main function to use to deserialize a value.
    - `Ret` is the returned value or the structure you want to deserialize.
    - `Val` is the value type you want to deserialize from. Currently, only an implementation for `serde_json::Value` is provided
        in this crate, but you could add your own! Feel free to look into our `serde_json` module.
    - `E` is the error type that should be used if an error happens during the deserialization.
7. The `Deserr` derive proc macro

## Example

### Implementing deserialize for a custom type with a custom error

In the following example, we're going to deserialize a structure containing a bunch of fields and 
uses a custom error type that accumulates all the errors encountered while deserializing the structure.

```rust
use deserr::{deserialize, DeserializeError, Deserr, ErrorKind, errors::JsonError, Value, ValueKind, IntoValue, take_cf_content, MergeWithError, ValuePointerRef, ValuePointer};
use serde_json::json;
use std::str::FromStr;
use std::ops::ControlFlow;
use std::fmt;
use std::convert::Infallible;

/// This is our custom error type. It'll accumulate multiple `JsonError`.
#[derive(Debug)]
struct MyError(Vec<JsonError>);

impl DeserializeError for MyError {
    /// Create a new error with the custom message.
    ///
    /// Return `ControlFlow::Continue` to continue deserializing even though an error was encountered.
    /// We could return `ControlFlow::Break` as well to stop right here.
    fn error<V: IntoValue>(self_: Option<Self>, error: ErrorKind<V>, location: ValuePointerRef) -> ControlFlow<Self, Self> {
        /// The `take_cf_content` return the inner error in a `ControlFlow<E, E>`.
        let error = take_cf_content(JsonError::error(None, error, location));

        let errors = if let Some(MyError(mut errors)) = self_ {
            errors.push(error);
            errors
        } else {
            vec![error]
        };
        ControlFlow::Continue(MyError(errors))
    }
}

/// We have to implements `MergeWithError` between our error type _aaand_ our error type.
impl MergeWithError<MyError> for MyError {
    fn merge(self_: Option<Self>, mut other: MyError, _merge_location: ValuePointerRef) -> ControlFlow<Self, Self> {
        if let Some(MyError(mut errors)) = self_ {
                other.0.append(&mut errors);
        }
        ControlFlow::Continue(other)
    }
}

#[derive(Debug, Deserr, PartialEq, Eq)]
#[deserr(deny_unknown_fields)]
struct Search {
    #[deserr(default = String::new())]
    query: String,
    #[deserr(try_from(&String) = FromStr::from_str -> IndexUidError)]
    index: IndexUid,
    #[deserr(from(String) = From::from)]
    field: Wildcard,
    #[deserr(default)]
    filter: Option<serde_json::Value>,
    // Even though this field is an `Option` it IS mandatory.
    limit: Option<usize>,
    #[deserr(default)]
    offset: usize,
}

/// An `IndexUid` can only be composed of ascii characters.
#[derive(Debug, PartialEq, Eq)]
struct IndexUid(String);
/// If we encounter a non-ascii character this is the error type we're going to throw.
struct IndexUidError(char);

impl FromStr for IndexUid {
    type Err = IndexUidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(c) = s.chars().find(|c| !c.is_ascii()) {
            Err(IndexUidError(c))
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

impl fmt::Display for IndexUidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Encountered invalid character: `{}`, only ascii characters are accepted in the index",
            self.0
        )
    }
}

/// We need to define how the `IndexUidError` error is going to be merged with our
/// custom error type.
impl MergeWithError<IndexUidError> for MyError {
    fn merge(self_: Option<Self>, other: IndexUidError, merge_location: ValuePointerRef) -> ControlFlow<Self, Self> {
            // To be consistent with the other error and automatically get the position of the error we re-use the `JsonError`
            // type and simply define ourself as an `Unexpected` error.
        let error = take_cf_content(JsonError::error::<Infallible>(None, ErrorKind::Unexpected { msg: other.to_string() }, merge_location));
        let errors = if let Some(MyError(mut errors)) = self_ {
            errors.push(error);
            errors
        } else {
            vec![error]
        };
        ControlFlow::Continue(MyError(errors))
    }
}

/// A `Wildcard` can either contains a normal value or be a unit wildcard.
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

// Here is an example of a typical payload we could deserialize:
let data = deserialize::<Search, _, MyError>(
    json!({ "index": "mieli", "field": "doggo", "filter": ["id = 1", ["catto = jorts"]], "limit": null }),
).unwrap();
assert_eq!(data, Search {
    query: String::new(),
    index: IndexUid(String::from("mieli")),
    field: Wildcard::Value(String::from("doggo")),
    filter: Some(json!(["id = 1", ["catto = jorts"]])),
    limit: None,
    offset: 0,
});

// And here is what happens when everything goes wrong at the same time:
let error = deserialize::<Search, _, MyError>(
    json!({ "query": 12, "index": "mieli 🍯", "field": true, "offset": "🔢"  }),
).unwrap_err();
// We're going to stringify all the error so it's easier to read
assert_eq!(error.0.into_iter().map(|error| error.to_string()).collect::<Vec<String>>().join("\n"),
"\
Invalid value type at `.query`: expected a string, but found a positive integer: `12`
Invalid value type at `.offset`: expected a positive integer, but found a string: `\"🔢\"`
Invalid value at `.index`: Encountered invalid character: `🍯`, only ascii characters are accepted in the index
Invalid value type at `.field`: expected a string, but found a boolean: `true`
Missing field `limit`\
");
```

### Supported features

#### `rename_all`

Rename all the fields of the struct according to the given case convention.
The possible values are `lowercase`, `camelCase`.
If you need more case conventions, please open an issue; adding more is trivial.

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

#### `deny_unknown_fields`

Throw an error when encountering unknown fields.
When this attribute is absent, unknown fields are ignored by default.

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

It is also possible to provide a custom function to handle the error.

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

#### `tag`

Externally tag an enum.
Deserr does not support internally tagging your enum yet, which means you'll always
need to use this attribute if you're deserializing an enum.
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

#### `from`

Deserializing a type from a function instead of a `Value`.
You need to provide the following information;
1. The input type of the function (here `&String`)
2. The path of the function (here, we're simply using the std `FromStr` implementation)

deserr will first try to deserialize the given type using its `Deserr<E>` implementation.
That means the input type of the `from` can be complex. Then deserr will call your
function.

See also `try_from` if your function can fail.

##### It can be used as a container attribute

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

##### Or as a field attribute

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

#[derive(Deserr, Debug, PartialEq, Eq)]
struct Search {
    query: String,
    #[deserr(from(String) = From::from)]
    field: Wildcard,
}

let data = deserialize::<Search, _, JsonError>(
    json!({ "query": "doggo", "field": "catto" }),
)
.unwrap();
assert_eq!(data, Search { query: String::from("doggo"), field: Wildcard::Value(String::from("catto")) });

let data = deserialize::<Search, _, JsonError>(
    json!({ "query": "doggo", "field": "*" }),
)
.unwrap();
assert_eq!(data, Search { query: String::from("doggo"), field: Wildcard::Wildcard });
```

#### `try_from`

Try deserializing a type from a function instead of a `Value`.
You need to provide the following information;
1. The input type of the function (here `&String`)
2. The path of the function (here, we're simply using the std `FromStr` implementation)
3. The error type that this function can return (here `Infallible`)

deserr will first try to deserialize the given type using its `Deserr<E>` implementation.
That means the input type of the `try_from` can be complex. Then deserr will call your
function and accumulate the specified error against the error type of the caller.

##### It can be used as a container attribute

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

##### Or as a field attribute

```rust
use deserr::{Deserr, deserialize, errors::JsonError};
use serde_json::json;
use std::convert::Infallible;
use std::str::FromStr;
use std::num::ParseIntError;

#[derive(Deserr, Debug, PartialEq, Eq)]
struct Search {
    query: String,
    #[deserr(try_from(&String) = FromStr::from_str -> ParseIntError)]
    limit: usize,

}

let data = deserialize::<Search, _, JsonError>(
    json!({ "query": "doggo", "limit": "12" }),
)
.unwrap();
assert_eq!(data, Search { query: String::from("doggo"), limit: 12 });

let error = deserialize::<Search, _, JsonError>(
    json!({ "query": "doggo", "limit": 12 }),
)
.unwrap_err();
assert_eq!(error.to_string(), "Invalid value type at `.limit`: expected a string, but found a positive integer: `12`");
```

#### `validate`

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

#### `default`

Allows you to specify a default value for a field.

Note that, unlike serde, by default, `Option` doesn't automatically use this attribute.
Here you need to explicitly define whether your type can get a default value.
This makes it less error-prone and easier to make an optional field mandatory.

```rust
use deserr::{Deserr, deserialize, errors::JsonError};
use serde_json::json;

#[derive(Deserr, Debug, PartialEq, Eq)]
struct Search {
    #[deserr(default)]
    query: Option<String>,
    #[deserr(default = 20)]
    limit: usize,
}

let data = deserialize::<Search, _, JsonError>(
    json!({ "query": "doggo", "limit": 4 }),
)
.unwrap();
assert_eq!(data, Search { query: Some(String::from("doggo")), limit: 4 });

let data = deserialize::<Search, _, JsonError>(
    json!({ "query": "doggo" }),
)
.unwrap();
assert_eq!(data, Search { query: Some(String::from("doggo")), limit: 20 });
```

#### `skip`

Allows you to skip the deserialization of a field.
It won't show up in the list of fields generated by `deny_unknown_fields` or in the
`UnknownKey` variant of the `ErrorKind` type.

```rust
use deserr::{Deserr, deserialize, errors::JsonError};
use serde_json::json;

#[derive(Deserr, Debug, PartialEq, Eq)]
struct Search {
    query: String,
    // A field can be skipped if it implements `Default` or if the `default` attribute is specified.
    #[deserr(skip)]
    hidden: usize,
}

let data = deserialize::<Search, _, JsonError>(
    json!({ "query": "doggo" }),
)
.unwrap();
assert_eq!(data, Search { query: String::from("doggo"), hidden: 0 });

// if you try to specify the field, it is ignored
let data = deserialize::<Search, _, JsonError>(
    json!({ "query": "doggo", "hidden": 2 }),
)
.unwrap();
assert_eq!(data, Search { query: String::from("doggo"), hidden: 0 });

// Here, we're going to see how skip interacts with `deny_unknown_fields`

#[derive(Deserr, Debug, PartialEq, Eq)]
#[deserr(deny_unknown_fields)]
struct Search2 {
    query: String,
    // A field can be skipped if it implements `Default`.
    #[deserr(skip)]
    hidden: usize,
}

let error = deserialize::<Search2, _, JsonError>(
    json!({ "query": "doggo", "hidden": 1 }),
)
.unwrap_err();
// NOTE: `hidden` isn't in the list of expected fields + `hidden` is effectively considered as a non-existing field.
assert_eq!(error.to_string(), "Unknown field `hidden`: expected one of `query`");
```

#### `map`

Map a field **after** it has been deserialized.

```rust
use deserr::{Deserr, deserialize, errors::JsonError};
use serde_json::json;

#[derive(Deserr, Debug, PartialEq, Eq)]
struct Search {
    query: String,
    #[deserr(map = add_one)]
    limit: usize,
}

fn add_one(n: usize) -> usize {
    n.saturating_add(1)
}

let data = deserialize::<Search, _, JsonError>(
    json!({ "query": "doggo", "limit": 0 }),
)
.unwrap();
assert_eq!(data, Search { query: String::from("doggo"), limit: 1 });

// Let's see how `map` interacts with the `default` attributes.
#[derive(Deserr, Debug, PartialEq, Eq)]
struct Search2 {
    query: String,
    #[deserr(default, map = add_one)]
    limit: usize,
}

let data = deserialize::<Search2, _, JsonError>(
    json!({ "query": "doggo" }),
)
.unwrap();
// As we can see, the `map` attribute is applied AFTER the `default`.
assert_eq!(data, Search2 { query: String::from("doggo"), limit: 1 });
```

#### `missing_field_error`

Gives you the opportunity to customize the error message if this specific field
is missing.

```rust
use deserr::{Deserr, DeserializeError, ValuePointerRef, ErrorKind, deserialize, errors::JsonError};
use serde_json::json;
use std::convert::Infallible;

#[derive(Deserr, Debug, PartialEq, Eq)]
struct Search {
    #[deserr(missing_field_error = missing_query_field)]
    query: String,
    limit: usize,
}

fn missing_query_field<E: DeserializeError>(_field_name: &str, location: ValuePointerRef) -> E {
    deserr::take_cf_content(E::error::<Infallible>(
        None,
        ErrorKind::Unexpected {
            msg: String::from("I really need the query field, please give it to me uwu"),
        },
        location,
    ))
}

let error = deserialize::<Search, _, JsonError>(
    json!({ "limit": 0 }),
)
.unwrap_err();
assert_eq!(error.to_string(), "Invalid value: I really need the query field, please give it to me uwu");
```

#### `error`

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

It can also be used as a field attribute;

```rust
use deserr::{Deserr, DeserializeError, ValuePointerRef, ErrorKind, deserialize, errors::JsonError};
use serde_json::json;

// Since the error returned by the `Search` structure needs to implements `MergeWithError<JsonError>`
// we also need to specify the `error` attribute as a `JsonError`. But as you will see later there are
// other solutions.
#[derive(Deserr, Debug, PartialEq, Eq)]
#[deserr(error = JsonError)]
struct Search<A> {
    #[deserr(error = JsonError)]
    query: A,
    limit: usize,
}
```

#### `where_predicate`

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

#### `needs_predicate`

Automatically adds `where_predicate = FieldType: Deserr<ErrType>` for each field with this attribute.

```rust
use deserr::{Deserr, DeserializeError, MergeWithError, deserialize, errors::JsonError};
use serde_json::json;

#[derive(Deserr, Debug, PartialEq, Eq)]
struct Search<A> {
    #[deserr(needs_predicate)]
    query: A,
    limit: usize,
}
```

Is strictly equivalent to the following:

```rust
use deserr::{Deserr, DeserializeError, MergeWithError, deserialize, errors::JsonError};
use serde_json::json;

// `__Deserr_E` represents the Error returned by the generated `Deserr` implementation.
#[derive(Deserr, Debug, PartialEq, Eq)]
#[deserr(where_predicate = A: Deserr<__Deserr_E>)]
struct Search<A> {
    query: A,
    limit: usize,
}
```

### Comparison with serde

Since deserr needs to first deserialize the payload into a generic `Value` that allocates
a lot of memory before creating your structure, it's a lot slower than serde.

For example, at Meilisearch for our search route, in case of a valid payload, we observed a 400% slowdown (4 times slower).
That made our search request deserialize in 2µs instead of 500ns.
This is fast enough for most use cases but could be an issue if most of your time is spent deserializing.

#### Datastructure support

| datastructure       | serde | deserr | note |
|---------------------|-------|--------|------|
| Struct              |  yes  |  yes   |      |
| Tuple struct        |  yes  |  no    |      |
| Untagged Enum       |  yes  |  no    |      |
| Untagged unit Enum  |  yes  |  yes   |      |
| Tagged Enum         |  yes  |  yes   |      |

#### Container attributes

| features            | serde | deserr | note |
|---------------------|-------|--------|------|
| rename              |  yes  |  no    |      |
| rename_all          |  yes  |  yes   |      |
| deny_unknown_fields |  yes  |  yes   | With deserr you can call a custom function when an unknown field is encountered |
| tag                 |  yes  |  yes   |      |
| tag+content         |  yes  |  no    |      |
| untagged            |  yes  |  no    | it's only supported for unit enums |
| bound               |  yes  |  no    | Can be emulated with `where_predicate` |
| default             |  yes  |  no    |      |
| remote              |  yes  |  no    |      |
| transparent         |  yes  |  no    |      |
| from                |  yes  |  yes   |      |
| try_from            |  yes  |  yes   |      |
| into                |  yes  |  no    |      |
| crate               |  yes  |  no    |      |
| validate            |  no   |  yes   | Allows you to validate the content of struct **after** it has been deserialized |
| error               |  no   |  yes   | Specify the error type that should be used while deserializing this structure |
| where_predicate     |  no   |  yes   | Let you add where clauses to the generated `Deserr` implementation |

#### Field attributes

| features            | serde | deserr | note |
|---------------------|-------|--------|------|
| rename              |  yes  |  no    |      |
| alias               |  yes  |  no    |      |
| default             |  yes  |  yes   |      |
| flatten             |  yes  |  no    | serde doesn't support flattening + denying unknown field |
| skip                |  yes  |  yes   |      |
| deserialize_with    |  yes  |  no    | But it's kinda emulated with `from` and `try_from` |
| with                |  yes  |  no    |      |
| borrow              |  yes  |  no    | deserr does not support types with references |
| bound               |  yes  |  no    |      |
| map                 |  no   |  yes   | Allows you to map the value **after** it was deserialized |
| from                |  no   |  yes   | Deserialize this field from an infallible function |
| try_from            |  no   |  yes   | Deserialize this field from a fallible function |
| missing_field_error |  no   |  yes   | Allows you to return a custom error if this field is missing |
| error               |  no   |  yes   | Specify the error type that should be used while deserializing this field |

### Feature flags

#### `serde-json`

Import [`serde_json`](https://crates.io/crates/serde_json) and provide;
- An implementation of `deserr::IntoValue` for `serde_json::Value` which make it easy to use both crate together.
- A default implementation of the `JsonError` type that provide the best generic error messages possible.

#### `serde-cs`
Import [`serde-cs`](https://crates.io/crates/serde-cs) and provide;
- An implementation of `Deserr` for `serde_cs::CS<R>`.

#### `actix-web`
Import [`actix-web`](https://crates.io/crates/actix-web) and [`futures`](https://crates.io/crates/futures) and provide;
- An implementation of a json actix-web extractor if used with the `serde-json` feature.
- An implementation of `ResponseError` for the `JsonError` type if used with the `serde-json` feature.


### FAQ

#### But why?
At Meilisearch, we wanted to customize the error code we return when we fail
the deserialization of a specific field.
Some error messages were also not clear at all and impossible to edit.

#### What about the maintenance?
At Meilisearch we're already using deserr in production; thus, it's well maintained.

#### Where can I see more examples of usage of this crate?
Currently, you can read our examples in the `examples` directory of this repository.
You can also look at our integration test; each attribute has a simple-to-read test.

And obviously, you can read the code of Meilisearch where deserr is used on all our
routes.

#### My question is not listed
Please, if you think there is a bug in this lib or would like a new feature,
open an issue or a discussion.
If you would like to chat more directly with us, you can join us on discord
at https://discord.com/invite/meilisearch

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
