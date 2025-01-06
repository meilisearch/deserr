<p align="center"><img width="280px" title="The deserr logo is a crab similar to Ferris with an ice cream all in place of his body" src="https://raw.githubusercontent.com/meilisearch/deserr/main/assets/deserr.png"></a>
<h1 align="center">deserr</h1>

[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE-MIT)
[![License](https://img.shields.io/badge/license-apache-green)](LICENSE-APACHE)
[![Crates.io](https://img.shields.io/crates/v/deserr)](https://crates.io/crates/deserr)
[![Docs](https://docs.rs/deserr/badge.svg)](https://docs.rs/deserr)
[![dependency status](https://deps.rs/repo/github/meilisearch/deserr/status.svg)](https://deps.rs/repo/github/meilisearch/deserr)

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

You may be looking for:
- [The `docs.rs` documentation](https://docs.rs/deserr/latest/deserr/)
- [The reference book](https://meilisearch.github.io/deserr/overview.html)

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

#### The logo
The logo was graciously offered and crafted by @irevoire 's sister after a lot of back and forth.
Many thanks to her.

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
