use miette::Result;
use schematic::{Config, ConfigLoader, Segment, ValidateError};
use serde::Serialize;

fn validate_string<D, C>(_: &str, _: &D, _: &C) -> Result<(), ValidateError> {
    Err(ValidateError::with_segments(
        "This string is ugly!",
        vec![Segment::Index(1), Segment::Key("foo".to_owned())],
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
#[config(file = "test.yml")]
struct TestConfig {
    #[setting(validate = validate_string)]
    string: String,
    #[setting(validate = validate_number)]
    number: usize,
    #[setting(nested)]
    nested: NestedConfig,
}

fn main() -> Result<()> {
    let config = ConfigLoader::<TestConfig>::json()
        //.code(r#"{ "string": "abc", "other": 123 }"#)?
        .code(r#"{ "string": "abc" }"#)?
        .load()?;

    dbg!(&config.config.string);
    dbg!(&config.sources);

    println!("{}", serde_json::to_string_pretty(&config).unwrap());

    Ok(())
}
