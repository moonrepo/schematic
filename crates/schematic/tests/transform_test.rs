#![allow(dead_code)]

use schematic::*;
use std::collections::HashMap;

fn validate_string<T, C>(value: &str, _: &T, _: &C, _: bool) -> ValidateResult {
    if value == "FOO" {
        return Err(ValidateError::new("invalid string FOO"));
    }

    Ok(())
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

#[derive(Debug, Config, PartialEq)]
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
        .code("string: abc", "code.yaml")
        .unwrap()
        .code("number: 123", "code.yaml")
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.string, "ABC");
    assert_eq!(result.config.number, 246);
    assert_eq!(result.config.vector, vec!["item"]);
}

#[test]
fn errors_for_invalid_transformed_field() {
    let error = ConfigLoader::<Transforms>::new()
        .code("string: foo", "code.yaml")
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate code.yaml. \n  string: invalid string FOO"
    )
}

#[derive(Debug, Config)]
pub struct TransformsOptional {
    #[setting(transform = transform_string)]
    string: Option<String>,
    #[setting(transform = transform_number)]
    number: Option<usize>,
    #[setting(transform = transform_vec)]
    vector: Option<Vec<String>>,
}

#[test]
fn transforms_optional_values() {
    let result = ConfigLoader::<TransformsOptional>::new()
        .code("string: abc", "code.yaml")
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.string, Some("ABC".to_owned()));
    assert_eq!(result.config.number, None);
    assert_eq!(result.config.vector, None);
}

fn transform_nested_vec(
    mut value: Vec<PartialTransforms>,
    _ctx: &(),
) -> TransformResult<Vec<PartialTransforms>> {
    value.iter_mut().for_each(|v| {
        v.number = Some(1);
    });

    Ok(value)
}

fn transform_nested_map(
    value: HashMap<usize, PartialTransforms>,
    _ctx: &(),
) -> TransformResult<HashMap<usize, PartialTransforms>> {
    Ok(value)
}

#[derive(Debug, Config)]
pub struct TransformsNested {
    #[setting(nested, transform = transform_nested_vec)]
    list: Vec<Transforms>,
    #[setting(nested, transform = transform_nested_map)]
    map: HashMap<usize, Transforms>,
}

#[test]
fn transforms_nested_values() {
    let result = ConfigLoader::<TransformsNested>::new()
        .code(
            r#"
list:
  - string: xyz"#,
            "code.yaml",
        )
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(
        result.config.list,
        vec![Transforms {
            string: "XYZ".to_owned(),
            number: 1,
            vector: vec!["item".to_owned()],
        }]
    );
}

// A failing transform names the setting it was for, as the error alone may
// not say which
mod errors {
    use super::*;

    fn fail(_: String, _: &()) -> TransformResult<String> {
        Err(HandlerError::new("nope"))
    }

    #[derive(Debug, Config)]
    pub struct Failing {
        #[setting(transform = fail)]
        name: String,
    }

    #[derive(Debug, Config)]
    #[serde(rename_all = "lowercase")]
    pub enum FailingVariant {
        File {
            #[setting(transform = fail)]
            path: String,
        },
        #[setting(transform = fail)]
        Tuple(String),
    }

    #[derive(Debug, Config)]
    pub struct Variants {
        #[setting(nested)]
        source: Option<FailingVariant>,
    }

    fn load<T: Config>(code: &str) -> String {
        ConfigLoader::<T>::new()
            .code(code, "code.json")
            .unwrap()
            .load()
            .err()
            .unwrap()
            .to_full_string()
    }

    #[test]
    fn names_the_setting() {
        assert_eq!(
            load::<Failing>(r#"{ "name": "abc" }"#),
            "Failed to transform name. nope"
        );
    }

    #[test]
    fn names_the_field_of_a_variant() {
        assert_eq!(
            load::<Variants>(r#"{ "source": { "file": { "path": "abc" } } }"#),
            "Failed to transform file.path. nope"
        );
    }

    #[test]
    fn names_a_variant() {
        assert_eq!(
            load::<Variants>(r#"{ "source": { "tuple": "abc" } }"#),
            "Failed to transform tuple. nope"
        );
    }
}
