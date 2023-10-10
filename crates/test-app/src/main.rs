use chrono::NaiveDateTime;
use miette::Result;
use rust_decimal::Decimal;
use schematic::{Config, ConfigLoader, Format, ValidateError};
use serde::Serialize;

fn validate_string<D, C>(_: &str, _: &D, _: &C) -> Result<(), ValidateError> {
    use schematic::PathSegment;
    Err(ValidateError::with_segments(
        "This string is ugly!",
        vec![PathSegment::Index(1), PathSegment::Key("foo".to_owned())],
    ))
    // Ok(())
}

fn validate_number<D, C>(_: &usize, _: &D, _: &C) -> Result<(), ValidateError> {
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
    #[setting(validate = validate_string)]
    string: String,
    #[setting(validate = validate_number)]
    number: usize,
    #[setting(nested)]
    nested: NestedConfig,
    datetime: NaiveDateTime,
    decimal: Decimal,
    // regex: Regex,
}

fn main() -> Result<()> {
    let config = ConfigLoader::<TestConfig>::new()
        // .code(r#"{ "string": "abc", "other": 123 }"#, Format::Json)?
        // .code("{\n  \"string\": 123\n}", Format::Json)?
        .code("{\n  \"string\": \"\" \n}", Format::Json)?
        .load()?;

    dbg!(&config.config.string);
    dbg!(&config.layers);

    println!("{}", serde_json::to_string_pretty(&config).unwrap());

    Ok(())
}
