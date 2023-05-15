#![allow(dead_code)]

use schematic::*;

fn test_string(value: &String) -> Result<(), ValidateError> {
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
    let error = ConfigLoader::<Validate>::new(SourceFormat::Json)
        .code(r#"{ "string1": "abc" }"#)
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate config. \n  string1: invalid string"
    )
}

#[test]
fn errors_for_nested_field() {
    let error = ConfigLoader::<Validate>::new(SourceFormat::Json)
        .code(r#"{ "string1": "abc", "nested": { "string2": "abc" } }"#)
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate config. \n  string1: invalid string\n  nested.string2: invalid string"
    )
}

fn test_string_path(_: &String) -> Result<(), ValidateError> {
    Err(ValidateError::with_segments(
        "invalid string",
        vec![Segment::Index(1), Segment::Key("foo".to_owned())],
    ))
}

#[derive(Config)]
pub struct ValidatePath {
    #[setting(validate = test_string_path)]
    string: String,
}

#[test]
fn can_customize_path() {
    let error = ConfigLoader::<ValidatePath>::new(SourceFormat::Json)
        .code(r#"{ "string": "abc" }"#)
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate config. \n  string[1].foo: invalid string"
    )
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
    #[setting(validate = validate::in_range(1, 5))]
    range: i32,
}

#[test]
fn runs_the_validator_funcs() {
    let error = ConfigLoader::<ValidateFuncs>::new(SourceFormat::Json)
        .code(r#"{}"#)
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate config. \n  contains: does not contain \"foo\"\n  email: not a valid email: value is empty\n  ip: not a valid IP address\n  ip_v4: not a valid IPv4 address\n  ip_v6: not a valid IPv6 address\n  regex: does not match pattern /^foo$/\n  min: length is lower than 1\n  len: length is lower than 1\n  url: not a valid url: relative URL without a base\n  range: lower than 1"
    )
}

#[derive(Config)]
pub struct ValidateOptional {
    #[setting(validate = test_string)]
    string1: Option<String>,
}

#[test]
fn skips_optional_fields() {
    let result = ConfigLoader::<ValidateOptional>::new(SourceFormat::Json)
        .code(r#"{}"#)
        .unwrap()
        .load();

    assert!(result.is_ok());
}

#[test]
fn errors_for_optional_field() {
    let error = ConfigLoader::<ValidateOptional>::new(SourceFormat::Json)
        .code(r#"{ "string1": "abc" }"#)
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate config. \n  string1: invalid string"
    )
}
