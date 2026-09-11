#![allow(dead_code, deprecated)]

use indexmap::{IndexMap, IndexSet};
use schematic::schema::{IntegerKind, IntegerType, SchemaGenerator, StringType, TemplateOptions};
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
    /// Flattened field...
    #[setting(flatten)]
    flattened: HashMap<String, serde_json::Value>,

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

/// A port number, constrained to the registered range.
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
struct Port(u16);

impl Schematic for Port {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.integer(IntegerType {
            kind: IntegerKind::U16,
            min: Some(1024),
            max: Some(49151),
            ..IntegerType::default()
        })
    }
}

/// A short identifier.
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
struct Ident(String);

impl Schematic for Ident {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.string(StringType {
            max_length: Some(32),
            min_length: Some(1),
            pattern: Some("^[a-z][a-z0-9-]*$".into()),
            ..StringType::default()
        })
    }
}

/// Either a list of targets, or a map of targets to paths.
#[derive(Clone, Config)]
#[serde(untagged)]
enum Targets {
    /// A list of targets.
    List(Vec<String>),
    /// A map of targets.
    Map(HashMap<String, String>),
}

/// A config that exercises everything the docs can describe.
///
/// - With a list item.
/// - And another.
#[derive(Clone, Config)]
struct DocsConfig {
    /// A string with a default and aliases.
    #[serde(alias = "label", alias = "title")]
    #[setting(default = "hello", env = "DOCS_NAME")]
    name: String,
    /// A constrained integer.
    port: Port,
    /// A constrained string.
    ident: Option<Ident>,
    /// An optional nested config.
    #[setting(nested)]
    nested: Option<AnotherConfig>,
    /// A list of nested configs.
    #[setting(nested)]
    list: Vec<AnotherConfig>,
    /// A map of enums.
    map: HashMap<String, BasicEnum>,
    /// An enum, which carries its own default.
    level: BasicEnum,
    /// A deprecated field, use `name` instead.
    #[deprecated = "Use `name` instead."]
    old_name: Option<String>,
    /// A tuple of values.
    pair: (String, u32),
    /// An inline struct.
    duration: std::time::Duration,
    /// A union of a list or a map.
    #[setting(nested)]
    targets: Targets,
    /// A list of nullable enums.
    nullable_items: Vec<Option<BasicEnum>>,
    /// An optional boolean.
    #[serde(default)]
    boolean: bool,
    #[setting(skip)]
    hidden: String,
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
            "expand_array".into(),
            "expand_array_primitive".into(),
            "expand_object".into(),
            "expand_object_primitive".into(),
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

    #[test]
    fn only_fields() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("schema.json");

        create_template_generator()
            .generate(
                &file,
                JsoncTemplateRenderer::new({
                    let mut options = create_template_options();
                    options.only_fields.push("float32".into());
                    options.only_fields.push("string".into());
                    options
                }),
            )
            .unwrap();

        assert_snapshot!(fs::read_to_string(file).unwrap());
    }

    #[test]
    fn custom_values() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("schema.json");

        create_template_generator()
            .generate(
                &file,
                JsoncTemplateRenderer::new({
                    let mut options = create_template_options();
                    options.custom_values.insert(
                        "expandArrayPrimitive".into(),
                        Schema::array(ArrayType::new(Schema::integer(IntegerType::new_unsigned(
                            IntegerKind::Usize,
                            123456,
                        )))),
                    );
                    options.custom_values.insert(
                        "nested.opt".into(),
                        Schema::union(UnionType::new_any([
                            Schema::string(StringType::new("custom nested value")),
                            Schema::null(),
                        ])),
                    );
                    options.custom_values.insert(
                        "string".into(),
                        Schema::string(StringType::new("custom value")),
                    );
                    options
                }),
            )
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

#[cfg(feature = "renderer_api_docs")]
mod api_docs {
    use super::*;
    use schematic::schema::api_docs::*;

    fn generate(generator: SchemaGenerator, options: ApiDocsOptions) -> String {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("docs.md");

        generator
            .generate(&file, ApiDocsRenderer::new(options))
            .unwrap();

        fs::read_to_string(file).unwrap()
    }

    #[test]
    fn defaults() {
        assert_snapshot!(generate(create_generator(), ApiDocsOptions::default()));
    }

