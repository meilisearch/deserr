use deserr::{deserialize, Deserr, JsonError};
use insta::{assert_debug_snapshot, assert_display_snapshot};
use serde_json::json;

#[test]
fn rename_all_camel_case() {
    #[allow(unused)]
    #[derive(Debug, Deserr)]
    #[deserr(rename_all = camelCase)]
    struct Struct {
        word: String,
        multiple_words: String,
        #[deserr(rename = "renamed_field")]
        renamed_field: String,
    }

    let data = deserialize::<Struct, _, JsonError>(
        json!({ "word": "doggo", "multipleWords": "good doggo", "renamed_field": "bork" }),
    )
    .unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        word: "doggo",
        multiple_words: "good doggo",
        renamed_field: "bork",
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(
        json!({ "Word": "doggo", "multipleWords": "good doggo", "renamed_field": "bork" }),
    )
    .unwrap_err();

    assert_display_snapshot!(data, @"Missing field `word`");

    let data = deserialize::<Struct, _, JsonError>(
        json!({ "word": "doggo", "multiple_words": "good doggo", "renamed_field": "bork" }),
    )
    .unwrap_err();

    assert_display_snapshot!(data, @"Missing field `multipleWords`");

    let data = deserialize::<Struct, _, JsonError>(
        json!({ "word": "doggo", "multipleWords": "good doggo", "renamedField": "bork" }),
    )
    .unwrap_err();

    assert_display_snapshot!(data, @"Missing field `renamed_field`");
}

#[allow(non_snake_case)]
#[test]
fn rename_all_lowercase() {
    #[allow(unused)]
    #[derive(Debug, Deserr)]
    #[deserr(rename_all = lowercase)]
    struct Struct {
        word: String,
        SCREAMING_WORD: String,
        #[deserr(rename = "BORK")]
        smol: String,
    }

    let data = deserialize::<Struct, _, JsonError>(
        json!({ "word": "doggo", "screaming_word": "good doggo", "BORK": "bork" }),
    )
    .unwrap();

    assert_debug_snapshot!(data, @r###"
    Struct {
        word: "doggo",
        SCREAMING_WORD: "good doggo",
        smol: "bork",
    }
    "###);

    let data = deserialize::<Struct, _, JsonError>(
        json!({ "Word": "doggo", "SCREAMING_WORD": "good doggo", "BORK": "bork" }),
    )
    .unwrap_err();

    assert_display_snapshot!(data, @"Missing field `word`");

    let data = deserialize::<Struct, _, JsonError>(
        json!({ "word": "doggo", "screamingWord": "good doggo", "BORK": "bork" }),
    )
    .unwrap_err();

    assert_display_snapshot!(data, @"Missing field `screaming_word`");

    let data = deserialize::<Struct, _, JsonError>(
        json!({ "word": "doggo", "screaming_word": "good doggo", "smol": "bork" }),
    )
    .unwrap_err();

    assert_display_snapshot!(data, @"Missing field `BORK`");
}
