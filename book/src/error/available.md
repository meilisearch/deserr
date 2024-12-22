# Already available error type

Deserr comes with two predefined error type for json and query parameters.

### Json

Json support is made through the [`JsonError`](https://docs.rs/deserr/latest/deserr/errors/json/struct.JsonError.html) type.

#### Changes to the error messages 

Here's a non-exhaustive list of some of the changes that are made to the error message compared to `serde_json`:
- Instead of providing the bytes indice of the error it provides the path of the error using dot: `error.on.field[3]`.
- Use the word `array` instead of `Sequence`
- Use the word `object` instead of `Map`
- Never talk about rust type like `u8` and instead use words like number/integer or the bounds of the number directly.
- When using the `deny_unknown_parameter` container attribute deserr will:
  - List all the available fields of the object.
  - Find and propose the field with the closest name of what was typed with a "did you mean" message.

#### Examples

```rust
use deserr::{Deserr, errors::JsonError};
use serde_json::json;
#[derive(Deserr, Debug)]
#[deserr(deny_unknown_fields, rename_all = camelCase)]
struct Search {
    q: Values,
    filter: u8,
}
#[derive(Deserr, Debug)]
#[deserr(rename_all = camelCase)]
enum Values {
    Q,
    Filter,
}

// The field name is wrong but is close enough of `filter`
let value = json!({ "filler": "doggo" });
let err = deserr::deserialize::<Search, _, JsonError>(value).unwrap_err();
assert_eq!(err.to_string(), "Unknown field `filler`: did you mean `filter`? expected one of `q`, `filter`");

// The field name isn't close to anything
let value = json!({ "a": "doggo" });
let err = deserr::deserialize::<Search, _, JsonError>(value).unwrap_err();
assert_eq!(err.to_string(), "Unknown field `a`: expected one of `q`, `filter`");

// Did you mean also works with enum value
let value = json!({ "q": "filler" });
let err = deserr::deserialize::<Search, _, JsonError>(value).unwrap_err();
assert_eq!(err.to_string(), "Unknown value `filler` at `.q`: did you mean `filter`? expected one of `q`, `filter`");

let value = json!({ "filter": [2] });
let err = deserr::deserialize::<Search, _, JsonError>(value).unwrap_err();
assert_eq!(err.to_string(), "Invalid value type at `.filter`: expected a positive integer, but found an array: `[2]`");
```

### Query Parameter

Query parameter support is made through the [`QueryParamError`](https://docs.rs/deserr/latest/deserr/errors/query_params/struct.QueryParamError.html) type.

#### Changes to the error messages 

Here's a non-exhaustive list of some of the changes that are made to the error message compared to `serde_qs`:
- Instead of providing the bytes indice of the error it provides the path of the error using dot: `error.on.parameter[3]`.
- Use the word `multiple values` instead of `Sequence`
- Use the word `multiple parameters` instead of `Map`
- Never talk about rust type like `u8` and instead use words like number/integer or the bounds of the number directly.
- When using the `deny_unknown_parameter` container attribute deserr will:
  - List all the available parameters of the object.
  - Find and propose the parameter with the closest name of what was typed with a "did you mean" message.

#### Examples


```rust
use deserr::{Deserr, errors::QueryParamError};
use serde_json::json;
#[derive(Deserr, Debug)]
#[deserr(deny_unknown_fields, rename_all = camelCase)]
struct Search {
    q: Values,
    filter: u8,
}
#[derive(Deserr, Debug)]
#[deserr(rename_all = camelCase)]
enum Values {
    Q,
    Filter,
}

// The field name is wrong but is close enough of `filter`
let value = json!({ "filler": "doggo" });
let err = deserr::deserialize::<Search, _, QueryParamError>(value).unwrap_err();
assert_eq!(err.to_string(), "Unknown parameter `filler`: did you mean `filter`? expected one of `q`, `filter`");

// The parameter name isn't close to anything
let value = json!({ "a": "doggo" });
let err = deserr::deserialize::<Search, _, QueryParamError>(value).unwrap_err();
assert_eq!(err.to_string(), "Unknown parameter `a`: expected one of `q`, `filter`");

// Did you mean also works with enum value
let value = json!({ "q": "filler" });
let err = deserr::deserialize::<Search, _, QueryParamError>(value).unwrap_err();
assert_eq!(err.to_string(), "Unknown value `filler` for parameter `q`: did you mean `filter`? expected one of `q`, `filter`");

let value = json!({ "filter": [2] });
let err = deserr::deserialize::<Search, _, QueryParamError>(value).unwrap_err();
// The query parameters are always expecting string in the values
assert_eq!(err.to_string(), "Invalid value type for parameter `filter`: expected a string, but found multiple values");
```

### Want another format

Feel free to open an issue or a PR
