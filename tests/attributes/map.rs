use deserr::{deserialize, errors::JsonError, Deserr};
use insta::assert_debug_snapshot;
use serde_json::json;

#[test]
fn map() {
    #[allow(unused)]
    #[derive(Debug, Deserr)]
    struct Struct {
        #[deserr(map = square)]
        doggo: usize,
    }

    fn square(n: usize) -> usize {
        n * n
    }

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": 1 })).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: 1,
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({})).unwrap_err();

    assert_debug_snapshot!(data, @r###"
    JsonError(
        "Missing field `doggo`",
    )
    "###);
}

#[test]
fn map_and_default() {
    #[allow(unused)]
    #[derive(Debug, Deserr)]
    struct Struct {
        #[deserr(default = 2, map = square)]
        doggo: usize,
    }

    fn square(n: usize) -> usize {
        n * n
    }

    let data = deserialize::<Struct, _, JsonError>(json!({ "doggo": 1 })).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: 1,
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(json!({})).unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: 4,
    }
    "###);
}
