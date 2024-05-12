#![allow(dead_code, deprecated)]

use schematic::*;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Config)]
pub struct SomeConfig {
    foo: String,
    bar: usize,
}

#[derive(Config)]
enum AllUnit {
    Foo,
    #[setting(default)]
    Bar,
    Baz,
}

#[derive(Config)]
enum AllUnnamed {
    Foo(String),
    Bar(bool),
    Baz(usize),
}

#[derive(Config)]
enum OfBothTypes {
    #[setting(null)]
    Foo,
    #[setting(default)]
    Bar(bool, usize),
}

fn merge_tuple<C>(
    prev: (String, usize),
    next: (String, usize),
    _: &C,
) -> MergeResult<(String, usize)> {
    Ok(Some((format!("{}-{}", prev.0, next.0), (prev.1 + next.1))))
}

fn validate_tuple<T, C>(_: (&String, &usize), _: &T, _: &C, _: bool) -> ValidateResult {
    Ok(())
}

fn validate_nested<T, C>(_: &PartialSomeConfig, _: &T, _: &C, _: bool) -> ValidateResult {
    Ok(())
}

#[derive(Config)]
enum Collections {
    #[setting(merge = merge::append_vec, validate = validate::min_length(1))]
    List(Vec<String>),
    #[setting(merge = merge::merge_btreemap, validate = validate::min_length(1))]
    Map(BTreeMap<String, String>),
    #[setting(merge = merge_tuple, validate = validate_tuple)]
    Tuple(String, usize),
}

#[derive(Config)]
enum NestedConfigs {
    String(String),
    #[setting(default, nested, validate = validate_nested)]
    Object(SomeConfig),
    #[setting(nested)]
    Objects(SomeConfig, SomeConfig),
}

#[allow(unused_parens)]
fn validate_string<T, C>(_: (&Option<String>), _: &T, _: &C, _: bool) -> Result<(), ValidateError> {
    Ok(())
}

#[derive(Config)]
enum ValidateConfigs {
    Normal(String),
    #[setting(validate = validate_string)]
    Optional(Option<String>),
    #[setting(validate = validate_string, required)]
    Required(Option<String>),
    #[setting(required)]
    RequiredMany(Option<String>, Option<String>),
}

#[derive(Config, Serialize)]
#[serde(untagged, expecting = "something")]
enum WithSerde {
    #[serde(rename = "fooooo")]
    Foo(String),
    #[serde(alias = "barrrrr")]
    Bar(bool),
    #[setting(rename = "bazzzzz")]
    Baz(usize),
}

/// Container
#[derive(Config)]
enum WithComments {
    // Variant
    Foo,
    /// Variant
    Bar,
    /** Variant */
    Baz,
}

#[derive(Config)]
#[config(serde(untagged, expecting = "something"))]
enum Untagged {
    Foo,
    Bar(bool),
    #[setting(rename = "bazzer")]
    Baz(usize, String),
    #[setting(nested)]
    Qux(SomeConfig),
}

#[derive(Config)]
enum ExternalTagged {
    Foo,
    Bar(bool),
    #[setting(rename = "bazzer")]
    Baz(usize),
    #[setting(nested)]
    Qux(SomeConfig),
}

#[derive(Config)]
#[config(serde(tag = "type"))]
enum InternalTagged {
    Foo,
    Bar(bool),
    #[setting(rename = "bazzer")]
    Baz(usize),
    #[setting(nested)]
    Qux(SomeConfig),
}

#[derive(Config)]
#[config(serde(tag = "type", content = "content"))]
enum AdjacentTagged {
    Foo,
    Bar(bool),
    #[setting(rename = "bazzer")]
    Baz(usize),
    #[setting(nested)]
    Qux(SomeConfig),
}

fn create_gen() -> schema::SchemaGenerator<'static> {
    let mut generator = schema::SchemaGenerator::default();
    generator.add::<AllUnit>();
    generator.add::<AllUnnamed>();
    generator.add::<OfBothTypes>();
    generator.add::<NestedConfigs>();
    generator.add::<WithSerde>();
    generator.add::<WithComments>();
    generator.add::<Untagged>();
    generator.add::<ExternalTagged>();
    generator.add::<InternalTagged>();
    generator.add::<AdjacentTagged>();
    generator.add::<PartialAllUnit>();
    generator.add::<PartialAllUnnamed>();
    generator.add::<PartialOfBothTypes>();
    generator.add::<PartialNestedConfigs>();
    generator.add::<PartialWithSerde>();
    generator.add::<PartialWithComments>();
    generator.add::<PartialUntagged>();
    generator.add::<PartialExternalTagged>();
    generator.add::<PartialInternalTagged>();
    generator.add::<PartialAdjacentTagged>();
    generator
}

#[cfg(feature = "renderer_json_schema")]
#[test]
fn generates_json_schema() {
    use starbase_sandbox::{assert_snapshot, create_empty_sandbox};

    let sandbox = create_empty_sandbox();
    let file = sandbox.path().join("schema.json");

    let generator = create_gen();
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

    let generator = create_gen();
    generator
        .generate(&file, schema::typescript::TypeScriptRenderer::default())
        .unwrap();

    assert!(file.exists());
    assert_snapshot!(std::fs::read_to_string(file).unwrap());
}
