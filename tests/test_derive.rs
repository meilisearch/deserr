use jayson::{
    AccumulatedErrors, DeserializeError, DeserializeFromValue, SingleDeserializeError,
    ValuePointerRef,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq)]
pub enum MyError {
    Unexpected(String),
    MissingField(String),
    IncorrectValueKind { accepted: Vec<jayson::ValueKind> },
    UnknownKey { key: String, accepted: Vec<String> },
    CustomMissingField(u8),
}

impl SingleDeserializeError for MyError {
    fn location(&self) -> Option<jayson::ValuePointer> {
        None
    }

    fn incorrect_value_kind(
        _actual: jayson::ValueKind,
        accepted: &[jayson::ValueKind],
        _location: ValuePointerRef,
    ) -> Self {
        Self::IncorrectValueKind {
            accepted: accepted.into(),
        }
    }

    fn missing_field(field: &str, _location: ValuePointerRef) -> Self {
        Self::MissingField(field.to_string())
    }

    fn unknown_key(key: &str, accepted: &[&str], _location: ValuePointerRef) -> Self {
        Self::UnknownKey {
            key: key.to_string(),
            accepted: accepted.into_iter().map(<_>::to_string).collect(),
        }
    }

    fn unexpected(msg: &str, _location: ValuePointerRef) -> Self {
        Self::Unexpected(msg.to_string())
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[serde(tag = "sometag")]
#[jayson(tag = "sometag")]
enum Tag {
    A,
    B,
}

fn unknown_field_error_gen<E>(k: &str, location: jayson::ValuePointerRef) -> E
where
    E: DeserializeError,
{
    match E::unexpected(None, k, location) {
        Ok(e) => e,
        Err(e) => e,
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(deny_unknown_fields = unknown_field_error_gen)]
struct Example {
    x: String,
    t1: Tag,
    t2: Box<Tag>,
    n: Box<Nested>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
struct Nested {
    y: Option<Vec<String>>,
    z: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError)]
struct StructWithDefaultAttr {
    x: bool,
    #[serde(default = "create_default_u8")]
    #[jayson(default = create_default_u8())]
    y: u8,
    #[serde(default = "create_default_option_string")]
    #[jayson(default = create_default_option_string())]
    z: Option<String>,
}
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError)]
struct StructWithTraitDefaultAttr {
    #[serde(default)]
    #[jayson(default)]
    y: u8,
}

fn create_default_u8() -> u8 {
    152
}
fn create_default_option_string() -> Option<String> {
    Some("hello".to_owned())
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[serde(tag = "t")]
#[jayson(error = MyError, tag = "t")]
enum EnumWithOptionData {
    A {
        x: Option<u8>,
    },
    B {
        #[serde(default = "create_default_option_string")]
        #[jayson(default = create_default_option_string())]
        x: Option<String>,
        #[serde(default = "create_default_u8")]
        #[jayson(default = create_default_u8())]
        y: u8,
    },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, rename_all = camelCase)]
#[serde(rename_all = "camelCase")]
struct RenamedAllCamelCaseStruct {
    renamed_field: bool,
}
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, rename_all = lowercase)]
#[serde(rename_all = "lowercase")]
struct RenamedAllLowerCaseStruct {
    renamed_field: bool,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, tag = "t", rename_all = camelCase)]
#[serde(tag = "t")]
#[serde(rename_all = "camelCase")]
enum RenamedAllCamelCaseEnum {
    SomeField { my_field: bool },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, tag = "t")]
#[serde(tag = "t")]
enum RenamedAllFieldsCamelCaseEnum {
    #[jayson(rename_all = camelCase)]
    #[serde(rename_all = "camelCase")]
    SomeField { my_field: bool },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError)]
struct StructWithRenamedField {
    #[jayson(rename = "renamed_field")]
    #[serde(rename = "renamed_field")]
    x: bool,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, deny_unknown_fields)]
#[serde(deny_unknown_fields)]
struct StructDenyUnknownFields {
    x: bool,
}

fn unknown_field_error(k: &str, _location: ValuePointerRef) -> MyError {
    MyError::UnknownKey {
        key: k.to_owned(),
        accepted: vec!["don't know".to_string()],
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, deny_unknown_fields = unknown_field_error)]
#[serde(deny_unknown_fields)]
struct StructDenyUnknownFieldsCustom {
    x: bool,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, tag = "t", deny_unknown_fields)]
#[serde(tag = "t", deny_unknown_fields)]
enum EnumDenyUnknownFields {
    SomeField { my_field: bool },
    Other { my_field: bool, y: u8 },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, tag = "t", deny_unknown_fields = unknown_field_error)]
#[serde(tag = "t", deny_unknown_fields)]
enum EnumDenyUnknownFieldsCustom {
    SomeField { my_field: bool },
    Other { my_field: bool, y: u8 },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError)]
