use deserr::errors::JsonError;
use serde_json::json;
use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};

#[allow(unused)]
#[derive(Debug, deserr::Deserr)]
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
    #[deserr(default = NonZeroU8::MIN)]
    non_zero_u8: NonZeroU8,
    #[deserr(default = NonZeroU16::MIN)]
    non_zero_u16: NonZeroU16,
    #[deserr(default = NonZeroU32::MIN)]
    non_zero_u32: NonZeroU32,
    #[deserr(default = NonZeroU64::MIN)]
    non_zero_u64: NonZeroU64,
    #[deserr(default = NonZeroU128::MIN)]
    non_zero_u128: NonZeroU128,
    #[deserr(default = NonZeroUsize::MIN)]
    non_zero_usize: NonZeroUsize,
    #[deserr(default = NonZeroI8::MIN)]
    non_zero_i8: NonZeroI8,
    #[deserr(default = NonZeroI16::MIN)]
    non_zero_i16: NonZeroI16,
    #[deserr(default = NonZeroI32::MIN)]
    non_zero_i32: NonZeroI32,
    #[deserr(default = NonZeroI64::MIN)]
    non_zero_i64: NonZeroI64,
    #[deserr(default = NonZeroI128::MIN)]
    non_zero_i128: NonZeroI128,
    #[deserr(default = NonZeroIsize::MIN)]
    non_zero_isize: NonZeroIsize,
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
        "Invalid value at `.u8`: value: `256` is too large to be deserialized, maximum value authorized is `255`",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "u8": -1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value type at `.u8`: expected a positive integer, but found a negative integer: `-1`",
    )
    "###);

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "u16": u16::MAX as u32 + 1 }))
        .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.u16`: value: `65536` is too large to be deserialized, maximum value authorized is `65535`",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "u16": -1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value type at `.u16`: expected a positive integer, but found a negative integer: `-1`",
    )
    "###);

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "u32": u32::MAX as u64 + 1 }))
        .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.u32`: value: `4294967296` is too large to be deserialized, maximum value authorized is `4294967295`",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "u32": -1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value type at `.u32`: expected a positive integer, but found a negative integer: `-1`",
    )
    "###);

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "u64": -1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value type at `.u64`: expected a positive integer, but found a negative integer: `-1`",
    )
    "###);

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "usize": -1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value type at `.usize`: expected a positive integer, but found a negative integer: `-1`",
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
fn non_zero_positive_integer() {
    // ensuring it deserialize correctly over the whole range of number.
    for i in u8::from(NonZeroU8::MIN)..=u8::from(NonZeroU8::MAX) {
        deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_u8": i })).unwrap();
    }

    let ret =
        deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_u8": u8::MAX as u16 + 1 }))
            .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.non_zero_u8`: value: `256` is too large to be deserialized, maximum value authorized is `255`",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_u8": 0 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.non_zero_u8`: a non-zero integer value lower than `255` was expected, but found a zero",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_u8": -1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value type at `.non_zero_u8`: expected a positive integer, but found a negative integer: `-1`",
    )
    "###);

    let ret =
        deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_u16": u16::MAX as u32 + 1 }))
            .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.non_zero_u16`: value: `65536` is too large to be deserialized, maximum value authorized is `65535`",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_u16": 0 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.non_zero_u16`: a non-zero integer value lower than `65535` was expected, but found a zero",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_u16": -1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value type at `.non_zero_u16`: expected a positive integer, but found a negative integer: `-1`",
    )
    "###);

    let ret =
        deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_u32": u32::MAX as u64 + 1 }))
            .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.non_zero_u32`: value: `4294967296` is too large to be deserialized, maximum value authorized is `4294967295`",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_u32": 0 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.non_zero_u32`: a non-zero integer value lower than `4294967295` was expected, but found a zero",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_u32": -1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value type at `.non_zero_u32`: expected a positive integer, but found a negative integer: `-1`",
    )
    "###);

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_u64": 0 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.non_zero_u64`: a non-zero integer value lower than `18446744073709551615` was expected, but found a zero",
    )
    "###);

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_u64": -1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value type at `.non_zero_u64`: expected a positive integer, but found a negative integer: `-1`",
    )
    "###);

    let ret =
        deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_usize": 0 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.non_zero_usize`: a non-zero integer value lower than `18446744073709551615` was expected, but found a zero",
    )
    "###);

    let ret =
        deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_usize": -1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value type at `.non_zero_usize`: expected a positive integer, but found a negative integer: `-1`",
    )
    "###);

    // we can't test the u64 and usize because we have no way to create a value that overflow since it's `serde_json` that doesn't support u128 yet.

    // let ret = deserr::deserialize::<Test, _, DefaultError>(json!({ "non_zero_u64": u64::MAX as u128 + 1 }))
    //     .unwrap_err();
    // insta::assert_debug_snapshot!(ret, @"");

    // let ret =
    //     deserr::deserialize::<Test, _, DefaultError>(json!({ "non_zero_usize": -1 }))
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
        "Invalid value at `.i8`: value: `128` is too large to be deserialized, maximum value authorized is `127`",
    )
    "###);
    let ret =
        deserr::deserialize::<Test, _, JsonError>(json!({ "i8": i8::MIN as i16 - 1 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.i8`: value: `-129` is too small to be deserialized, minimum value authorized is `-128`",
    )
    "###);

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "i16": i16::MAX as i32 + 1 }))
        .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.i16`: value: `32768` is too large to be deserialized, maximum value authorized is `32767`",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "i16": i16::MIN as i32 - 1 }))
        .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.i16`: value: `-32769` is too small to be deserialized, minimum value authorized is `-32768`",
    )
    "###);

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "i32": i32::MAX as i64 + 1 }))
        .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.i32`: value: `2147483648` is too large to be deserialized, maximum value authorized is `2147483647`",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "i32": i32::MIN as i64 - 1 }))
        .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.i32`: value: `-2147483649` is too small to be deserialized, minimum value authorized is `-2147483648`",
    )
    "###);

    // we can't test the i64 and isize because we have no way to create a value that overflow since it's `serde_json` that doesn't support i128 yet.
}

#[test]
fn non_zero_negative_integer() {
    // ensuring it deserialize correctly over the whole range of negative numbers.
    for i in i8::from(NonZeroI8::MIN)..-1 {
        deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_i8": i })).unwrap();
    }
    // ensuring it deserialize correctly over the whole range of positive numbers.
    for i in 1..i8::from(NonZeroI8::MAX) {
        deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_i8": i })).unwrap();
    }

    let ret =
        deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_i8": i8::MAX as i16 + 1 }))
            .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.non_zero_i8`: value: `128` is too large to be deserialized, maximum value authorized is `127`",
    )
    "###);
    let ret =
        deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_i8": i8::MIN as i16 - 1 }))
            .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.non_zero_i8`: value: `-129` is too small to be deserialized, minimum value authorized is `-128`",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_i8": 0 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.non_zero_i8`: a non-zero integer value higher than `-128` was expected, but found a zero",
    )
    "###);

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "i16": i16::MAX as i32 + 1 }))
        .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.i16`: value: `32768` is too large to be deserialized, maximum value authorized is `32767`",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "i16": i16::MIN as i32 - 1 }))
        .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.i16`: value: `-32769` is too small to be deserialized, minimum value authorized is `-32768`",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_i16": 0 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.non_zero_i16`: a non-zero integer value higher than `-32768` was expected, but found a zero",
    )
    "###);

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "i32": i32::MAX as i64 + 1 }))
        .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.i32`: value: `2147483648` is too large to be deserialized, maximum value authorized is `2147483647`",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_i32": 0 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.non_zero_i32`: a non-zero integer value higher than `-2147483648` was expected, but found a zero",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "i32": i32::MIN as i64 - 1 }))
        .unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.i32`: value: `-2147483649` is too small to be deserialized, minimum value authorized is `-2147483648`",
    )
    "###);
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_i64": 0 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.non_zero_i64`: a non-zero integer value higher than `-9223372036854775808` was expected, but found a zero",
    )
    "###);
    let ret =
        deserr::deserialize::<Test, _, JsonError>(json!({ "non_zero_isize": 0 })).unwrap_err();
    insta::assert_debug_snapshot!(ret, @r###"
    JsonError(
        "Invalid value at `.non_zero_isize`: a non-zero integer value higher than `-9223372036854775808` was expected, but found a zero",
    )
    "###);
}
