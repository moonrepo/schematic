#![allow(dead_code, deprecated)]

use indexmap::{IndexMap, IndexSet};
use schematic::schema::{SchemaGenerator, TemplateOptions};
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

derive_enum!(
    #[derive(ConfigEnum, Default)]
    pub enum FallbackEnum {
        #[default]
        Foo,
        Bar,
        Baz,
        #[variant(fallback)]
        Other(String),
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
#[deprecated]
struct GenConfig {
    boolean: bool,
    string: String,
    number: usize,
    float32: f32,
    float64: f64,
    /// This is a list of strings.
    vector: Vec<String>,
    map: HashMap<String, u64>,
    /// This is a list of `enumerable` values.
    enums: BasicEnum,
    fallback_enum: FallbackEnum,
    /// **Nested** field.
    #[setting(nested)]
    nested: AnotherConfig,

    // Types
    date: chrono::NaiveDate,
    datetime: chrono::NaiveDateTime,
    decimal: rust_decimal::Decimal,
    time: chrono::NaiveTime,
    path: PathBuf,
    regex: RegexSetting,
    rel_path: relative_path::RelativePathBuf,
    url: Option<url::Url>,
    uuid: uuid::Uuid,
    version: Option<semver::Version>,
    version2: VersionSetting,
    version_req: semver::VersionReq,
    json_value: serde_json::Value,
    toml_value: Option<toml::Value>,
    yaml_value: serde_norway::Value,
    indexmap: IndexMap<String, String>,
    indexset: Option<IndexSet<String>>,
}

/// Some comment.
#[derive(Clone, Config)]
#[config(env_prefix = "ENV_PREFIX_")]
pub struct TwoDepthConfig {
    /// An optional string.
    opt: Option<String>,
    skipped: String,
}

/// Some comment.
#[derive(Clone, Config)]
pub struct OneDepthConfig {
    /// This is another nested field.
    #[setting(nested)]
    two: TwoDepthConfig,
    #[setting(skip)]
    skipped: String,
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
    fallback_enum: FallbackEnum,
    /// This is a nested struct with its own fields.
    #[setting(nested)]
    nested: AnotherConfig,
    /// This is a nested struct with its own fields.
    #[setting(nested)]
    one: OneDepthConfig,
    skipped: String,

    /// This field is testing array expansion.
    #[setting(nested)]
    expand_array: Vec<AnotherConfig>,
    expand_array_primitive: Vec<usize>,
    empty_array: Vec<usize>,

    /// This field is testing object expansion.
    #[setting(nested)]
    expand_object: HashMap<String, AnotherConfig>,
    expand_object_primitive: HashMap<String, usize>,
    empty_object: HashMap<String, usize>,
}

fn create_generator() -> SchemaGenerator {
    let mut generator = SchemaGenerator::default();
    generator.add::<GenConfig>();
    generator
}

fn create_template_generator() -> SchemaGenerator {
    let mut generator = SchemaGenerator::default();
    generator.add::<TemplateConfig>();
    generator
}

fn create_template_options() -> TemplateOptions {
    TemplateOptions {
        comment_fields: vec!["float32".into(), "map".into()],
        expand_fields: vec![
            "expandArray".into(),
            "expandArrayPrimitive".into(),
            "expandObject".into(),
            "expandObjectPrimitive".into(),
        ],
        hide_fields: vec!["skipped".into(), "one.two.skipped".into()],
        ..TemplateOptions::default()
    }
}

#[cfg(feature = "renderer_json_schema")]
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

    #[test]
    fn partials() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("schema.json");

        let mut generator = create_generator();
        generator.add::<PartialGenConfig>();
        generator
            .generate(&file, JsonSchemaRenderer::default())
            .unwrap();

        assert_snapshot!(fs::read_to_string(file).unwrap());
    }

    #[test]
    fn not_required() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("schema.json");

        create_generator()
            .generate(
                &file,
                JsonSchemaRenderer::new(JsonSchemaOptions {
                    mark_struct_fields_required: false,
                    ..JsonSchemaOptions::default()
                }),
            )
            .unwrap();

        assert_snapshot!(fs::read_to_string(file).unwrap());
    }

    #[test]
    fn with_titles() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("schema.json");

        create_generator()
            .generate(
                &file,
                JsonSchemaRenderer::new(JsonSchemaOptions {
                    set_field_name_as_title: true,
                    ..JsonSchemaOptions::default()
                }),
            )
            .unwrap();

        assert_snapshot!(fs::read_to_string(file).unwrap());
    }

    #[test]
    fn with_markdown_descs() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("schema.json");

        create_generator()
            .generate(
                &file,
                JsonSchemaRenderer::new(JsonSchemaOptions {
                    markdown_description: true,
                    ..JsonSchemaOptions::default()
                }),
            )
            .unwrap();

        assert_snapshot!(fs::read_to_string(file).unwrap());
    }
}

