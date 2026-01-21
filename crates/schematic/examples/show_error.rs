#![allow(unused)]

use schematic::*;

#[derive(Config)]
pub struct SomeConfig {
    foo: String,
    bar: usize,
}

#[derive(Config)]
#[config(serde(untagged))]
enum Untagged {
    Foo,
    Bar(bool),
    #[setting(rename = "bazzer")]
    Baz(usize, String),
    #[setting(nested)]
    Qux(SomeConfig),
}

fn main() {
    let json = r#""invalid_string_value""#;
    let result: Result<PartialUntagged, _> = serde_json::from_str(json);
    println!("Error:\n{}", result.unwrap_err());
}
