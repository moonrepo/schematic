use schematic::*;

#[derive(Debug, Config)]
pub struct NativeDefaults {
    boolean: bool,
    string: String,
    number: usize,
    vector: Vec<String>,
}

#[tokio::test]
async fn uses_native_defaults() {
    let result = ConfigLoader::<NativeDefaults>::new(SourceFormat::Yaml)
        .load()
        .await
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "");
    assert_eq!(result.config.number, 0);
    assert_eq!(result.config.vector, Vec::<String>::new());
}

#[derive(Debug, Config)]
pub struct CustomDefaults {
    #[setting(default = true)]
    boolean: bool,
    #[setting(default = "foo")]
    string: String,
    #[setting(default = 123)]
    number: usize,
    #[setting(default = Vec::from([1, 2, 3, 4]))]
    vector: Vec<usize>,
}

#[tokio::test]
async fn uses_custom_setting_defaults() {
    let result = ConfigLoader::<CustomDefaults>::new(SourceFormat::Yaml)
        .load()
        .await
        .unwrap();

    assert!(result.config.boolean);
    assert_eq!(result.config.string, "foo");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.vector, vec![1, 2, 3, 4]);
}