#[cfg(all(feature = "renderer_template", feature = "json"))]
mod template_json {
    use super::*;
    use schematic::schema::*;

    #[test]
    fn defaults() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("schema.json");

        create_template_generator()
            .generate(&file, JsoncTemplateRenderer::new(create_template_options()))
            .unwrap();

        assert_snapshot!(fs::read_to_string(file).unwrap());
    }

    #[test]
    fn without_comments() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("schema.json");

        create_template_generator()
            .generate(&file, JsonTemplateRenderer::new(create_template_options()))
            .unwrap();

        assert_snapshot!(fs::read_to_string(file).unwrap());
    }
}

#[cfg(all(feature = "renderer_template", feature = "pkl"))]
mod template_pkl {
    use super::*;
    use schematic::schema::*;

    #[test]
    fn defaults() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("schema.pkl");

        create_template_generator()
            .generate(&file, PklTemplateRenderer::new(create_template_options()))
            .unwrap();

        assert_snapshot!(fs::read_to_string(file).unwrap());
    }
}

#[cfg(all(feature = "renderer_template", feature = "toml"))]
mod template_toml {
    use super::*;
    use schematic::schema::*;

    #[test]
    fn defaults() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("schema.toml");

        create_template_generator()
            .generate(&file, TomlTemplateRenderer::new(create_template_options()))
            .unwrap();

        assert_snapshot!(fs::read_to_string(file).unwrap());
    }
}

#[cfg(all(feature = "renderer_template", feature = "yaml"))]
mod template_yaml {
    use super::*;
    use schematic::schema::*;

    #[test]
    fn defaults() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("schema.yaml");

        create_template_generator()
            .generate(&file, YamlTemplateRenderer::new(create_template_options()))
            .unwrap();

        assert_snapshot!(fs::read_to_string(file).unwrap());
    }

    // https://github.com/moonrepo/schematic/issues/139
    #[test]
    fn issue_139() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("schema.yaml");

        #[derive(Debug, Clone, Config)]
        pub struct ProjectConfig {
            #[setting(nested)]
            pub nested: NestedConfig,
        }

        #[derive(Debug, Clone, Config)]
        pub struct NestedConfig {
            #[setting(default = true)]
            pub one: bool,
        }

        let mut generator = SchemaGenerator::default();
        generator.add::<ProjectConfig>();
        generator
            .generate(&file, YamlTemplateRenderer::new(create_template_options()))
            .unwrap();

        assert_snapshot!(fs::read_to_string(file).unwrap());
    }
}

#[cfg(feature = "renderer_typescript")]
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
    fn partials() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("types.ts");

        let mut generator = create_generator();
        generator.add::<PartialGenConfig>();
        generator
            .generate(&file, TypeScriptRenderer::new(TypeScriptOptions::default()))
            .unwrap();

        assert_snapshot!(fs::read_to_string(file).unwrap());
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
    fn props_optional() {
        assert_snapshot!(generate(TypeScriptOptions {
            property_format: PropertyFormat::Optional,
            ..TypeScriptOptions::default()
        }));
    }

    #[test]
    fn props_optional_undefined() {
        assert_snapshot!(generate(TypeScriptOptions {
            property_format: PropertyFormat::OptionalUndefined,
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
