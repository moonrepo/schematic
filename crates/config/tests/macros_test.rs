#![allow(dead_code)]

use schematic::*;
use std::collections::HashMap;

fn default_bool() -> bool {
    true
}

mod private {
    pub fn default_string() -> String {
        String::from("bar")
    }
}

config_enum!(
    pub enum SomeEnum {
        #[default]
        A,
        B,
        C,
    }
);

#[derive(Config)]
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
struct DefaultValues {
    #[setting(default = true)]
    boolean: bool,
    #[setting(default_fn = default_bool)]
    boolean_fn: bool,
    // #[setting(default = 'a')]
    // chars: char,
    #[setting(default = "foo")]
    string: String,
    #[setting(default_fn = private::default_string)]
    string_fn: String,
    #[setting(default = 123)]
    number: usize,
    #[setting(default = [1, 2, 3, 4])]
    array: [u8; 4],
    #[setting(default = (1, 2, 3, 4))]
    tuple: (u8, u8, u8, u8),
    #[setting(default_fn = SomeEnum::default)]
    enums: SomeEnum,
    // Invalid
    // #[setting(default = true, default_fn = default_bool)]
    // invalid: bool,
}

#[derive(Config)]
struct Nested {
    #[setting(nested)]
    one: ValueTypes,
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

fn merge_basic(_: String, _: String) -> Option<String> {
    None
}

#[derive(Config)]
struct Merging {
    #[setting(merge = merge_basic)]
    basic: String,
}

#[derive(Config)]
struct ExtendsString {
    #[setting(extends)]
    extends: String,
    // #[setting(extends)]
    // extends2: String,
    // #[setting(extends)]
    // extends_int: usize,
}

#[derive(Config)]
struct ExtendsList {
    #[setting(extends)]
    extends: Vec<String>,
}

#[derive(Config)]
struct ExtendsEnum {
    #[setting(extends)]
    extends: ExtendsFrom,
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

#[derive(Config)]
struct Validations {
    #[setting(validate(custom(vec_from_env)))]
    basic: String,
}

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
