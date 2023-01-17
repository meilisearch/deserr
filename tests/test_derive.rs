use deserr::{
    DefaultError, DefaultErrorContent, DeserializeError, DeserializeFromValue, ErrorKind,
    MergeWithError, ValueKind, ValuePointerRef,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[serde(tag = "sometag")]
#[deserr(tag = "sometag")]
enum Tag {
    A,
    B,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
enum Untagged {
    A,
    B,
}

fn unknown_field_error_gen<E>(k: &str, accepted: &[&str], location: deserr::ValuePointerRef) -> E
where
    E: DeserializeError,
{
    match E::error::<serde_json::Value>(None, ErrorKind::UnknownKey { key: k, accepted }, location)
    {
        Ok(e) => e,
        Err(e) => e,
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(deny_unknown_fields = unknown_field_error_gen)]
struct Example {
    x: String,
    t1: Tag,
    t2: Box<Tag>,
    ut1: Untagged,
    ut2: Box<Untagged>,
    n: Box<Nested>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
struct Nested {
    #[deserr(default)]
    y: Option<Vec<String>>,
    #[deserr(default)]
    z: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError)]
struct StructWithDefaultAttr {
    x: bool,
    #[serde(default = "create_default_u8")]
    #[deserr(default = create_default_u8())]
    y: u8,
    #[serde(default = "create_default_option_string")]
    #[deserr(default = create_default_option_string())]
    z: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError)]
struct StructWithTraitDefaultAttr {
    #[serde(default)]
    #[deserr(default)]
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
#[deserr(error = DefaultError, tag = "t")]
enum EnumWithOptionData {
    A {
        #[deserr(default)]
        x: Option<u8>,
    },
    B {
        #[serde(default = "create_default_option_string")]
        #[deserr(default = create_default_option_string())]
        x: Option<String>,
        #[serde(default = "create_default_u8")]
        #[deserr(default = create_default_u8())]
        y: u8,
    },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError, rename_all = camelCase)]
#[serde(rename_all = "camelCase")]
struct RenamedAllCamelCaseStruct {
    renamed_field: bool,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError, rename_all = lowercase)]
#[serde(rename_all = "lowercase")]
struct RenamedAllLowerCaseStruct {
    renamed_field: bool,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError, tag = "t", rename_all = camelCase)]
#[serde(tag = "t")]
#[serde(rename_all = "camelCase")]
enum RenamedAllCamelCaseEnum {
    SomeField { my_field: bool },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError, tag = "t")]
#[serde(tag = "t")]
enum RenamedAllFieldsCamelCaseEnum {
    #[deserr(rename_all = camelCase)]
    #[serde(rename_all = "camelCase")]
    SomeField { my_field: bool },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError)]
struct StructWithRenamedField {
    #[deserr(rename = "renamed_field")]
    #[serde(rename = "renamed_field")]
    x: bool,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError, rename_all = camelCase)]
struct StructWithRenamedFieldAndRenameAll {
    #[deserr(rename = "renamed_field")]
    #[serde(rename = "renamed_field")]
    x: bool,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError, deny_unknown_fields)]
#[serde(deny_unknown_fields)]
struct StructDenyUnknownFields {
    x: bool,
}

fn unknown_field_error(k: &str, _accepted: &[&str], location: ValuePointerRef) -> DefaultError {
    DefaultError {
        location: location.to_owned(),
        content: DefaultErrorContent::UnknownKey {
            key: k.to_owned(),
            accepted: vec!["don't know".to_string()],
        },
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError, deny_unknown_fields = unknown_field_error)]
#[serde(deny_unknown_fields)]
struct StructDenyUnknownFieldsCustom {
    x: bool,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError, tag = "t", deny_unknown_fields)]
#[serde(tag = "t", deny_unknown_fields)]
enum EnumDenyUnknownFields {
    SomeField { my_field: bool },
    Other { my_field: bool, y: u8 },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError, tag = "t", deny_unknown_fields = unknown_field_error)]
