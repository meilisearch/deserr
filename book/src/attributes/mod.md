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
