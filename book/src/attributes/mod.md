# Attributes

[Attributes](https://doc.rust-lang.org/reference/attributes.html) are used to customize the `Deserr`
implementations produced by deserr's derive.

There are three categories of attributes:

- [**Container attributes**] — apply to a struct or enum declaration.
- [**Variant attributes**] — apply to a variant of an enum.
- [**Field attributes**] — apply to one field in a struct or in an enum variant.

[**Container attributes**]: container.md
[**Variant attributes**]: variant.md
[**Field attributes**]: field.md

```rust
# use deserr::Deserr;
#
#[derive(Deserr)]
#[deserr(deny_unknown_fields)]  // <-- this is a container attribute
struct S {
    #[deserr(default)]  // <-- this is a field attribute
    f: i32,
}

#[derive(Deserr)]
#[deserr(rename_all = camelCase)]  // <-- this is also a container attribute
enum E {
    #[deserr(rename = "_deserr")]  // <-- this is a variant attribute
    DeserrIsGreat,
    SerdeIsAwesome
}
#
# fn main() {}
```

Note that a single struct, enum, variant, or field may have multiple attributes
on it.

## Feature comparison table with serde

#### Datastructure support

| datastructure       | serde | deserr | note |
|---------------------|-------|--------|------|
| Struct              |  yes  |  yes   |      |
| Tuple struct        |  yes  |  no    |      |
| Untagged Enum       |  yes  |  no    |      |
| Untagged unit Enum  |  yes  |  yes   |      |
| Tagged Enum         |  yes  |  yes   |      |

#### Container attributes

| features            | serde | deserr                                       | note                                                                            |
|---------------------|-------|----------------------------------------------|---------------------------------------------------------------------------------|
| rename              |  yes  |  no                                          |                                                                                 |
| rename_all          |  yes  |  [yes](container.md#deserrrenameall)         |                                                                                 |
| deny_unknown_fields |  yes  |  [yes](container.md#deserrdenyunknownfields) | With deserr you can call a custom function when an unknown field is encountered |
| tag                 |  yes  |  [yes](container.md#deserrtag)               |                                                                                 |
| tag+content         |  yes  |  no                                          |                                                                                 |
| untagged            |  yes  |  no                                          | it's only supported for unit enums                                              |
| bound               |  yes  |  no                                          | Can be emulated with `where_predicate`                                          |
| default             |  yes  |  no                                          |                                                                                 |
| remote              |  yes  |  no                                          |                                                                                 |
| transparent         |  yes  |  no                                          |                                                                                 |
| from                |  yes  |  [yes](container.md#deserrfrom)              |                                                                                 |
| try_from            |  yes  |  [yes](container.md#deserrtryfrom)           |                                                                                 |
| into                |  yes  |  no                                          |                                                                                 |
| crate               |  yes  |  no                                          |                                                                                 |
| validate            |  no   |  [yes](container.md#deserrvalidate)          | Allows you to validate the content of struct **after** it has been deserialized |
| error               |  no   |  [yes](container.md#deserrerror)             | Specify the error type that should be used while deserializing this structure   |
| where_predicate     |  no   |  [yes](container.md#deserrwherepredicate)    | Let you add where clauses to the generated `Deserr` implementation              |

#### Field attributes

| features            | serde | deserr                                     | note                                                                      |
|---------------------|-------|--------------------------------------------|---------------------------------------------------------------------------|
| rename              |  yes  |  [yes](field.md#deserrrename)              |                                                                           |
| alias               |  yes  |  no                                        |                                                                           |
| default             |  yes  |  [yes](field.md#deserrdefault)             |                                                                           |
| flatten             |  yes  |  no                                        | serde doesn't support flattening + denying unknown field                  |
| skip                |  yes  |  [yes](field.md#deserrskip)                |                                                                           |
| deserialize_with    |  yes  |  no                                        | But it's kinda emulated with `from` and `try_from`                        |
| with                |  yes  |  no                                        |                                                                           |
| borrow              |  yes  |  no                                        | deserr does not support types with references                             |
| bound               |  yes  |  no                                        |                                                                           |
| map                 |  no   |  [yes](field.md#deserrmap)                 | Allows you to map the value **after** it was deserialized                 |
| from                |  no   |  [yes](field.md#deserrfrom)                | Deserialize this field from an infallible function                        |
| try_from            |  no   |  [yes](field.md#deserrtry_from)            | Deserialize this field from a fallible function                           |
| missing_field_error |  no   |  [yes](field.md#deserrmissing_field_error) | Allows you to return a custom error if this field is missing              |
| error               |  no   |  [yes](field.md#deserrerror)               | Specify the error type that should be used while deserializing this field |