#[serde(tag = "t", deny_unknown_fields)]
enum EnumDenyUnknownFieldsCustom {
    SomeField { my_field: bool },
    Other { my_field: bool, y: u8 },
}
fn missing_x_field(_field_name: &str, location: ValuePointerRef) -> DefaultError {
    DefaultError {
        location: location.to_owned(),
        content: DefaultErrorContent::MissingField("lol".to_string()),
    }
}
fn custom_mising_field(_field_name: &str, location: ValuePointerRef) -> DefaultError {
    DefaultError {
        location: location.to_owned(),
        content: DefaultErrorContent::CustomMissingField(1),
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError)]
struct StructMissingFieldError {
    #[deserr(missing_field_error = missing_x_field)]
    x: bool,
    #[deserr(missing_field_error = custom_mising_field)]
    y: bool,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError, tag = "t")]
enum EnumMissingFieldError {
    A {
        #[deserr(missing_field_error = custom_mising_field)]
        x: bool,
    },
    B {
        x: bool,
    },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError, tag = "t")]
#[serde(tag = "t")]
enum EnumRenamedVariant {
    #[serde(rename = "Apple")]
    #[deserr(rename = "Apple")]
    A { x: bool },
    #[serde(rename = "Beta")]
    #[deserr(rename = "Beta")]
    B,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError, tag = "t")]
#[serde(tag = "t")]
enum EnumRenamedField {
    A {
        #[deserr(rename = "Xylem")]
        #[serde(rename = "Xylem")]
        x: bool,
    },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError, tag = "t")]
#[serde(tag = "t")]
enum EnumRenamedAllVariant {
    #[deserr(rename_all = camelCase)]
    #[serde(rename_all = "camelCase")]
    P { water_potential: bool },
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(error = DefaultError)]
struct Generic<A> {
    some_field: A,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(where_predicate = __Deserr_E: MergeWithError<DefaultError>, where_predicate = A: DeserializeFromValue<DefaultError>)]
struct Generic2<A> {
    #[deserr(error = DefaultError, default)]
    some_field: Option<A>,
}

fn map_option(x: Option<u8>) -> Option<u8> {
    match x {
        Some(0) => None,
        Some(x) => Some(x),
        None => Some(1),
    }
}
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
struct FieldMap {
    #[deserr(default, map = map_option)]
    some_field: Option<u8>,
}

// For AscDesc, we have __Deserr_E where __Deserr_E: MergeWithError<AscDescError>
// Then for the struct that contains AscDesc, we don't want to repeat this whole requirement
// so instead we do: AscDesc: DeserializeFromValue<__Deserr_E>
// but that's only if it's generic! If it's not, we don't even need to have any requirements

// #[deserr(where_predicates_from_fields)]

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, DeserializeFromValue)]
#[deserr(where_predicate = Option<u8> : DeserializeFromValue<__Deserr_E>)]
struct FieldConditions {
    #[deserr(default)]
    some_field: Option<u8>,
}

pub enum NeverError {}

fn parse_hello(b: bool) -> Result<Hello, NeverError> {
    match b {
        true => Ok(Hello::A),
        false => Ok(Hello::B),
    }
}
fn parse_hello2(b: bool) -> Result<Hello2, NeverError> {
    match b {
        true => Ok(Hello2::A),
        false => Ok(Hello2::B),
    }
}
fn parse_hello3(b: &str) -> Result<Hello3, DefaultError> {
    match b {
        "A" => Ok(Hello3::A),
        "B" => Ok(Hello3::B),
        _ => Err(DefaultError {
            location: ValuePointerRef::Origin.to_owned(),
            content: DefaultErrorContent::Unexpected("Hello3 from error".to_string()),
        }),
    }
}

#[derive(Debug, PartialEq, DeserializeFromValue)]
#[deserr(from(bool) = parse_hello -> NeverError)]
enum Hello {
    A,
    B,
}

