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
