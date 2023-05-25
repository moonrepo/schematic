#![allow(dead_code)]

use schematic::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

fn default_bool<C>(_: &C) -> Option<bool> {
    Some(true)
}

mod private {
    pub fn default_string<C>(_: &C) -> Option<String> {
        Some("bar".into())
    }
}

derive_enum!(
    #[derive(Default)]
    pub enum SomeEnum {
        #[default]
        A,
        B,
        C,
    }
);

#[derive(Config)]
#[config(file = "test.json")]
pub struct ValueTypes {
    boolean: bool,
    string: String,
    number: usize,
    vector: Vec<String>,
    map: HashMap<String, u64>,
    enums: SomeEnum,
}

#[derive(Config)]
pub struct OptionalValues {
    required: bool,
    optional: Option<String>,
}

#[derive(Config)]
#[config(file = "some/path/file.yml")]
struct DefaultValues {
    #[setting(default = true)]
    boolean: bool,
    #[setting(default = default_bool)]
    boolean_fn: bool,
    // #[setting(default = 'a')]
    // chars: char,
    #[setting(default = "foo")]
    string: String,
    #[setting(default = private::default_string)]
    string_fn: String,
    #[setting(default = "foo.json")]
    file_string: String,
    #[setting(default = "foo with. many values!")]
    long_string: String,
    #[setting(default = "foo/bar")]
    path_string: PathBuf,
    #[setting(default = 123)]
    number: usize,
    #[setting(default = [1, 2, 3, 4])]
    array: [u8; 4],
    #[setting(default = (1, 2, 3, 4))]
    tuple: (u8, u8, u8, u8),
    #[setting(default = vec![1, 2, 3, 4])]
    vector: Vec<usize>,
    // #[setting(default = SomeEnum::default)]
    enums: SomeEnum,
    // Invalid
    // #[setting(default = true, default = default_bool)]
    // invalid: bool,
}

#[derive(Config)]
struct Nested {
    #[setting(nested)]
    one: ValueTypes,
    #[setting(nested)]
    two: Option<ValueTypes>,
    #[setting(nested)]
    list: Vec<ValueTypes>,
    #[setting(nested)]
    map: HashMap<String, ValueTypes>,
    #[setting(nested)]
    map2: Option<BTreeMap<String, ValueTypes>>,
    // Invalid
    // #[setting(nested)]
    // two: bool,
    // #[setting(nested, default = true)]
    // no_defualt: ValueTypes,
}

#[derive(Config)]
struct Serde {
    #[setting(rename = "renamed")]
    rename: String,
    #[setting(skip)]
    skipped: String,
    #[setting(skip, rename = "renamed")]
    all: bool,
}

fn merge_basic<C>(_: String, _: String, _: &C) -> Result<Option<String>, ConfigError> {
    Ok(None)
}

#[derive(Config)]
struct Merging {
    #[setting(merge = merge_basic)]
    basic: String,
}

#[derive(Config)]
struct ExtendsString {
    #[setting(extend)]
    extends: String,
    // #[setting(extend)]
    // extends2: String,
    // #[setting(extend)]
    // extends_int: usize,
}

#[derive(Config)]
struct ExtendsList {
    #[setting(extend)]
    extends: Vec<String>,
}

#[derive(Config)]
struct ExtendsEnum {
    #[setting(extend)]
    extends: ExtendsFrom,
}

#[derive(Config)]
struct ExtendsOptional {
    #[setting(extend)]
    extends: Option<Vec<String>>,
}

fn vec_from_env(_: String) -> Result<Vec<String>, ConfigError> {
    Ok(vec![])
}

#[derive(Config)]
struct EnvVars {
    #[setting(env = "FOO")]
    basic: String,
    #[setting(env = "BAR", parse_env = vec_from_env)]
    advanced: Vec<String>,
    // #[setting(parse_env = vec_from_env)]
    // invalid: Vec<String>,
}

fn validate_test<T, C>(_: &str, _: &T, _: &C) -> Result<(), ValidateError> {
    Ok(())
}

#[derive(Config)]
pub struct NestedValidations {
    #[setting(validate = validate_test)]
    basic: String,
}

#[derive(Config)]
struct Validations {
    #[setting(validate = validate_test)]
    basic: String,
    #[setting(validate = validate_test)]
    optional: Option<String>,
    #[setting(nested)]
    nested: NestedValidations,
    #[setting(nested)]
    nested2: Option<NestedValidations>,
}

/// Container comment.
#[derive(Config)]
struct Comments {
    // Normal
    normal: bool,
    /// Docs
    docs: bool,
    /* Inline block */
    inline_block: bool,
    /**
     * Block
     */
    block: bool,
}

#[derive(ConfigEnum)]
enum BasicEnum {
    Foo,
    Bar,
    Baz,
}

#[derive(ConfigEnum, Deserialize, Serialize)]
#[serde(rename = "Test", rename_all = "UPPERCASE")]
enum CustomFormatEnum {
    Foo,
    Bar,
    #[variant(value = "b-a-z")]
    Baz,
}
