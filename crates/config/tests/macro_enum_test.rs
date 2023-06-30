#![allow(dead_code, deprecated)]

use schematic::*;
use serde::Serialize;

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

#[derive(Config)]
enum NestedConfigs {
    String(String),
    #[setting(default, nested)]
    Object(SomeConfig),
    #[setting(nested)]
    Objects(SomeConfig, SomeConfig),
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

fn create_gen() -> schema::SchemaGenerator {
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

#[cfg(feature = "json_schema")]
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

#[cfg(feature = "typescript")]
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
