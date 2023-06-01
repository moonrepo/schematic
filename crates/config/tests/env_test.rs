#![allow(dead_code)]

use schematic::*;
use serial_test::serial;
use std::{env, path::PathBuf};

#[derive(Debug, Config)]
pub struct EnvVars {
    #[setting(env = "ENV_STRING")]
    string: String,
    #[setting(env = "ENV_NUMBER")]
    number: usize,
    #[setting(env = "ENV_BOOL")]
    boolean: bool,
    #[setting(env = "ENV_PATH")]
    path: PathBuf,
}

#[derive(Debug, Config)]
pub struct EnvVarParse {
    #[setting(env = "ENV_VEC_STRING", parse_env = schematic::env::split_comma)]
    list1: Vec<String>,
    #[setting(env = "ENV_VEC_NUMBER", parse_env = schematic::env::split_semicolon)]
    list2: Vec<usize>,
}

fn reset_vars() {
    env::remove_var("ENV_STRING");
    env::remove_var("ENV_NUMBER");
    env::remove_var("ENV_BOOL");
    env::remove_var("ENV_PATH");
    env::remove_var("ENV_VEC_STRING");
    env::remove_var("ENV_VEC_NUMBER");
    env::remove_var("ENV_LIST_1");
    env::remove_var("ENV_LIST_2");
}

#[test]
#[serial]
fn defaults_to_env_var() {
    reset_vars();
    env::set_var("ENV_STRING", "foo");
    env::set_var("ENV_NUMBER", "123");
    env::set_var("ENV_BOOL", "true");
    env::set_var("ENV_PATH", "some/path");

    let result = ConfigLoader::<EnvVars>::new().load().unwrap();

    assert!(result.config.boolean);
    assert_eq!(result.config.string, "foo");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.path, PathBuf::from("some/path"));
}

#[test]
#[serial]
#[should_panic(expected = "InvalidEnvVar(\"ENV_NUMBER\"")]
fn errors_on_parse_fail() {
    reset_vars();
    env::set_var("ENV_NUMBER", "abc");

    ConfigLoader::<EnvVars>::new().load().unwrap();
}

#[test]
#[serial]
fn parses_into_env_vars() {
    reset_vars();
    env::set_var("ENV_VEC_STRING", "1,2,3");
    env::set_var("ENV_VEC_NUMBER", "1;2;3");

    let result = ConfigLoader::<EnvVarParse>::new().load().unwrap();

    assert_eq!(result.config.list1, vec!["1", "2", "3"]);
    assert_eq!(result.config.list2, vec![1, 2, 3]);
}

#[test]
#[serial]
#[should_panic(
    expected = "InvalidEnvVar(\"ENV_VEC_NUMBER\", \"Failed to parse \\\"a\\\" into the correct type.\")"
)]
fn errors_on_split_parse_fail() {
    reset_vars();
    env::set_var("ENV_VEC_NUMBER", "1;a;3");

    ConfigLoader::<EnvVarParse>::new().load().unwrap();
}

#[test]
#[serial]
fn env_var_takes_precedence() {
    reset_vars();
    env::set_var("ENV_STRING", "foo");

    let result = ConfigLoader::<EnvVars>::new()
        .code("string: bar")
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.string, "foo");
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
    env::set_var("ENV_STRING", "foo");

    let result = ConfigLoader::<EnvVarsBase>::new()
        .code("{}")
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.nested.string, "foo");
    assert!(result.config.opt_nested.is_none());
}

#[test]
#[serial]
fn loads_env_vars_for_optional_nested_when_valued() {
    reset_vars();
    env::set_var("ENV_STRING", "foo");

    let result = ConfigLoader::<EnvVarsBase>::new()
        .code("optNested:\n  string: bar")
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

#[test]
#[serial]
fn loads_from_prefixed() {
    reset_vars();
    env::set_var("ENV_STRING", "foo");
    env::set_var("ENV_NUMBER", "123");
    env::set_var("ENV_BOOL", "true");
    env::set_var("ENV_PATH", "some/path");
    env::set_var("ENV_LIST_1", "1,2,3");
    env::set_var("ENV_LIST_2", "1;2;3");

    let result = ConfigLoader::<EnvVarsPrefixed>::new().load().unwrap();

    assert!(result.config.boolean);
    assert_eq!(result.config.string, "foo");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.path, PathBuf::from("some/path"));
    assert_eq!(result.config.list1, vec!["1", "2", "3"]);
    assert_eq!(result.config.list2, vec![1, 2, 3]);
}