struct StructMissingFieldError {
    #[jayson(missing_field_error = SingleDeserializeError::missing_field("lol", location) )]
    x: bool,
    #[jayson(missing_field_error = MyError::CustomMissingField(1))]
    y: bool,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, tag = "t")]
enum EnumMissingFieldError {
    A {
        #[jayson(missing_field_error = MyError::CustomMissingField(0))]
        x: bool,
    },
    B {
        x: bool,
    },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, tag = "t")]
#[serde(tag = "t")]
enum EnumRenamedVariant {
    #[serde(rename = "Apple")]
    #[jayson(rename = "Apple")]
    A { x: bool },
    #[serde(rename = "Beta")]
    #[jayson(rename = "Beta")]
    B,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, tag = "t")]
#[serde(tag = "t")]
enum EnumRenamedField {
    A {
        #[jayson(rename = "Xylem")]
        #[serde(rename = "Xylem")]
        x: bool,
    },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError, tag = "t")]
#[serde(tag = "t")]
enum EnumRenamedAllVariant {
    #[jayson(rename_all = camelCase)]
    #[serde(rename_all = "camelCase")]
    P { water_potential: bool },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = MyError)]
struct Generic<A> {
    some_field: A,
}

#[track_caller]
fn compare_with_serde_roundtrip<T>(x: T)
where
    T: Serialize + DeserializeFromValue<MyError> + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_value(&x).unwrap();
    let result: T = jayson::deserialize(json).unwrap();

    assert_eq!(result, x);
}

#[track_caller]
fn compare_with_serde<T>(j: &str)
where
    T: DeserializeOwned + DeserializeFromValue<MyError> + PartialEq + std::fmt::Debug,
{
    let json: serde_json::Value = serde_json::from_str(j).unwrap();

    let actual_serde: Result<T, _> = serde_json::from_str(j);
    let actual_jayson: Result<T, _> = jayson::deserialize(json);

    match (actual_serde, actual_jayson) {
        (Ok(actual_serde), Ok(actual_jayson)) => {
            assert_eq!(actual_jayson, actual_serde);
        }
        (Err(_), Err(_)) => {}
        (Ok(_), Err(_)) => panic!("jayson fails to deserialize but serde does not"),
        (Err(_), Ok(_)) => panic!("serde fails to deserialize but jayson does not"),
    }
}

#[track_caller]
fn assert_error_matches<T>(j: &str, expected: MyError)
where
    T: DeserializeFromValue<MyError> + PartialEq + std::fmt::Debug,
{
    let json: serde_json::Value = serde_json::from_str(j).unwrap();
    let actual: MyError = jayson::deserialize::<T, _, _>(json).unwrap_err();

    assert_eq!(actual, expected);
}
#[track_caller]
fn print_accumulated_error<T>(j: &str)
where
    T: DeserializeFromValue<AccumulatedErrors<MyError>> + PartialEq + std::fmt::Debug,
{
    let json: serde_json::Value = serde_json::from_str(j).unwrap();
    let actual: AccumulatedErrors<MyError> = jayson::deserialize::<T, _, _>(json).unwrap_err();

    println!("{actual:?}");
}

