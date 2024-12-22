<p align="center"><img width="280px" title="The deserr logo is a crab similar to Ferris with an ice cream all in place of his body" src="https://raw.githubusercontent.com/meilisearch/deserr/main/assets/deserr.png"></a>

# Overview

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
