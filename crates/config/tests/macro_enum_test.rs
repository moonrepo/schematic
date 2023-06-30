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

#[derive(Config, Serialize)]
#[serde(untagged)]
enum Untagged {
    Foo,
    Bar(bool),
    #[setting(rename = "bazzer")]
    Baz(usize, String),
}

#[derive(Config, Serialize)]
enum ExternalTagged {
    Foo,
    Bar(bool),
    #[setting(rename = "bazzer")]
    Baz(usize),
}

#[derive(Config, Serialize)]
#[serde(tag = "type")]
enum InternalTagged {
    Foo,
    Bar(bool),
    #[setting(rename = "bazzer")]
    Baz(usize),
}

#[derive(Config, Serialize)]
#[serde(tag = "type", content = "content")]
enum AdjacentTagged {
    Foo,
    Bar(bool),
    #[setting(rename = "bazzer")]
    Baz(usize),
}