#[derive(Debug, PartialEq, DeserializeFromValue)]
#[deserr(error = DefaultError, from(bool) = parse_hello2 -> NeverError)]
enum Hello2 {
    A,
    B,
}

#[derive(Debug, PartialEq, DeserializeFromValue)]
#[deserr(from(& String) = parse_hello3 -> DefaultError)]
enum Hello3 {
    A,
    B,
}

#[derive(Debug, PartialEq, DeserializeFromValue)]
#[deserr(where_predicate = Hello: DeserializeFromValue<__Deserr_E>)]
struct ContainsHello {
    _x: Hello,
}

#[derive(Debug, PartialEq, DeserializeFromValue)]
#[deserr(error = DefaultError)]
struct ContainsHello2 {
    _x: Hello,
}

#[derive(Debug, PartialEq, DeserializeFromValue)]
struct ContainsHello3 {
    #[deserr(needs_predicate)]
    _x: Hello,
}

struct MyValidationError;
impl MergeWithError<MyValidationError> for DefaultError {
    fn merge(
        _self_: Option<Self>,
        _other: MyValidationError,
        merge_location: ValuePointerRef,
    ) -> Result<Self, Self> {
        Err(DefaultError {
            location: merge_location.to_owned(),
            content: DefaultErrorContent::Validation,
        })
    }
}

fn validate_it(x: Validated, _location: ValuePointerRef) -> Result<Validated, MyValidationError> {
    if x.x as u16 > x.y {
        Err(MyValidationError)
    } else {
        Ok(x)
    }
}

fn validate_it2(
    x: Validated2,
    _location: ValuePointerRef,
) -> Result<Validated2, MyValidationError> {
    if x.x as u16 > x.y {
        Err(MyValidationError)
    } else {
        Ok(x)
    }
}

#[derive(Debug, DeserializeFromValue)]
#[deserr(validate = validate_it -> MyValidationError)]
struct Validated {
    x: u8,
    y: u16,
}

#[derive(Debug, DeserializeFromValue)]
#[deserr(error = DefaultError, validate = validate_it2 -> MyValidationError)]
struct Validated2 {
    x: u8,
    y: u16,
}

#[derive(Debug, PartialEq, Eq, DeserializeFromValue)]
pub struct From {
    #[deserr(from(&String) = u8_from_str -> MyParseIntError)]
    x: u8,
    y: u16,
}

pub struct MyParseIntError(std::num::ParseIntError);

fn usize_from_str(x: &str) -> Result<usize, MyParseIntError> {
    usize::from_str(x).map_err(MyParseIntError)
}
fn u8_from_str(x: &str) -> Result<u8, MyParseIntError> {
    u8::from_str(x).map_err(MyParseIntError)
}

impl MergeWithError<MyParseIntError> for DefaultError {
    fn merge(
        _self_: Option<Self>,
        other: MyParseIntError,
        merge_location: ValuePointerRef,
    ) -> Result<Self, Self> {
        Err(DefaultError {
            location: merge_location.to_owned(),
            content: DefaultErrorContent::Unexpected(other.0.to_string()),
        })
    }
}

#[derive(Debug, PartialEq, Eq, DeserializeFromValue)]
pub struct From2 {
    #[deserr(from(&String) = u8_from_str -> MyParseIntError)]
    x: u8,
    #[deserr(default = 3, from(&String) = usize_from_str -> MyParseIntError)]
    y: usize,
}

impl MergeWithError<NeverError> for DefaultError {
    fn merge(
        _self_: Option<Self>,
        _other: NeverError,
        _merge_location: ValuePointerRef,
    ) -> Result<Self, Self> {
        unreachable!()
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize, DeserializeFromValue)]
pub struct Skipped1 {
    #[deserr(skip)]
    #[serde(skip)]
    x: u8,
    #[deserr(skip)]
    #[serde(skip)]
    y: Option<u8>,

    z: bool,
}

