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
    float32: f32,
    float64: f64,
    #[allow(clippy::box_collection)]
    boxed: Box<String>,
}

#[test]
fn uses_native_defaults() {
    let result = ConfigLoader::<NativeDefaults>::new().load().unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "");
    assert_eq!(result.config.number, 0);
    assert_eq!(result.config.vector, Vec::<String>::new());
    assert_eq!(result.config.float32, 0.0);
    assert_eq!(result.config.float64, 0.0);
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
    #[setting(default = 1.32)]
    float: f32,
}

#[test]
fn uses_custom_setting_defaults() {
    let result = ConfigLoader::<CustomDefaults>::new().load().unwrap();

    assert!(result.config.boolean);
    assert_eq!(result.config.string, "foo");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.vector, vec![1, 2, 3, 4]);
    assert_eq!(result.config.float, 1.32);
}

fn default_int(_ctx: &()) -> DefaultValueResult<usize> {
    Ok(Some(456))
}

#[derive(Debug, Config)]
pub struct ReqOptDefaults {
    required: usize,
    #[setting(default = 123)]
    required_with_default: usize,
    #[setting(default = default_int)]
    required_with_default_fn: usize,

    optional: Option<usize>,
    #[setting(default = 123)]
    optional_with_default: Option<usize>,
    #[setting(default = default_int)]
    optional_with_default_fn: Option<usize>,
}

#[test]
fn handles_required_optional_defaults() {
    let result = ConfigLoader::<ReqOptDefaults>::new().load().unwrap();

    assert_eq!(result.config.required, 0);
    assert_eq!(result.config.required_with_default, 123);
    assert_eq!(result.config.required_with_default_fn, 456);
    assert_eq!(result.config.optional, None);
    assert_eq!(result.config.optional_with_default, Some(123));
    assert_eq!(result.config.optional_with_default_fn, Some(456));
}

#[test]
fn can_overwrite_optional_fields() {
    let result = ConfigLoader::<ReqOptDefaults>::new()
        .code("required: 789\noptional: 456", "code.yaml")
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

fn default_count(ctx: &Context) -> DefaultValueResult<usize> {
    Ok(Some(ctx.count * 2))
}

fn default_path(ctx: &Context) -> DefaultValueResult<PathBuf> {
    Ok(Some(ctx.root.join("sub")))
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
    let result = ConfigLoader::<ContextDefaults>::new()
        .load_with_context(&context)
        .unwrap();

    assert_eq!(result.config.count, 10);
    assert_eq!(result.config.path, PathBuf::from("/root/sub"));
}

#[derive(Debug, Config)]
pub struct NestedDefaults {
    #[setting(nested)]
    nested: NativeDefaults,
    #[setting(nested)]
    nested_boxed: Box<NativeDefaults>,
    #[setting(nested)]
    nested_opt: Option<NativeDefaults>,
    #[setting(nested)]
    nested_opt_boxed: Option<Box<NativeDefaults>>,
    #[setting(nested)]
    nested_vec: Vec<NativeDefaults>,
    #[setting(nested)]
    #[allow(clippy::vec_box)]
    nested_vec_boxed: Vec<Box<NativeDefaults>>,
    #[setting(nested)]
    nested_vec_opt_boxed: Vec<Option<Box<NativeDefaults>>>,
    #[setting(nested)]
    nested_map: HashMap<String, NativeDefaults>,
    #[setting(nested)]
    nested_map_boxed: HashMap<String, Box<NativeDefaults>>,
    #[setting(nested)]
    nested_map_opt_boxed: HashMap<String, Option<Box<NativeDefaults>>>,
}

#[test]
fn handles_nested_defaults() {
    let result = ConfigLoader::<NestedDefaults>::new().load().unwrap();

    assert!(result.config.nested_opt.is_none());
    assert!(!result.config.nested.boolean);
}

#[cfg(feature = "renderer_json_schema")]
#[test]
fn generates_json_schema() {
    use starbase_sandbox::{assert_snapshot, create_empty_sandbox};

    let sandbox = create_empty_sandbox();
    let file = sandbox.path().join("schema.json");

    let mut generator = schema::SchemaGenerator::default();
    generator.add::<CustomDefaults>();
    generator.add::<ReqOptDefaults>();
    generator.add::<ContextDefaults>();
    generator.add::<NestedDefaults>();
    generator
        .generate(&file, schema::json_schema::JsonSchemaRenderer::default())
        .unwrap();

    assert!(file.exists());
    assert_snapshot!(std::fs::read_to_string(file).unwrap());
}

#[cfg(feature = "renderer_typescript")]
#[test]
fn generates_typescript() {
    use starbase_sandbox::{assert_snapshot, create_empty_sandbox};

    let sandbox = create_empty_sandbox();
    let file = sandbox.path().join("config.ts");

    let mut generator = schema::SchemaGenerator::default();
    generator.add::<CustomDefaults>();
    generator.add::<ReqOptDefaults>();
    generator.add::<ContextDefaults>();
    generator.add::<NestedDefaults>();
    generator
        .generate(&file, schema::typescript::TypeScriptRenderer::default())
        .unwrap();

    assert!(file.exists());
    assert_snapshot!(std::fs::read_to_string(file).unwrap());
}

// A default that fails names the setting it was for
mod errors {
    use super::*;

    #[derive(Debug, thiserror::Error)]
    #[error("bad default")]
    struct BadDefault;

    fn fail(_: &()) -> Result<Option<usize>, BadDefault> {
        Err(BadDefault)
    }

    #[derive(Debug, Config)]
    pub struct FromHandler {
        #[setting(default = fail)]
        count: usize,
    }

    #[derive(Debug, Config)]
    pub struct FromString {
        #[setting(default = "abc")]
        port: u16,
    }

    #[derive(Debug, Config)]
    pub enum FromVariant {
        #[setting(default = "abc")]
        Port(u16),
    }

    #[derive(Debug, Config)]
    pub struct HoldsVariant {
        #[setting(nested)]
        port: FromVariant,
    }

    fn load<T: Config>() -> String {
        ConfigLoader::<T>::new()
            .load()
            .err()
            .unwrap()
            .to_full_string()
    }

    #[test]
    fn names_the_setting_of_a_handler() {
        assert_eq!(
            load::<FromHandler>(),
            "Invalid default value for count. bad default"
        );
    }

    #[test]
    fn names_the_setting_of_a_string() {
        assert_eq!(
            load::<FromString>(),
            "Invalid default value for port. invalid digit found in string"
        );
    }

    #[test]
    fn names_a_variant() {
        assert_eq!(
            load::<HoldsVariant>(),
            "Invalid default value for Port. invalid digit found in string"
        );
    }
}
