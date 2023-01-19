use deserr::serde_json::JsonError;
use serde_json::json;

#[allow(unused)]
#[derive(Debug, deserr::DeserializeFromValue)]
#[deserr(deny_unknown_fields)]
struct Test {
    #[deserr(default)]
    u8: u8,
    #[deserr(default)]
    u16: u16,
    #[deserr(default)]
    u32: u32,
    #[deserr(default)]
    u64: u64,
    #[deserr(default)]
    usize: usize,

    #[deserr(default)]
    i8: i8,
    #[deserr(default)]
    i16: i16,
    #[deserr(default)]
    i32: i32,
    #[deserr(default)]
    i64: i64,
    #[deserr(default)]
    isize: isize,
}

#[test]
fn positive_integer() {
    // ensuring it deserialize correctly over the whole range of number.
    for i in u8::MIN..=u8::MAX {
        deserr::deserialize::<Test, _, JsonError>(json!({ "u8": i })).unwrap();
    }

    let ret =
        deserr::deserialize::<Test, _, JsonError>(json!({ "u8": u8::MAX as u16 + 1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "value: `256` is too large to be deserialized, maximum value authorized is `255` at `.u8`.",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "u8": -1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "invalid type: NegativeInteger `-1`, expected a Integer at `.u8`.",
    )
    "###);

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "u16": u16::MAX as u32 + 1 }))
        .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "value: `65536` is too large to be deserialized, maximum value authorized is `65535` at `.u16`.",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "u16": -1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "invalid type: NegativeInteger `-1`, expected a Integer at `.u16`.",
    )
    "###);

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "u32": u32::MAX as u64 + 1 }))
        .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "value: `4294967296` is too large to be deserialized, maximum value authorized is `4294967295` at `.u32`.",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "u32": -1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "invalid type: NegativeInteger `-1`, expected a Integer at `.u32`.",
    )
    "###);

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "u64": -1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "invalid type: NegativeInteger `-1`, expected a Integer at `.u64`.",
    )
    "###);

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "usize": -1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "invalid type: NegativeInteger `-1`, expected a Integer at `.usize`.",
    )
    "###);

    // we can't test the u64 and usize because we have no way to create a value that overflow since it's `serde_json` that doesn't support u128 yet.

    // let ret = deserr::deserialize::<Test, _, DefaultError>(json!({ "u64": u64::MAX as u128 + 1 }))
    //     .unwrap_err();
    // insta::assert_debug_snapshot!(ret, @"");

    // let ret =
    //     deserr::deserialize::<Test, _, DefaultError>(json!({ "usize": -1 }))
    //         .unwrap_err();
    // insta::assert_debug_snapshot!(ret, @"");
}

#[test]
fn negative_integer() {
    // ensuring it deserialize correctly over the whole range of number.
    for i in i8::MIN..=i8::MAX {
        deserr::deserialize::<Test, _, JsonError>(json!({ "i8": i })).unwrap();
    }

    let ret =
        deserr::deserialize::<Test, _, JsonError>(json!({ "i8": i8::MAX as i16 + 1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "value: `128` is too large to be deserialized, maximum value authorized is `127` at `.i8`.",
    )
    "###);
    let ret =
        deserr::deserialize::<Test, _, JsonError>(json!({ "i8": i8::MIN as i16 - 1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "value: `-129` is too small to be deserialized, minimum value authorized is `-128` at `.i8`.",
    )
    "###);

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "i16": i16::MAX as i32 + 1 }))
        .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "value: `32768` is too large to be deserialized, maximum value authorized is `32767` at `.i16`.",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "i16": i16::MIN as i32 - 1 }))
        .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "value: `-32769` is too small to be deserialized, minimum value authorized is `-32768` at `.i16`.",
    )
    "###);

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "i32": i32::MAX as i64 + 1 }))
        .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "value: `2147483648` is too large to be deserialized, maximum value authorized is `2147483647` at `.i32`.",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "i32": i32::MIN as i64 - 1 }))
        .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "value: `-2147483649` is too small to be deserialized, minimum value authorized is `-2147483648` at `.i32`.",
    )
    "###);

    // we can't test the i64 and isize because we have no way to create a value that overflow since it's `serde_json` that doesn't support i128 yet.
}