fn default_skipped_x() -> u8 {
    1
}
fn default_skipped_y() -> Option<u8> {
    Some(3)
}

#[derive(Debug, PartialEq, Eq, Deserialize, DeserializeFromValue)]
#[deserr(deny_unknown_fields)]
pub struct Skipped2 {
    #[deserr(default = default_skipped_x(), skip)]
    #[serde(default = "default_skipped_x", skip)]
    x: u8,
    #[deserr(skip, default = default_skipped_y())]
    #[serde(skip, default = "default_skipped_y")]
    y: Option<u8>,

    z: bool,
}

#[track_caller]
fn compare_with_serde_roundtrip<T>(x: T)
where
    T: Serialize + DeserializeFromValue<DefaultError> + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_value(&x).unwrap();
    let result: T = deserr::deserialize(json).unwrap();

    assert_eq!(result, x);
}

#[track_caller]
fn compare_with_serde<T>(j: &str)
where
    T: DeserializeOwned + DeserializeFromValue<DefaultError> + PartialEq + std::fmt::Debug,
{
    let json: Value = serde_json::from_str(j).unwrap();

    let actual_serde: Result<T, _> = serde_json::from_str(j);
    let actual_deserr: Result<T, _> = deserr::deserialize(json);

    match (actual_serde, actual_deserr) {
        (Ok(actual_serde), Ok(actual_deserr)) => {
            assert_eq!(actual_deserr, actual_serde);
        }
        (Err(_), Err(_)) => {}
        (Ok(_), Err(e)) => {
            panic!("deserr fails to deserialize but serde does not with error: {e:?}")
        }
        (Err(e), Ok(_)) => {
            panic!("serde fails to deserialize but deserr does not with error: {e:?}")
        }
    }
}

#[track_caller]
fn assert_error_matches<T, E>(j: &str, expected: E)
where
    E: DeserializeError + PartialEq + std::fmt::Debug,
    T: DeserializeFromValue<E> + std::fmt::Debug,
{
    let json: Value = serde_json::from_str(j).unwrap();
    let actual: E = deserr::deserialize::<T, _, _>(json).unwrap_err();

    assert_eq!(actual, expected);
}

