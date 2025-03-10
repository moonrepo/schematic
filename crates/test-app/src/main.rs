use chrono::NaiveDateTime;
use miette::Result;
use rust_decimal::Decimal;
use schematic::{Config, ConfigLoader, Format, ValidateError};
use serde::Serialize;
use std::collections::HashMap;

fn validate_string<D, C>(_: &str, _: &D, _: &C, _: bool) -> Result<(), ValidateError> {
    use schematic::PathSegment;
    Err(ValidateError::with_segments(
        "This string is ugly!",
        vec![PathSegment::Index(1), PathSegment::Key("foo".to_owned())],
    ))
    // Ok(())
}

fn validate_number<D, C>(_: &usize, _: &D, _: &C, _: bool) -> Result<(), ValidateError> {
    Err(ValidateError::new("Nah, we don't accept numbers."))
    // Ok(())
}

#[derive(Debug, Config, Serialize)]
pub struct NestedConfig {
    #[setting(validate = validate_string)]
    string2: String,
    #[setting(validate = validate_number)]
    number2: usize,
}

#[derive(Debug, Config, Serialize)]
struct TestConfig {
    #[setting(validate = validate_string, env = "TEST_VAR")]
    string: String,
    #[setting(validate = validate_number)]
    number: usize,
    #[setting(nested)]
    nested: NestedConfig,
    datetime: NaiveDateTime,
    decimal: Decimal,
    // regex: Regex,
    map: HashMap<String, usize>,
}

fn main() -> Result<()> {
    dbg!(TestConfig::settings());

    let config = ConfigLoader::<TestConfig>::new()
        // .code(r#"{ "string": "abc", "other": 123 }"#, Format::Json)? // parse error
        // .code("{\n  \"string\": 123\n}", Format::Json)? // parse error
        .code("{\n  \"string\": \"\", \"number\": 1 \n}", Format::Json)? // validate error
        .set_help("let's go!")
        .load()?;

    dbg!(&config.config.string);
    dbg!(&config.layers);

    println!("{}", serde_json::to_string_pretty(&config).unwrap());

    Ok(())
}
