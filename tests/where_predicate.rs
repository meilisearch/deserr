use deserr::{deserialize, serde_json::JsonError, DeserializeFromValue};
use insta::assert_debug_snapshot;
use serde_json::json;

#[test]
fn where_attribute() {
    #[allow(unused)]
    #[derive(Debug, DeserializeFromValue)]
    #[deserr(where_predicate = T: DeserializeFromValue<__Deserr_E>)]
    struct Struct<T> {
        doggo: String,
        catto: T,
    }

    let data =
        deserialize::<Struct<String>, _, JsonError>(json!({ "doggo": "bork", "catto": "jorts" }))
            .unwrap();
    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: "bork",
        catto: "jorts",
    }
    "###);

    let data =
        deserialize::<Struct<usize>, _, JsonError>(json!({ "doggo": "bork", "catto": 3 })).unwrap();
    assert_debug_snapshot!(data, @r###"
    Struct {
        doggo: "bork",
        catto: 3,
    }
    "###);
}
