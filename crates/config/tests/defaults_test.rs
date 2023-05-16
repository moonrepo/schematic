use schematic::*;
use std::path::PathBuf;

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
    #[setting(default_str = "foo")]
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

#[derive(Default)]
pub struct Context {
    count: usize,
    root: PathBuf,
}

fn default_count(ctx: &Context) -> usize {
    ctx.count * 2
}

fn default_path(ctx: &Context) -> PathBuf {
    ctx.root.join("sub")
}

#[derive(Debug, Config)]
#[config(context = Context)]
pub struct ContextDefaults {
    #[setting(default_fn = default_count)]
    count: usize,
    #[setting(default_fn = default_path)]
    path: PathBuf,
}

#[test]
fn sets_defaults_from_context() {
    let context = Context {
        count: 5,
        root: PathBuf::from("/root"),
    };
    let result = ConfigLoader::<ContextDefaults>::new(SourceFormat::Yaml)
        .load_with_context(&context)
        .unwrap();

    assert_eq!(result.config.count, 10);
    assert_eq!(result.config.path, PathBuf::from("/root/sub"));
}
