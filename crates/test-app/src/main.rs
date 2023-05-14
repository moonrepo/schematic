use miette::Result;
use schematic::{Config, ConfigLoader, ValidateError};
use serde::Serialize;

fn validate_string(value: &str) -> Result<(), ValidateError> {
    // Err(ValidateError::new("This string is ugly!"))
    Ok(())
}

fn validate_number(value: &usize) -> Result<(), ValidateError> {
    Err(ValidateError::new("Nah, we don't accept numbers."))
    // Ok(())
}

#[derive(Debug, Config, Serialize)]
pub struct NestedConfig {
    #[setting(validate = validate_string)]
    string: String,
    #[setting(validate = validate_number)]
    number: usize,
}

#[derive(Debug, Config, Serialize)]
struct TestConfig {
    #[setting(validate = validate_string)]
    string: String,
    // #[setting(validate = validate_number)]
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