#[track_caller]
fn assert_ok_matches<T, E>(j: &str, expected: T)
where
    E: DeserializeError + PartialEq + std::fmt::Debug,
    T: DeserializeFromValue<E> + std::fmt::Debug + PartialEq,
{
    let json: Value = serde_json::from_str(j).unwrap();
    let actual: T = deserr::deserialize::<T, _, E>(json).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn test_de() {
    // arbitrary struct, roundtrip
    compare_with_serde_roundtrip(Example {
        x: "X".to_owned(),
        t1: Tag::A,
        t2: Box::new(Tag::B),
        ut1: Untagged::A,
        ut2: Box::new(Untagged::B),
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

    // struct with renamed field and rename_all rule, roundtrip
    compare_with_serde_roundtrip(StructWithRenamedFieldAndRenameAll { x: true });
    assert_ok_matches(
        r#"{ "renamed_field": true }"#,
        StructWithRenamedFieldAndRenameAll { x: true },
    );

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

    assert_error_matches::<StructDenyUnknownFieldsCustom, DefaultError>(
        r#"{
            "x": true,
            "y": 8
        }
        "#,
        unknown_field_error("y", &[], ValuePointerRef::Origin),
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
    assert_error_matches::<EnumDenyUnknownFieldsCustom, DefaultError>(
        r#"{
            "t": "SomeField",
            "my_field": true,
            "other": true
        }
        "#,
        unknown_field_error("other", &[], ValuePointerRef::Origin),
    );

    // struct with custom missing field error, error check 1
    assert_error_matches::<StructMissingFieldError, DefaultError>(
        r#"{
            "y": true
        }
        "#,
        DefaultError {
            location: ValuePointerRef::Origin.to_owned(),
            content: DefaultErrorContent::MissingField("lol".to_string()),
        },
    );
    // struct with custom missing field error, error check 2
    assert_error_matches::<StructMissingFieldError, DefaultError>(
        r#"{
            "x": true
        }
        "#,
        DefaultError {
            location: ValuePointerRef::Origin.to_owned(),
            content: DefaultErrorContent::CustomMissingField(1),
        },
    );

    // enum with custom missing field error, error check 1
    assert_error_matches::<EnumMissingFieldError, DefaultError>(
        r#"{
            "t": "A"
        }
        "#,
        DefaultError {
            location: ValuePointerRef::Origin.to_owned(),
            content: DefaultErrorContent::CustomMissingField(1),
        },
    );

    // enum with custom missing field error, error check 2
    assert_error_matches::<EnumMissingFieldError, DefaultError>(
        r#"{
            "t": "B"
        }
        "#,
        DefaultError {
            location: ValuePointerRef::Origin.to_owned(),
            content: DefaultErrorContent::MissingField("x".to_owned()),
        },
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
    assert_error_matches::<EnumDenyUnknownFieldsCustom, DefaultError>(
        r#"{
            "t": "SomeField",
            "my_field": true,
            "other": true
        }
        "#,
        unknown_field_error("other", &[], ValuePointerRef::Origin),
    );

    assert_ok_matches::<Hello, DefaultError>("true", Hello::A);

    assert_error_matches::<Validated, DefaultError>(
        r#"{
            "x": 2,
            "y": 1
        }
        "#,
        DefaultError {
            location: ValuePointerRef::Origin.to_owned(),
            content: DefaultErrorContent::Validation,
        },
    );

    assert_ok_matches::<FieldMap, DefaultError>(
        r#"{ "some_field": null }"#,
        FieldMap {
            some_field: Some(1),
        },
    );

    assert_ok_matches::<FieldMap, DefaultError>(
        r#"{  }"#,
        FieldMap {
            some_field: Some(1),
        },
    );

    assert_ok_matches::<FieldMap, DefaultError>(
        r#"{ "some_field": 0 }"#,
        FieldMap { some_field: None },
    );

    assert_ok_matches::<FieldMap, DefaultError>(
        r#"{ "some_field": 2 }"#,
        FieldMap {
            some_field: Some(2),
        },
    );

    assert_ok_matches::<From, DefaultError>(r#"{ "x": "2", "y": 14 }"#, From { x: 2, y: 14 });
    assert_error_matches::<From, DefaultError>(
        r#"{ "x": "hello", "y": 14 }"#,
        DefaultError {
            location: ValuePointerRef::Origin.push_key("x").to_owned(),
            content: DefaultErrorContent::Unexpected(String::from("invalid digit found in string")),
        },
    );
    assert_error_matches::<From, DefaultError>(
        r#"{ "x": 2, "y": 14 }"#,
        DefaultError {
            location: ValuePointerRef::Origin.push_key("x").to_owned(),
            content: DefaultErrorContent::IncorrectValueKind {
                accepted: vec![ValueKind::String],
            },
        },
    );

    // just want to ensure that it works even though we're going to ask multiple time the same constraint
    assert_ok_matches::<From2, DefaultError>(r#"{ "x": "2", "y": "14" }"#, From2 { x: 2, y: 14 });
    assert_ok_matches::<From2, DefaultError>(r#"{ "x": "2" }"#, From2 { x: 2, y: 3 });

    // Struct skipped fields
    compare_with_serde::<Skipped1>(
        r#"{
            "x": 89,
            "z": true
        }
        "#,
    );

    // Struct skipped fields
    compare_with_serde::<Skipped2>(
        r#"{
            "z": true
        }
        "#,
    );

    assert_error_matches::<Skipped2, DefaultError>(
        r#"{ "x": 78, "z": true }"#,
        DefaultError {
            location: ValuePointerRef::Origin.to_owned(),
            content: DefaultErrorContent::UnknownKey {
                key: "x".to_owned(),
                accepted: vec!["z".to_owned()],
            },
        },
    );
}
