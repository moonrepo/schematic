#![allow(dead_code)]

use schematic::*;
use std::collections::HashMap;

fn test_string<T, C>(value: &str, _: &T, _: &C, _: bool) -> ValidateResult {
    if value.is_empty() {
        return Ok(());
    }

    Err(ValidateError::new("invalid string"))
}

#[derive(Config)]
pub struct NestedValidate {
    #[setting(validate = test_string)]
    string2: String,
}

#[derive(Config)]
pub struct Validate {
    #[setting(validate = test_string)]
    string1: String,
    #[setting(nested)]
    nested: NestedValidate,
}

#[test]
fn errors_for_field() {
    let error = ConfigLoader::<Validate>::new()
        .code(r#"{ "string1": "abc" }"#, Format::Json)
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate Validate. \n  string1: invalid string"
    )
}

#[test]
fn errors_for_nested_field() {
    let error = ConfigLoader::<Validate>::new()
        .code(
            r#"{ "string1": "abc", "nested": { "string2": "abc" } }"#,
            Format::Json,
        )
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate Validate. \n  string1: invalid string\n  nested.string2: invalid string"
    )
}

fn test_string_path<T, C>(_: &String, _: &T, _: &C) -> Result<(), ValidateError> {
    Err(ValidateError::with_segments(
        "invalid string",
        [PathSegment::Index(1), PathSegment::Key("foo".to_owned())],
    ))
}

#[derive(Config)]
pub struct ValidateFuncs {
    #[setting(validate = validate::alphanumeric)]
    alnum: String,
    #[setting(validate = validate::ascii)]
    ascii: String,
    #[setting(validate = validate::contains("foo"))]
    contains: String,
    #[setting(validate = validate::email)]
    email: String,
    #[setting(validate = validate::ip)]
    ip: String,
    #[setting(validate = validate::ip_v4)]
    ip_v4: String,
    #[setting(validate = validate::ip_v6)]
    ip_v6: String,
    #[setting(validate = validate::regex("^foo$"))]
    regex: String,
    #[setting(validate = validate::min_length(1))]
    min: String,
    #[setting(validate = validate::max_length(1))]
    max: String,
    #[setting(validate = validate::in_length(1, 5))]
    len: String,
    #[setting(validate = validate::url)]
    url: String,
    #[setting(validate = validate::url_secure)]
    url_secure: String,
    #[setting(validate = validate::in_range(1, 5))]
    range: i32,
    #[setting(validate = validate::extends_string)]
    ext_str: String,
    #[setting(validate = validate::extends_list)]
    ext_list: Vec<String>,
    #[setting(validate = validate::extends_from)]
    ext_from: ExtendsFrom,
}

#[test]
fn runs_the_validator_funcs() {
    let error = ConfigLoader::<ValidateFuncs>::new()
        .code(r#"{}"#, Format::Json)
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate ValidateFuncs. \n  contains: does not contain \"foo\"\n  email: not a valid email: value is empty\n  ip: not a valid IP address\n  ip_v4: not a valid IPv4 address\n  ip_v6: not a valid IPv6 address\n  regex: does not match pattern /^foo$/\n  min: length is lower than 1\n  len: length is lower than 1\n  url: not a valid url: relative URL without a base\n  url_secure: not a valid url: relative URL without a base\n  range: lower than 1\n  ext_str: only file paths and URLs can be extended"
    )
}

#[derive(Config)]
pub struct ValidateOptional {
    #[setting(validate = test_string)]
    string1: Option<String>,
}

#[test]
fn skips_optional_fields() {
    let result = ConfigLoader::<ValidateOptional>::new()
        .code(r#"{}"#, Format::Json)
        .unwrap()
        .load();

    assert!(result.is_ok());
}

#[test]
fn errors_for_optional_field() {
    let error = ConfigLoader::<ValidateOptional>::new()
        .code(r#"{ "string1": "abc" }"#, Format::Json)
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate ValidateOptional. \n  string1: invalid string"
    )
}

#[derive(Config)]
pub struct ValidateCollections {
    #[setting(nested)]
    list: Vec<NestedValidate>,
    #[setting(nested)]
    map: HashMap<String, NestedValidate>,
}

#[test]
fn errors_for_nested_field_collections() {
    let error = ConfigLoader::<ValidateCollections>::new()
        .code(
            r#"{ "list": [ {"string2": "abc"} ], "map": { "key": {"string2": "abc"} } }"#,
            Format::Json,
        )
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate ValidateCollections. \n  list[0].string2: invalid string\n  map.key.string2: invalid string"
    )
}

#[derive(Config)]
pub struct ValidateRequired {
    #[setting(required)]
    string: Option<String>,
}

#[derive(Config)]
pub enum ValidateEnumRequired {
    Optional(Option<String>),
    #[setting(required)]
    Required(Option<String>),
}

#[test]
fn doesnt_error_if_required_and_notempty() {
    let result = ConfigLoader::<ValidateRequired>::new()
        .code(r#"{ "string": "abc" }"#, Format::Json)
        .unwrap()
        .load();

    assert!(result.is_ok());

    let result = ConfigLoader::<ValidateEnumRequired>::new()
        .code(r#"{ "required": "abc" }"#, Format::Json)
        .unwrap()
        .load();

    assert!(result.is_ok());

    let result = ConfigLoader::<ValidateEnumRequired>::new()
        .code(r#"{ "optional": null }"#, Format::Json)
        .unwrap()
        .load();

    assert!(result.is_ok());
}

#[test]
fn errors_if_required_and_empty() {
    let error = ConfigLoader::<ValidateRequired>::new()
        .code(r#"{}"#, Format::Json)
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate ValidateRequired. \n  string: this setting is required"
    );

    let error = ConfigLoader::<ValidateEnumRequired>::new()
        .code(r#"{ "required": null }"#, Format::Json)
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate ValidateEnumRequired. \n  required: this setting is required"
    );
}
