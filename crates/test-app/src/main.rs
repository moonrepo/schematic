use miette::Result;
use schematic::{Config, ConfigLoader, ValidateError};
use serde::Serialize;

fn validate_string<D, C>(_: &str, _: &D, _: &C) -> Result<(), ValidateError> {
    use schematic::Segment;
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
        // .code(r#"{ "string": "abc", "other": 123 }"#)?
        // .code("{\n  \"string\": 123\n}")?
        .code("{\n  \"string\": \"\" \n}")?
        .load()?;

    dbg!(&config.config.string);
    dbg!(&config.layers);

    println!("{}", serde_json::to_string_pretty(&config).unwrap());

    Ok(())
}