#[test]
fn test_de() {
    // arbitrary struct, roundtrip
    compare_with_serde_roundtrip(Example {
        x: "X".to_owned(),
        t1: Tag::A,
        t2: Box::new(Tag::B),
        n: Box::new(Nested {
            y: Some(vec!["Y".to_owned(), "Y".to_owned()]),
            z: None,
        }),
    });

    // struct rename all camel case, roundtrip
    compare_with_serde_roundtrip(RenamedAllCamelCaseStruct {
        renamed_field: true,
    });
    // struct rename all lower case, roundtrip
    compare_with_serde_roundtrip(RenamedAllLowerCaseStruct {
        renamed_field: true,
    });

    // enum rename all variants camel case, roundtrip
    compare_with_serde_roundtrip(RenamedAllCamelCaseEnum::SomeField { my_field: true });

    // struct with renamed field, roundtrip
    compare_with_serde_roundtrip(RenamedAllFieldsCamelCaseEnum::SomeField { my_field: true });

    // struct default attributes serde, roundtrip
    compare_with_serde_roundtrip(StructWithDefaultAttr {
        x: true,
        y: 1,
        z: None,
    });

    // struct default attributes, missing field
    compare_with_serde::<StructWithDefaultAttr>(
        r#"{
            "x": true,
            "y": 10
        }
        "#,
    );

    // struct default attribute using Default trait, missing field
    compare_with_serde::<StructWithTraitDefaultAttr>(r#"{ }"#);

    // enum with optional data inside variant, roundtrip
    compare_with_serde_roundtrip(EnumWithOptionData::A { x: None });

    // enum with optional data inside variant, missing field
    compare_with_serde::<EnumWithOptionData>(r#"{ "t": "A" }"#);

    // enum with optional and defaultable data inside variant, missing fields
    compare_with_serde::<EnumWithOptionData>(r#"{ "t": "B" }"#);

    // enum with optional and defaultable data inside variant, all fields present
    compare_with_serde::<EnumWithOptionData>(
        r#"{
            "t": "B",
            "x": null,
            "y": 10
        }
        "#,
    );

    // struct with renamed field, roundtrip
    compare_with_serde_roundtrip(StructWithRenamedField { x: true });

    // struct with deny_unknown_fields, with unknown fields
    compare_with_serde::<StructDenyUnknownFields>(
        r#"{
            "x": true,
            "y": 8
        }
        "#,
    );

    // struct with deny_unknown_fields, roundtrip
    compare_with_serde_roundtrip(StructDenyUnknownFields { x: true });

    // enum with deny_unknown_fields, with unknown fields
    compare_with_serde::<EnumDenyUnknownFields>(
        r#"{
            "t": "SomeField",
            "my_field": true,
            "other": true
        }
        "#,
    );

    // enum with deny_unknown_fields, missing tag
    compare_with_serde::<EnumDenyUnknownFields>(
        r#"{
            "my_field": true,
            "other": true
        }
        "#,
    );

    // enum with deny_unknown_fields, roundtrip 1
    compare_with_serde_roundtrip(EnumDenyUnknownFields::SomeField { my_field: true });

    // enum with deny_unknown_fields, roundtrip 2
    compare_with_serde_roundtrip(EnumDenyUnknownFields::Other {
        my_field: true,
        y: 8,
    });

    // struct with deny_unknown_fields with custom error function
    compare_with_serde::<StructDenyUnknownFieldsCustom>(
        r#"{
            "x": true,
            "y": 8
        }
        "#,
    );

    // struct with deny_unknown_fields with custom error function
    // assert error value is correct

    assert_error_matches::<StructDenyUnknownFieldsCustom>(
        r#"{
            "x": true,
            "y": 8
        }
        "#,
        unknown_field_error("y", ValuePointerRef::Origin),
    );

    // struct with deny_unknown_fields with custom error function
    compare_with_serde::<EnumDenyUnknownFieldsCustom>(
        r#"{
            "t": "SomeField",
            "my_field": true,
            "other": true
        }
        "#,
    );

    // enum with deny_unknown_fields with custom error function, error check
    assert_error_matches::<EnumDenyUnknownFieldsCustom>(
        r#"{
            "t": "SomeField",
            "my_field": true,
            "other": true
        }
        "#,
        unknown_field_error("other", ValuePointerRef::Origin),
    );

    // struct with custom missing field error, error check 1
    assert_error_matches::<StructMissingFieldError>(
        r#"{
            "y": true
        }
        "#,
        MyError::MissingField("lol".to_string()),
    );
    // struct with custom missing field error, error check 2
    assert_error_matches::<StructMissingFieldError>(
        r#"{
            "x": true
        }
        "#,
        MyError::CustomMissingField(1),
    );

    // enum with custom missing field error, error check 1
    assert_error_matches::<EnumMissingFieldError>(
        r#"{
            "t": "A"
        }
        "#,
        MyError::CustomMissingField(0),
    );

    // enum with custom missing field error, error check 2
    assert_error_matches::<EnumMissingFieldError>(
        r#"{
            "t": "B"
        }
        "#,
        MyError::MissingField("x".to_owned()),
    );

    // enum with renamed variants, roundtrip 1
    compare_with_serde_roundtrip(EnumRenamedVariant::A { x: true });
    // enum with renamed variants, roundtrip 2
    compare_with_serde_roundtrip(EnumRenamedVariant::B);

    // enum with renamed field, roundtrip
    compare_with_serde_roundtrip(EnumRenamedField::A { x: true });

    // enum with rename_all variant, roundtrip
    compare_with_serde_roundtrip(EnumRenamedAllVariant::P {
        water_potential: true,
    });

    // generic no bounds, roundtrip
    compare_with_serde_roundtrip(Generic::<EnumRenamedAllVariant> {
        some_field: EnumRenamedAllVariant::P {
            water_potential: true,
        },
    });

    // enum with deny_unknown_fields with custom error function, error check
    assert_error_matches::<EnumDenyUnknownFieldsCustom>(
        r#"{
            "t": "SomeField",
            "my_field": true,
            "other": true
        }
        "#,
        unknown_field_error("other", ValuePointerRef::Origin),
    );
}
#[test]
fn test_accumulated_errors() {
    // TODO: should be generic over all errors that can be merged into AccumulatedErrors
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, DeserializeFromValue)]
    #[jayson(error = AccumulatedErrors<MyError>, deny_unknown_fields)]
    struct S {
        x: u8,
        y: bool,
    }

    print_accumulated_error::<S>(
        r#"{
            "x": true,
            "z": true
        }
        "#,
    );
}
