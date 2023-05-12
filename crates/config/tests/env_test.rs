use schematic::*;
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
}

#[test]
fn defaults_to_env_var() {
    reset_vars();
    env::set_var("ENV_STRING", "foo");
    env::set_var("ENV_NUMBER", "123");
    env::set_var("ENV_BOOL", "true");
    env::set_var("ENV_PATH", "some/path");

    let result = ConfigLoader::<EnvVars>::new(SourceFormat::Yaml)
        .load()
        .unwrap();

    assert!(result.config.boolean);
    assert_eq!(result.config.string, "foo");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.path, PathBuf::from("some/path"));
}

#[test]
#[should_panic(expected = "InvalidEnvVar(\"ENV_NUMBER\"")]
fn errors_on_parse_fail() {
    reset_vars();
    env::set_var("ENV_NUMBER", "abc");

    ConfigLoader::<EnvVars>::new(SourceFormat::Yaml)
        .load()
        .unwrap();
}

#[test]
fn parses_into_env_vars() {
    reset_vars();
    env::set_var("ENV_VEC_STRING", "1,2,3");
    env::set_var("ENV_VEC_NUMBER", "1;2;3");

    let result = ConfigLoader::<EnvVarParse>::new(SourceFormat::Yaml)
        .load()
        .unwrap();

    assert_eq!(result.config.list1, vec!["1", "2", "3"]);
    assert_eq!(result.config.list2, vec![1, 2, 3]);
}

#[test]
#[should_panic(
    expected = "InvalidEnvVar(\"ENV_VEC_NUMBER\", \"Failed to parse \\\"a\\\" into the correct type.\")"
)]
fn errors_on_split_parse_fail() {
    reset_vars();
    env::set_var("ENV_VEC_NUMBER", "1;a;3");

    ConfigLoader::<EnvVarParse>::new(SourceFormat::Yaml)
        .load()
        .unwrap();
}
