#![allow(dead_code)]

use schematic::*;

fn validate_string<T, C>(value: &str, _: &T, _: &C, _: bool) -> ValidateResult {
    if value != "FOO" {
        return Ok(());
    }

    Err(ValidateError::new("invalid string FOO"))
}

fn transform_string(value: String, _ctx: &()) -> TransformResult<String> {
    Ok(value.to_uppercase())
}

fn transform_number(value: usize, _ctx: &()) -> TransformResult<usize> {
    Ok(value * 2)
}

fn transform_vec(mut value: Vec<String>, _ctx: &()) -> TransformResult<Vec<String>> {
    value.push("item".into());
    Ok(value)
}

#[derive(Debug, Config)]
pub struct Transforms {
    #[setting(transform = transform_string, validate = validate_string)]
    string: String,
    #[setting(transform = transform_number)]
    number: usize,
    #[setting(transform = transform_vec)]
    vector: Vec<String>,
}

#[test]
fn transforms_values() {
    let result = ConfigLoader::<Transforms>::new()
        .code("string: abc", Format::Yaml)
        .unwrap()
        .code("number: 123", Format::Yaml)
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.string, "ABC");
    assert_eq!(result.config.number, 246);
    assert_eq!(result.config.vector, vec!["item"]);
}

#[test]
fn errors_for_transformed_field() {
    let error = ConfigLoader::<Transforms>::new()
        .code("string: foo", Format::Yaml)
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate Transforms. \n  string: invalid string FOO"
    )
}
