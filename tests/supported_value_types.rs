use deserr::{deserialize, errors::JsonError, Deserr};
use insta::assert_debug_snapshot;
use serde_json::json;

#[test]
fn numbers() {
    #[allow(dead_code)]
    #[derive(Debug, Deserr)]
    struct Struct {
        u8: u8,
        u16: u16,
        u32: u32,
        u64: u64,
        u128: u128,

        i8: i8,
        i16: i16,
        i32: i32,
        i64: i64,
        i128: i128,

        f32: f32,
        f64: f64,
    }

    let data = deserialize::<Struct, _, JsonError>(json!({
       "u8": 1,
       "u16": 1,
       "u32": 1,
       "u64": 1,
       "u128": 1,

       "i8": 1,
       "i16": 1,
       "i32": 1,
       "i64": 1,
       "i128": 1,

       "f32": 1,
       "f64": 1,
    }))
    .unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        u8: 1,
        u16: 1,
        u32: 1,
        u64: 1,
        u128: 1,
        i8: 1,
        i16: 1,
        i32: 1,
        i64: 1,
        i128: 1,
        f32: 1.0,
        f64: 1.0,
    }
    "###);
}

#[test]
fn strings() {
    #[allow(dead_code)]
    #[derive(Debug, Deserr)]
    struct Struct {
        c: char,
        s: String,
    }
    let data = deserialize::<Struct, _, JsonError>(json!({
       "c": "c",
       "s": "catto",
    }))
    .unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        c: 'c',
        s: "catto",
    }
    "###);
}

#[test]
fn boolean() {
    #[allow(dead_code)]
    #[derive(Debug, Deserr)]
    struct Struct {
        b: bool,
    }
    let data = deserialize::<Struct, _, JsonError>(json!({
       "b": true,
    }))
    .unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        b: true,
    }
    "###);
}

#[test]
fn tuple() {
    #[allow(dead_code)]
    #[derive(Debug, Deserr)]
    struct Struct1 {
        tuple: (),
    }
    let data = deserialize::<Struct1, _, JsonError>(json!({
       "tuple": null,
    }))
    .unwrap();

    // we can't create tuple of one elements, rust is going to complain and get rids of the parenthesis

    assert_debug_snapshot!(data, @r###"
    Struct1 {
        tuple: (),
    }
    "###);

    #[allow(dead_code)]
    #[derive(Debug, Deserr)]
    struct Struct2 {
        tuple: (bool, char),
    }
    let data = deserialize::<Struct2, _, JsonError>(json!({
       "tuple": [true, 'c'],
    }))
    .unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct2 {
        tuple: (
            true,
            'c',
        ),
    }
    "###);

    #[allow(dead_code)]
    #[derive(Debug, Deserr)]
    struct Struct3 {
        tuple: (bool, char, u8),
    }
    let data = deserialize::<Struct3, _, JsonError>(json!({
       "tuple": [true, 'c', 2],
    }))
    .unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct3 {
        tuple: (
            true,
            'c',
            2,
        ),
    }
    "###);
}

#[test]
fn array() {
    #[allow(dead_code)]
    #[derive(Debug, Deserr)]
    struct Struct0 {
        arr: [u8; 0],
    }
    let data = deserialize::<Struct0, _, JsonError>(json!({
       "arr": [],
    }))
    .unwrap();
    assert_debug_snapshot!(data, @r###"
    Struct0 {
        arr: [],
    }
    "###);

    #[allow(dead_code)]
    #[derive(Debug, Deserr)]
    struct Struct1 {
        arr: [u8; 1],
    }
    let data = deserialize::<Struct1, _, JsonError>(json!({
       "arr": [2],
    }))
    .unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct1 {
        arr: [
            2,
        ],
    }
    "###);

    #[allow(dead_code)]
    #[derive(Debug, Deserr)]
    struct Struct2 {
        arr: [bool; 2],
    }
    let data = deserialize::<Struct2, _, JsonError>(json!({
       "arr": [true, false],
    }))
    .unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct2 {
        arr: [
            true,
            false,
        ],
    }
    "###);

    #[allow(dead_code)]
    #[derive(Debug, Deserr)]
    struct Struct3 {
        arr: [bool; 3],
    }
    let data = deserialize::<Struct3, _, JsonError>(json!({
       "arr": [true, false, true],
    }))
    .unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct3 {
        arr: [
            true,
            false,
            true,
        ],
    }
    "###);
}
