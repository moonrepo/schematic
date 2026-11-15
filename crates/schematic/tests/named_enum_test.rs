#![allow(dead_code)]

use schematic::schema::SchemaGenerator;
use schematic::*;
use starbase_sandbox::{assert_snapshot, create_empty_sandbox};
use std::fs;

#[derive(Clone, Config)]
pub struct Retry {
    #[setting(default = 3)]
    attempts: u8,
}

/// Where to load from.
#[derive(Clone, Config)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Source {
    /// A file on disk.
    #[setting(default)]
    File {
        /// The file to read.
        #[setting(default = "config.json")]
        path: String,
        format: Option<String>,
    },
    Url {
        #[setting(required)]
        address: Option<String>,
        #[setting(nested)]
        retry: Retry,
    },
}

#[derive(Clone, Config)]
pub enum Mode {
    #[setting(default)]
    Off,
    On {
        level: u8,
    },
}

#[derive(Clone, Config)]
#[serde(untagged)]
pub enum Dependency {
    #[setting(default)]
    Version(String),
    Detailed {
        version: String,
        optional: Option<bool>,
    },
}

#[derive(Clone, Config)]
pub struct Settings {
    #[setting(nested)]
    source: Source,
    #[setting(nested)]
    mode: Mode,
    #[setting(nested)]
    dependency: Dependency,
}

fn generate<O, R: schematic::schema::SchemaRenderer<O>>(partial: bool, renderer: R) -> String {
    let sandbox = create_empty_sandbox();
    let file = sandbox.path().join("output");
    let mut generator = SchemaGenerator::default();

    if partial {
        generator.add::<PartialSettings>();
    } else {
        generator.add::<Settings>();
    }

    generator.generate(&file, renderer).unwrap();

    fs::read_to_string(file).unwrap()
}

#[cfg(feature = "renderer_json_schema")]
mod json_schema {
    use super::*;
    use schematic::schema::json_schema::*;

    #[test]
    fn renders_full() {
        assert_snapshot!(generate(false, JsonSchemaRenderer::default()));
    }

    // Every setting is optional, but an internally tagged variant still
    // requires its tag
    #[test]
    fn renders_partial() {
        assert_snapshot!(generate(true, JsonSchemaRenderer::default()));
    }
}

#[cfg(feature = "renderer_typescript")]
mod typescript {
    use super::*;
    use schematic::schema::typescript::*;

    #[test]
    fn renders_full() {
        assert_snapshot!(generate(false, TypeScriptRenderer::default()));
    }

    #[test]
    fn renders_partial() {
        assert_snapshot!(generate(true, TypeScriptRenderer::default()));
    }
}