    #[test]
    fn partials() {
        let mut generator = create_generator();
        generator.add::<PartialGenConfig>();

        assert_snapshot!(generate(generator, ApiDocsOptions::default()));
    }

    #[test]
    fn all_field_shapes() {
        let mut generator = SchemaGenerator::default();
        generator.add::<DocsConfig>();

        assert_snapshot!(generate(generator, ApiDocsOptions::default()));
    }

    #[test]
    fn template_config() {
        assert_snapshot!(generate(
            create_template_generator(),
            ApiDocsOptions::default()
        ));
    }

    #[test]
    fn unit_enum() {
        let mut generator = SchemaGenerator::default();
        generator.add::<BasicEnum>();

        assert_snapshot!(generate(generator, ApiDocsOptions::default()));
    }

    #[test]
    fn fallback_enum() {
        let mut generator = SchemaGenerator::default();
        generator.add::<FallbackEnum>();

        assert_snapshot!(generate(generator, ApiDocsOptions::default()));
    }

    #[test]
    fn union() {
        let mut generator = SchemaGenerator::default();
        generator.add::<Targets>();

        assert_snapshot!(generate(generator, ApiDocsOptions::default()));
    }

    // A type nested by an earlier one is already in the generator, so adding
    // it again has to make it the page that renders.
    #[test]
    fn readded_nested_type_becomes_the_page() {
        let mut generator = create_generator();
        generator.add::<AnotherConfig>();

        let output = generate(generator, ApiDocsOptions::default());

        assert!(output.starts_with("---\ntitle: AnotherConfig\n---"));
        assert_snapshot!(output);
    }

    #[test]
    fn without_required_or_aliases() {
        let mut generator = SchemaGenerator::default();
        generator.add::<DocsConfig>();

        assert_snapshot!(generate(
            generator,
            ApiDocsOptions {
                exclude_aliases: true,
                mark_struct_fields_required: false,
                ..ApiDocsOptions::default()
            }
        ));
    }

    #[test]
    fn custom_link_extension() {
        let mut generator = SchemaGenerator::default();
        generator.add::<DocsConfig>();

        let output = generate(
            generator,
            ApiDocsOptions {
                link_extension: String::new(),
                ..ApiDocsOptions::default()
            },
        );

        assert!(output.contains("[`AnotherConfig`](./AnotherConfig)"));
        assert!(output.contains("- [AnotherConfig](./AnotherConfig)"));
        assert!(!output.contains(".md"));
    }

    #[test]
    fn errors_without_schemas() {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("docs.md");

        let error = SchemaGenerator::default()
            .generate(&file, ApiDocsRenderer::default())
            .unwrap_err();

        assert!(error.to_string().contains("At least one type"));
    }
}

mod name_collisions {
    use super::*;

    // Schemas are keyed by name alone, so two types answering with the same
    // one would collapse into a single definition and every reference to it
    // would resolve to whichever was added first.
    struct First;
    struct Second;

    macro_rules! impl_collides {
        ($type:ty) => {
            impl Schematic for $type {
                fn schema_name() -> Option<String> {
                    Some("Collides".into())
                }

                fn build_schema(mut schema: SchemaBuilder) -> Schema {
                    schema.string_default()
                }
            }
        };
    }

    impl_collides!(First);
    impl_collides!(Second);

    #[test]
    #[should_panic(expected = "A schema named `Collides` has already been added")]
    fn reports_two_types_sharing_a_name() {
        let mut generator = SchemaGenerator::default();

        generator.add::<First>();
        generator.add::<Second>();
    }

    #[test]
    fn allows_adding_the_same_type_twice() {
        let mut generator = SchemaGenerator::default();

        generator.add::<First>();
        generator.add::<First>();

        assert_eq!(generator.schemas.len(), 1);
    }

    // Recursive configs legitimately register the same name at several
    // expansion depths, which must not be mistaken for a collision.
    #[derive(Config)]
    struct Recurses {
        #[setting(nested)]
        children: Vec<Recurses>,
    }

    #[test]
    fn allows_a_recursive_type() {
        let mut generator = SchemaGenerator::default();

        generator.add::<PartialRecurses>();

        assert!(generator.schemas.contains_key("PartialRecurses"));
    }
}
