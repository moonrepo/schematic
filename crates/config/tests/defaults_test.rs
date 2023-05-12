use schematic::*;

#[derive(Debug, Config)]
pub struct NativeDefaults {
    boolean: bool,
    string: String,
    number: usize,
    vector: Vec<String>,
}

#[test]
fn uses_native_defaults() {
    let result = ConfigLoader::<NativeDefaults>::new(SourceFormat::Yaml)
        .load()
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

#[test]
fn uses_custom_setting_defaults() {
    let result = ConfigLoader::<CustomDefaults>::new(SourceFormat::Yaml)
        .load()
        .unwrap();

    assert!(result.config.boolean);
    assert_eq!(result.config.string, "foo");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.vector, vec![1, 2, 3, 4]);
}

#[derive(Debug, Config)]
pub struct ReqOptDefaults {
    required: usize,
    #[setting(default = 123)]
    required_with_default: usize,

    optional: Option<usize>,
    // #[setting(default = 123)]
    // optional_with_default: Option<usize>,
}

#[test]
fn handles_required_optional_defaults() {
    let result = ConfigLoader::<ReqOptDefaults>::new(SourceFormat::Yaml)
        .load()
        .unwrap();

    assert_eq!(result.config.required, 0);
    assert_eq!(result.config.required_with_default, 123);
    assert_eq!(result.config.optional, None);
}

#[test]
fn can_overwrite_optional_fields() {
    let result = ConfigLoader::<ReqOptDefaults>::new(SourceFormat::Yaml)
        .code("required: 789\noptional: 456")
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.required, 789);
    assert_eq!(result.config.required_with_default, 123);
    assert_eq!(result.config.optional, Some(456));
}
