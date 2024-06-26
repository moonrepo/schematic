#![allow(dead_code, deprecated)]

use schematic::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

fn default_bool<C>(_: &C) -> DefaultValueResult<bool> {
    Ok(Some(true))
}

mod private {
    pub fn default_string<C>(_: &C) -> schematic::DefaultValueResult<String> {
        Ok(Some("bar".into()))
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
#[config(rename_all = "snake_case")]
pub struct ValueTypes {
    boolean: bool,
    string: String,
    number: usize,
    vector: Vec<String>,
    map: HashMap<String, u64>,
    enums: SomeEnum,
    s3_value: String,
    #[setting(nested, exclude)]
    other: OptionalValues,
    #[setting(flatten)]
    rest: HashMap<String, serde_json::Value>,
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
    path_string_box: Box<PathBuf>,
    #[setting(default = 123)]
    number: usize,
    #[setting(default = 1.32)]
    float32: f32,
    #[setting(default = 1.64)]
    float64: f64,
    #[setting(default = [1, 2, 3, 4])]
    array: [u8; 4],
    array_opt: Option<[u8; 4]>,
    #[setting(default = (1, 2, 3, 4))]
    tuple: (u8, u8, u8, u8),
    tuple_opt: Option<(u8, u8, u8, u8)>,
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
    #[setting(skip)]
    all: bool,
}

#[derive(Config, Serialize)]
#[serde(rename = "SerdeNativeRenamed", rename_all = "snake_case")]
struct SerdeNative {
    #[serde(alias = "test", rename = "renamed")]
    rename: String,
    #[serde(skip)]
    skipped: String,
    #[serde(skip, rename = "everything")]
    all: bool,
}

fn merge_basic<C>(_: String, _: String, _: &C) -> MergeResult<String> {
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

fn vec_from_env(_: String) -> ParseEnvResult<Vec<String>> {
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

fn validate_test<T, C>(_: &str, _: &T, _: &C, _: bool) -> ValidateResult {
    Ok(())
}

fn validate_nested<T, C>(_: &PartialNestedValidations, _: &T, _: &C, _: bool) -> ValidateResult {
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
    #[setting(required, validate = validate_test)]
    required: Option<String>,
    #[setting(nested, validate = validate_nested)]
    nested: NestedValidations,
    #[setting(nested, validate = validate_nested)]
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
    /// Docs with a super long comment that will span multiple lines.
    /// It also **contains** some _markdown_ [stuff](.).
    docs_long: bool,
    /* Inline block */
    inline_block: bool,
    /**
     * Block
     */
    #[deprecated = "Bye"]
    block: bool,
    /**
     * Block with a super long comment that will span multiple lines.
     * Block with a super long comment that will span multiple lines.
     */
    block_long: bool,
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
#[config(before_parse = "UPPERCASE")]
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
#[config(rename = "Aliased", before_parse = "lowercase")]
enum AliasedEnum {
    #[serde(alias = "a")]
    Foo,
    #[serde(alias = "b")]
    Bar,
    #[serde(alias = "c")]
    Baz,
}

#[derive(Config)]
struct UnnamedSingle(#[setting(default = "abc", validate = validate_test)] String);

#[derive(Config)]
struct UnnamedMultiple(String, Option<usize>, bool);

#[derive(Config)]
struct UnnamedNested(#[setting(nested)] OptionalValues);

#[derive(Config)]
struct UnnamedCollection(#[setting(nested)] HashMap<String, Option<OptionalValues>>);

#[cfg(feature = "renderer_json_schema")]
#[test]
fn generates_json_schema() {
    use starbase_sandbox::{assert_snapshot, create_empty_sandbox};

    let sandbox = create_empty_sandbox();
    let file = sandbox.path().join("schema.json");

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
    generator.add::<UnnamedSingle>();
    generator.add::<UnnamedMultiple>();
    generator.add::<UnnamedNested>();
    generator.add::<UnnamedCollection>();
    // Partials are separate
    generator.add::<PartialDefaultValues>();
    generator.add::<PartialNested>();
    generator.add::<PartialValidations>();
    generator
        .generate(&file, schema::json_schema::JsonSchemaRenderer::default())
        .unwrap();

    assert!(file.exists());
    assert_snapshot!(std::fs::read_to_string(file).unwrap());
}

#[cfg(feature = "renderer_typescript")]
#[test]
fn generates_typescript() {
    use schema::typescript::*;
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
    generator.add::<UnnamedSingle>();
    generator.add::<UnnamedMultiple>();
    generator.add::<UnnamedNested>();
    generator.add::<UnnamedCollection>();
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
                ..Default::default()
            }),
        )
        .unwrap();

    assert_snapshot!(std::fs::read_to_string(file).unwrap());
}
