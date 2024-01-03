#![allow(dead_code, deprecated)]

use schematic::schema::SchemaGenerator;
use schematic::*;
use starbase_sandbox::{assert_snapshot, create_empty_sandbox};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

derive_enum!(
    /** Docblock comment. */
    #[derive(ConfigEnum, Default)]
    pub enum BasicEnum {
        #[default]
        Foo,
        Bar,
        Baz,
    }
);

/// Some comment.
#[derive(Clone, Config)]
pub struct AnotherConfig {
    /// An optional string.
    opt: Option<String>,
    /// An optional enum.
    enums: Option<BasicEnum>,
}

#[derive(Clone, Config)]
struct GenConfig {
    boolean: bool,
    string: String,
    number: usize,
    float32: f32,
    float64: f64,
    vector: Vec<String>,
    map: HashMap<String, u64>,
    enums: BasicEnum,
    #[setting(nested)]
    nested: AnotherConfig,

    // Types
    date: chrono::NaiveDate,
    datetime: chrono::NaiveDateTime,
    decimal: rust_decimal::Decimal,
    time: chrono::NaiveTime,
    path: PathBuf,
    rel_path: relative_path::RelativePathBuf,
    url: Option<url::Url>,
    version: Option<semver::Version>,
    version_req: semver::VersionReq,
    spec: version_spec::VersionSpec,
    spec_unresolved: version_spec::UnresolvedVersionSpec,
    id: warpgate::Id,
    locator: Option<warpgate::PluginLocator>,
    json_value: serde_json::Value,
    toml_value: Option<toml::Value>,
    yaml_value: serde_yaml::Value,
}

/// Some comment.
#[derive(Clone, Config)]
pub struct TwoDepthConfig {
    /// An optional string.
    opt: Option<String>,
}

/// Some comment.
#[derive(Clone, Config)]
pub struct OneDepthConfig {
    /// This is another nested field.
    #[setting(nested)]
    two: TwoDepthConfig,
}

#[derive(Clone, Config)]
struct TemplateConfig {
    /// This is a boolean with a medium length description.
    #[setting(env = "TEMPLATE_BOOLEAN")]
    boolean: bool,
    /// This is a string.
    #[setting(default = "abc")]
    string: String,
    /// This is a number with a long description.
    /// This is a number with a long description.
    number: usize,
    /// This is a float thats deprecated.
    #[deprecated]
    float32: f32,
    /// This is a float.
    #[setting(default = 1.23)]
    float64: f64,
    /// This is a list of strings.
    vector: Vec<String>,
    /// This is a map of numbers.
    map: HashMap<String, u64>,
    /// This is an enum with a medium length description and deprecated.
    #[deprecated = "Dont use enums!"]
    enums: BasicEnum,
    /// This is a nested struct with its own fields.
    #[setting(nested)]
    nested: AnotherConfig,
    /// This is a nested struct with its own fields.
    #[setting(nested)]
    one: OneDepthConfig,
}

fn create_generator() -> SchemaGenerator {
    let mut generator = SchemaGenerator::default();
    generator.add::<GenConfig>();
    generator
}

fn create_template_generator() -> SchemaGenerator {
    let mut generator = SchemaGenerator::default();
    generator.add::<PartialTemplateConfig>();
    generator
}

#[cfg(feature = "json_schema")]
mod json_schema {
    use super::*;
    use schematic::schema::json_schema::*;

    #[test]
    fn defaults() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("schema.json");

        create_generator()
            .generate(&file, JsonSchemaRenderer::default())
            .unwrap();

        assert_snapshot!(fs::read_to_string(file).unwrap());
    }
}

#[cfg(all(feature = "template", feature = "json"))]
mod template_json {
    use super::*;
    use schematic::schema::template::*;

    #[test]
    fn defaults() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("schema.json");

        create_template_generator()
            .generate(&file, TemplateRenderer::with_format(Format::Json))
            .unwrap();

        assert_snapshot!(fs::read_to_string(file).unwrap());
    }
}

#[cfg(all(feature = "template", feature = "toml"))]
mod template_toml {
    use super::*;
    use schematic::schema::template::*;

    #[test]
    fn defaults() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("schema.toml");

        create_template_generator()
            .generate(&file, TemplateRenderer::with_format(Format::Toml))
            .unwrap();

        assert_snapshot!(fs::read_to_string(file).unwrap());
    }
}

#[cfg(all(feature = "template", feature = "yaml"))]
mod template_yaml {
    use super::*;
    use schematic::schema::template::*;

    #[test]
    fn defaults() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("schema.yaml");

        create_template_generator()
            .generate(&file, TemplateRenderer::with_format(Format::Yaml))
            .unwrap();

        assert_snapshot!(fs::read_to_string(file).unwrap());
    }
}

#[cfg(feature = "typescript")]
mod typescript {
    use super::*;
    use schematic::schema::typescript::*;

    fn generate(options: TypeScriptOptions) -> String {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("types.ts");

        create_generator()
            .generate(&file, TypeScriptRenderer::new(options))
            .unwrap();

        fs::read_to_string(file).unwrap()
    }

    #[test]
    fn defaults() {
        assert_snapshot!(generate(TypeScriptOptions::default()));
    }

    #[test]
    fn enums() {
        assert_snapshot!(generate(TypeScriptOptions {
            enum_format: EnumFormat::Enum,
            ..TypeScriptOptions::default()
        }));
    }

    #[test]
    fn value_enums() {
        assert_snapshot!(generate(TypeScriptOptions {
            enum_format: EnumFormat::ValuedEnum,
            ..TypeScriptOptions::default()
        }));
    }

    #[test]
    fn const_enums() {
        assert_snapshot!(generate(TypeScriptOptions {
            const_enum: true,
            enum_format: EnumFormat::Enum,
            ..TypeScriptOptions::default()
        }));
    }

    #[test]
    fn object_aliases() {
        assert_snapshot!(generate(TypeScriptOptions {
            object_format: ObjectFormat::Type,
            ..TypeScriptOptions::default()
        }));
    }

    #[test]
    fn exclude_refs() {
        assert_snapshot!(generate(TypeScriptOptions {
            exclude_references: vec!["BasicEnum".into(), "AnotherType".into()],
            ..TypeScriptOptions::default()
        }));
    }

    #[test]
    fn external_types() {
        assert_snapshot!(generate(TypeScriptOptions {
            external_types: HashMap::from_iter([(
                "./externals".into(),
                vec!["BasicEnum".into(), "AnotherType".into()]
            )]),
            ..TypeScriptOptions::default()
        }));
    }

    #[test]
    fn no_refs() {
        assert_snapshot!(generate(TypeScriptOptions {
            disable_references: true,
            indent_char: "  ".into(),
            ..TypeScriptOptions::default()
        }));
    }
}
