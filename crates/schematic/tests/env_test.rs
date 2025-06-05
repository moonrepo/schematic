#![allow(dead_code)]

use schematic::*;
use serial_test::serial;
use std::{env, path::PathBuf};

#[derive(Debug, Config)]
pub struct EnvVars {
    #[setting(env = "ENV_STRING")]
    string: String,
    #[setting(env = "ENV_STRING", parse_env = schematic::env::ignore_empty, default = "abc")]
    string_empty: String,
    #[setting(env = "ENV_NUMBER")]
    number: usize,
    #[setting(env = "ENV_BOOL")]
    boolean: bool,
    #[setting(env = "ENV_PATH")]
    path: PathBuf,
    #[setting(env = "ENV_FLOAT")]
    float: f32,
}

#[derive(Debug, Config)]
pub struct EnvVarParse {
    #[setting(env = "ENV_VEC_STRING", parse_env = schematic::env::split_comma)]
    list1: Vec<String>,
    #[setting(env = "ENV_VEC_NUMBER", parse_env = schematic::env::split_semicolon)]
    list2: Vec<usize>,
}

fn reset_vars() {
    unsafe {
        env::remove_var("ENV_STRING");
        env::remove_var("ENV_NUMBER");
        env::remove_var("ENV_BOOL");
        env::remove_var("ENV_PATH");
        env::remove_var("ENV_VEC_STRING");
        env::remove_var("ENV_VEC_NUMBER");
        env::remove_var("ENV_LIST1");
        env::remove_var("ENV_LIST2");
        env::remove_var("ENV_FLOAT")
    };
}

#[test]
#[serial]
fn defaults_to_env_var() {
    reset_vars();

    unsafe {
        env::set_var("ENV_STRING", "foo");
        env::set_var("ENV_NUMBER", "123");
        env::set_var("ENV_BOOL", "true");
        env::set_var("ENV_PATH", "some/path");
        env::set_var("ENV_FLOAT", "1.23")
    };

    let result = ConfigLoader::<EnvVars>::new().load().unwrap();

    assert!(result.config.boolean);
    assert_eq!(result.config.string, "foo");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.float, 1.23);
    assert_eq!(result.config.path, PathBuf::from("some/path"));
}

#[test]
#[serial]
#[should_panic(expected = "Invalid environment variable ENV_NUMBER.")]
fn errors_on_parse_fail() {
    reset_vars();

    unsafe { env::set_var("ENV_NUMBER", "abc") };

    ConfigLoader::<EnvVars>::new().load().unwrap();
}

#[test]
#[serial]
fn parses_into_env_vars() {
    reset_vars();

    unsafe {
        env::set_var("ENV_VEC_STRING", "1,2,3");
        env::set_var("ENV_VEC_NUMBER", "1;2;3")
    };

    let result = ConfigLoader::<EnvVarParse>::new().load().unwrap();

    assert_eq!(result.config.list1, vec!["1", "2", "3"]);
    assert_eq!(result.config.list2, vec![1, 2, 3]);
}

#[test]
#[serial]
#[should_panic(expected = "Invalid environment variable ENV_VEC_NUMBER.")]
fn errors_on_split_parse_fail() {
    reset_vars();

    unsafe { env::set_var("ENV_VEC_NUMBER", "1;a;3") };

    ConfigLoader::<EnvVarParse>::new().load().unwrap();
}

#[test]
#[serial]
fn env_var_takes_precedence() {
    reset_vars();

    unsafe { env::set_var("ENV_STRING", "foo") };

    let result = ConfigLoader::<EnvVars>::new()
        .code("string: bar", Format::Yaml)
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.string, "foo");
}

#[test]
#[serial]
fn can_ignore_empty_values() {
    reset_vars();

    unsafe { env::set_var("ENV_STRING", "") };

    let result = ConfigLoader::<EnvVars>::new()
        .code("string: bar", Format::Yaml)
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.string, "");
    assert_eq!(result.config.string_empty, "abc");
}

#[derive(Debug, Config)]
pub struct EnvVarsNested {
    #[setting(env = "ENV_STRING")]
    string: String,
}

#[derive(Debug, Config)]
pub struct EnvVarsBase {
    #[setting(nested)]
    nested: EnvVarsNested,
    #[setting(nested)]
    opt_nested: Option<EnvVarsNested>,
}

#[test]
#[serial]
fn loads_env_vars_for_nested() {
    reset_vars();

    unsafe { env::set_var("ENV_STRING", "foo") };

    let result = ConfigLoader::<EnvVarsBase>::new()
        .code("{}", Format::Yaml)
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.nested.string, "foo");
    assert_eq!(result.config.opt_nested.unwrap().string, "foo");
}

#[test]
#[serial]
fn loads_env_vars_for_optional_nested_when_valued() {
    reset_vars();

    unsafe { env::set_var("ENV_STRING", "foo") };

    let result = ConfigLoader::<EnvVarsBase>::new()
        .code("optNested:\n  string: bar", Format::Yaml)
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.nested.string, "foo");
    assert_eq!(result.config.opt_nested.unwrap().string, "foo");
}

#[derive(Debug, Config)]
#[config(env_prefix = "ENV_")]
pub struct EnvVarsPrefixed {
    string: String,
    number: usize,
    #[setting(rename = "bool")]
    boolean: bool,
    path: PathBuf,
    #[setting(parse_env = schematic::env::split_comma)]
    list1: Vec<String>,
    #[setting(parse_env = schematic::env::split_semicolon)]
    list2: Vec<usize>,
    #[setting(nested)]
    nested: EnvVarsNested,
}

#[derive(Debug, Config)]
pub struct EnvVarsPrefixedContainer {
    #[setting(nested)]
    nested: EnvVarsPrefixed,
    #[setting(nested)]
    opt_nested: Option<EnvVarsPrefixed>,
}

#[test]
#[serial]
fn loads_from_prefixed() {
    reset_vars();

    unsafe {
        env::set_var("ENV_STRING", "foo");
        env::set_var("ENV_NUMBER", "123");
        env::set_var("ENV_BOOL", "true");
        env::set_var("ENV_PATH", "some/path");
        env::set_var("ENV_LIST1", "1,2,3");
        env::set_var("ENV_LIST2", "1;2;3")
    };

    let result = ConfigLoader::<EnvVarsPrefixed>::new().load().unwrap();

    assert!(result.config.boolean);
    assert_eq!(result.config.string, "foo");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.path, PathBuf::from("some/path"));
    assert_eq!(result.config.list1, vec!["1", "2", "3"]);
    assert_eq!(result.config.list2, vec![1, 2, 3]);
}

#[cfg(feature = "renderer_json_schema")]
#[test]
fn generates_json_schema() {
    use starbase_sandbox::{assert_snapshot, create_empty_sandbox};

    let sandbox = create_empty_sandbox();
    let file = sandbox.path().join("schema.json");

    let mut generator = schema::SchemaGenerator::default();
    generator.add::<EnvVarsPrefixed>();
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
    generator.add::<EnvVarsPrefixed>();
    generator
        .generate(&file, schema::typescript::TypeScriptRenderer::default())
        .unwrap();

    assert!(file.exists());
    assert_snapshot!(std::fs::read_to_string(file).unwrap());
}
