# Variant attributes

### `#[deserr(rename = "...")]`

Deserialize this enum variant with the given name instead of its Rust name.

```rust
use deserr::{Deserr, deserialize, errors::JsonError};
use serde_json::json;

#[derive(Deserr, Debug, PartialEq, Eq)]
enum Dog {
  #[deserr(rename = "the kef")]
  Kefir,
  Echo,
  Intel
}

let data = deserialize::<Dog, _, JsonError>(
    json!("the kef"),
)
.unwrap();
assert_eq!(data, Dog::Kefir);
```

[Also available as a field attribute.](field.md#deserrrename)

### `#[deserr(rename_all = ...)]`

Rename all the variants according to the given case convention.
The possible values are: `lowercase`, `camelCase`.

If you need more values please open an issue, it's easy to implement and was simply not implemented because it isn't required for Meilisearch at the moment.

<div class="warning">

Unlike `serde`, you don't need to put the double-quotes (`"`) around the name of the case, e.g.: `#[deserr(rename_all = camelCase)]`.

</div>

```rust
use deserr::{Deserr, deserialize, errors::JsonError};
use serde_json::json;

#[derive(Deserr, Debug, PartialEq, Eq)]
#[deserr(rename_all = lowercase)]
enum Pets {
  KefirTheSnob,
  EchoTheFilthyGoblin,
  IntelTheWise,
}

let data = deserialize::<Pets, _, JsonError>(
    json!("echothefilthygoblin"),
)
.unwrap();
assert_eq!(data, Pets::EchoTheFilthyGoblin);
```
