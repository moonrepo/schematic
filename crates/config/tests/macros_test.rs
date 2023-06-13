#![allow(dead_code, deprecated)]

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
    #[derive(Default, ConfigEnum)]
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
#[config(allow_unknown_fields, rename_all = "kebab-case")]
struct Serde {
    #[setting(rename = "renamed")]
    rename: String,
    #[setting(skip)]
    skipped: String,
    #[setting(skip, rename = "renamed")]
    all: bool,
}

#[derive(Config, Serialize)]
#[serde(rename = "SerdeNativeRenamed", rename_all = "snake_case")]
struct SerdeNative {
    #[serde(alias = "test", rename = "renamed")]
    rename: String,
    #[serde(skip)]
    skipped: String,
    #[serde(skip, rename = "renamed")]
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

fn vec_from_env(_: String) -> Result<Option<Vec<String>>, ConfigError> {
    Ok(Some(vec![]))
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
#[deprecated(since = "1.2.3", note = "Invalid")]
struct Comments {
    // Normal
    normal: bool,
    /// Docs
    #[deprecated]
    docs: bool,
    /* Inline block */
    inline_block: bool,
    /**
     * Block
     */
    #[deprecated = "Bye"]
    block: bool,
}

#[derive(ConfigEnum, Debug)]
enum BasicEnum {
    Foo,
    /// Comment
    Bar,
    Baz,
}

#[derive(ConfigEnum, Debug, Deserialize, Serialize)]
#[serde(rename = "Test", rename_all = "UPPERCASE")]
enum CustomFormatEnum {
    Foo,
    #[serde(rename = "bAr")]
    Bar,
    #[variant(value = "b-a-z")]
    Baz,
}

#[derive(ConfigEnum, Debug)]
enum OtherEnum {
    Foo,
    #[deprecated]
    Bar,
    Baz,
    #[variant(fallback)]
    Other(String),
}

#[derive(ConfigEnum, Debug, Serialize)]
enum AliasedEnum {
    #[serde(alias = "a")]
    Foo,
    #[serde(alias = "b")]
    Bar,
    #[serde(alias = "c")]
    Baz,
}

#[cfg(feature = "typescript")]
#[test]
fn generates_typescript() {
    use renderers::typescript::*;
    use starbase_sandbox::{assert_snapshot, create_empty_sandbox};

    let sandbox = create_empty_sandbox();
    let file = sandbox.path().join("config.ts");

    let mut generator = schema::SchemaGenerator::default();
    generator.add::<SomeEnum>();
    generator.add::<BasicEnum>();
    generator.add::<CustomFormatEnum>();
    generator.add::<OtherEnum>();
    generator.add::<AliasedEnum>();
    generator.add::<ValueTypes>();
    generator.add::<OptionalValues>();
    generator.add::<DefaultValues>();
    generator.add::<Serde>();
    generator.add::<SerdeNative>();
    generator.add::<Merging>();
    generator.add::<ExtendsString>();
    generator.add::<ExtendsList>();
    generator.add::<ExtendsEnum>();
    generator.add::<ExtendsOptional>();
    generator.add::<EnvVars>();
    generator.add::<NestedValidations>();
    generator.add::<Validations>();
    generator.add::<Comments>();
    // Partials are separate
    generator.add::<PartialDefaultValues>();
    generator.add::<PartialNested>();
    generator.add::<PartialValidations>();
    generator
        .generate(&file, TypeScriptRenderer::default())
        .unwrap();

    assert!(file.exists());
    assert_snapshot!(std::fs::read_to_string(&file).unwrap());

    generator
        .generate(
            &file,
            TypeScriptRenderer::new(TypeScriptOptions {
                const_enum: true,
                enum_format: EnumFormat::Enum,
                object_format: ObjectFormat::Type,
            }),
        )
        .unwrap();

    assert_snapshot!(std::fs::read_to_string(file).unwrap());
}
