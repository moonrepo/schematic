use schematic::*;
use serde::Serialize;

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
    Foo,
    #[setting(default)]
    Bar(bool),
}

#[derive(Config, Serialize)]
#[serde(untagged, expecting = "asd")]
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
