#![allow(dead_code)]

use schematic::*;
use std::collections::HashMap;
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

#[derive(Default)]
pub struct Context {
    count: usize,
    root: PathBuf,
}

fn default_count(ctx: &Context) -> Option<usize> {
    Some(ctx.count * 2)
}

fn default_path(ctx: &Context) -> Option<PathBuf> {
    Some(ctx.root.join("sub"))
}

#[derive(Debug, Config)]
#[config(context = Context)]
pub struct ContextDefaults {
    #[setting(default = default_count)]
    count: usize,
    #[setting(default = default_path)]
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

#[derive(Config)]
pub struct NestedDefaults {
    #[setting(nested)]
    nested: NativeDefaults,
    #[setting(nested)]
    nested_opt: Option<NativeDefaults>,
    #[setting(nested)]
    nested_vec: Vec<NativeDefaults>,
    #[setting(nested)]
    nested_map: HashMap<String, NativeDefaults>,
}

#[test]
fn handles_nested_defaults() {
    let result = ConfigLoader::<NestedDefaults>::new(SourceFormat::Yaml)
        .load()
        .unwrap();

    assert!(result.config.nested_opt.is_none());
    assert!(!result.config.nested.boolean);
}
