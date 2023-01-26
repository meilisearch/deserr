use deserr::{deserialize, serde_json::JsonError, DeserializeFromValue};
use insta::{assert_debug_snapshot, assert_display_snapshot};
use serde_json::json;

#[test]
fn map() {
    #[allow(unused)]
    #[derive(Debug, DeserializeFromValue)]
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
        "Json deserialize error: missing field `doggo` at ``",
    )
    "###);
}

#[test]
fn map_and_default() {
    #[allow(unused)]
    #[derive(Debug, DeserializeFromValue)]
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
