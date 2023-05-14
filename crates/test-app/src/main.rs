use miette::Result;
use schematic::{Config, ConfigLoader};
use serde::Serialize;

#[derive(Debug, Config, Serialize)]
struct TestConfig {
    string: String,
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
