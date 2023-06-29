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
    #[setting(null)]
    Foo,
    #[setting(default)]
    Bar(bool, usize),
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

// #[derive(Config, Serialize)]
// #[serde(tag = "type")]
// enum InternalTagged {
//     Foo,
//     Bar(bool),
//     #[setting(rename = "bazzer")]
//     Baz(usize),
// }

#[derive(Config, Serialize)]
#[serde(tag = "type", content = "content")]
enum AdjacentTagged {
    Foo,
    Bar(bool),
    #[setting(rename = "bazzer")]
    Baz(usize),
}
