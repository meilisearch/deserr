use deserr::errors::JsonError;
use serde_json::json;

#[allow(unused)]
#[derive(Debug, deserr::Deserr)]
#[deserr(deny_unknown_fields)]
struct Test {
    #[deserr(default)]
    c: char,
}

#[test]
fn deserialize_char() {
    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "c": "j" })).unwrap();
    insta::assert_debug_snapshot!(ret, @r###"
    Test {
        c: 'j',
    }
    "###);

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "c": "jorts" })).unwrap_err();
    insta::assert_display_snapshot!(ret, @"Invalid value at `.c`: expected a string of one character, but found the following string of 5 characters: `jorts`");

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "c": "" })).unwrap_err();
    insta::assert_display_snapshot!(ret, @"Invalid value at `.c`: expected a string of one character, but found an empty string");

    let ret = deserr::deserialize::<Test, _, JsonError>(json!({ "c": null })).unwrap_err();
    insta::assert_display_snapshot!(ret, @"Invalid value type at `.c`: expected a string, but found null");
}
