use schematic::*;

#[derive(Debug, Config)]
pub struct NestedConfig {
    setting: bool,
}

#[derive(Debug, Config)]
pub struct Config {
    setting: bool,
    #[setting(nested)]
    nested: NestedConfig,
}

#[test]
fn invalid_type() {
    //     let a = r"
    //     [nested]
    //     setting = 'foo'
    // ";
    let a = r#"{ "setting": 123 }"#;

    let result = ConfigLoader::<Config>::new(SourceFormat::Json)
        .code(a)
        .unwrap()
        .load()
        .unwrap();
}
