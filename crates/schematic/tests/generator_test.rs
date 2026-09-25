#![allow(dead_code, deprecated)]

use indexmap::{IndexMap, IndexSet};
use schematic::schema::{IntegerKind, IntegerType, SchemaGenerator, StringType, TemplateOptions};
use schematic::*;
use starbase_sandbox::{assert_snapshot, create_empty_sandbox};
use std::collections::{BTreeMap, HashMap};
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
    fn enum_table() {
        let mut generator = SchemaGenerator::default();
        generator.add::<FallbackEnum>();

        assert_snapshot!(generate(
            generator,
            ApiDocsOptions {
                enum_format: ApiDocsEnumFormat::Table,
                ..ApiDocsOptions::default()
            }
        ));
    }

    // The table format only applies to unit enums, as union variants carry
    // values that need a section to describe
    #[test]
    fn enum_table_leaves_unions_as_sections() {
        let mut generator = SchemaGenerator::default();
        generator.add::<Targets>();

        let output = generate(
            generator,
            ApiDocsOptions {
                enum_format: ApiDocsEnumFormat::Table,
                ..ApiDocsOptions::default()
            },
        );

        assert!(output.contains("## Index"));
        assert!(output.contains("### `List`"));
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
    fn custom_frontmatter() {
        let mut generator = SchemaGenerator::default();
        generator.add::<AnotherConfig>();

        let output = generate(
            generator,
            ApiDocsOptions {
                frontmatter: BTreeMap::from_iter([
                    ("sidebar_position".to_owned(), "2".to_owned()),
                    ("description".to_owned(), "\"Another: config\"".to_owned()),
                ]),
                ..ApiDocsOptions::default()
            },
        );

        assert!(output.starts_with(
            "---\ntitle: AnotherConfig\ndescription: \"Another: config\"\nsidebar_position: 2\n---\n\nSome comment."
        ));
    }

    #[test]
    fn frontmatter_title_overrides_the_type_name() {
        let mut generator = SchemaGenerator::default();
        generator.add::<AnotherConfig>();

        let output = generate(
            generator,
            ApiDocsOptions {
                frontmatter: BTreeMap::from_iter([("title".to_owned(), "Another".to_owned())]),
                ..ApiDocsOptions::default()
            },
        );

        assert!(output.starts_with("---\ntitle: Another\n---\n"));
        assert!(!output.contains("title: AnotherConfig"));
    }

    #[test]
    fn without_index() {
        let mut generator = SchemaGenerator::default();
        generator.add::<DocsConfig>();

        let output = generate(
            generator,
            ApiDocsOptions {
                include_index: false,
                ..ApiDocsOptions::default()
            },
        );

        assert!(!output.contains("## Index"));
        assert!(output.contains("## Properties"));
    }

    // A documentation tool that ids headings its own way needs the index
    // links to follow it
    #[test]
    fn custom_anchors() {
        let mut generator = SchemaGenerator::default();
        generator.add::<DocsConfig>();

        let output = generate(
            generator,
            ApiDocsOptions {
                render_anchor: Box::new(|heading| format!("prop-{}", heading.replace('_', "-"))),
                ..ApiDocsOptions::default()
            },
        );

        assert!(output.contains("| [`nullable_items`](#prop-nullable-items) |"));
        assert!(!output.contains("](#nullable_items)"));
    }

    // The same function shapes inline links and the references list
    #[test]
    fn custom_links() {
        let mut generator = SchemaGenerator::default();
        generator.add::<DocsConfig>();

        let output = generate(
            generator,
            ApiDocsOptions {
                render_link: Box::new(|name| {
                    format!("<Link to=\"/docs/config/{name}\">{name}</Link>")
                }),
                ..ApiDocsOptions::default()
            },
        );

        assert!(
            output.contains(
                "| Type | <Link to=\"/docs/config/AnotherConfig\">AnotherConfig</Link> |"
            )
        );
        assert!(output.contains("- <Link to=\"/docs/config/AnotherConfig\">AnotherConfig</Link>"));
        // A composite type stays one code span, and links its references beside it
        assert!(output.contains(
            "| Type | `AnotherConfig[]` |\n| References | <Link to=\"/docs/config/AnotherConfig\">AnotherConfig</Link> |"
        ));
        assert!(!output.contains(".md"));
    }

    #[test]
    fn custom_tags_and_description() {
        let mut generator = SchemaGenerator::default();
        generator.add::<DocsConfig>();

        assert_snapshot!(generate(
            generator,
            ApiDocsOptions {
                render_tags: Box::new(|tags| {
                    tags.iter()
                        .map(|tag| match tag {
                            ApiDocsTag::Deprecated(Some(message)) => {
                                format!("<Badge>{tag}</Badge> {message}")
                            }
                            _ => format!("<Badge>{tag}</Badge>"),
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                }),
                render_description: Box::new(|description| {
                    format!(":::note\n{}\n:::", description.trim())
                }),
                ..ApiDocsOptions::default()
            }
        ));
    }

    // An empty result from either function omits that block, rather than
    // leaving a blank line where it would have been
    #[test]
    fn empty_tags_and_description_are_omitted() {
        let mut generator = SchemaGenerator::default();
        generator.add::<AnotherConfig>();

        let output = generate(
            generator,
            ApiDocsOptions {
                render_tags: Box::new(|_| String::new()),
                render_description: Box::new(|_| String::new()),
                ..ApiDocsOptions::default()
            },
        );

        assert!(!output.contains("Nullable"));
        assert!(!output.contains("Some comment."));
        assert!(!output.contains("\n\n\n"));
        assert!(output.contains("### `enums`\n\n| Attribute | Value |"));
    }

    #[test]
    fn generate_all_writes_every_page_and_an_index() {
        let sandbox = create_empty_sandbox();
        let dir = sandbox.path().join("docs");

        ApiDocsRenderer::default()
            .generate_all(&create_generator(), &dir)
            .unwrap();

        let mut files = fs::read_dir(&dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<_>>();
        files.sort();

        assert_eq!(
            files,
            vec![
                "AnotherConfig.md",
                "BasicEnum.md",
                "FallbackEnum.md",
                "GenConfig.md",
                "index.md",
            ]
        );

        // A page is the same as the one a single generate renders
        let single = generate(create_generator(), ApiDocsOptions::default());

        assert_eq!(
            fs::read_to_string(dir.join("GenConfig.md")).unwrap(),
            single
        );
        assert!(
            fs::read_to_string(dir.join("AnotherConfig.md"))
                .unwrap()
                .starts_with("---\ntitle: AnotherConfig\n---")
        );

        assert_snapshot!(fs::read_to_string(dir.join("index.md")).unwrap());
    }

    #[test]
    fn generate_all_without_index_page() {
        let sandbox = create_empty_sandbox();
        let dir = sandbox.path().join("docs");

        ApiDocsRenderer::new(ApiDocsOptions {
            index_page: None,
            ..ApiDocsOptions::default()
        })
        .generate_all(&create_generator(), &dir)
        .unwrap();

        assert!(!dir.join("index.md").exists());
        assert!(dir.join("GenConfig.md").exists());
    }

    #[test]
    fn generate_all_with_custom_file_extension() {
        let sandbox = create_empty_sandbox();
        let dir = sandbox.path().join("docs");

        ApiDocsRenderer::new(ApiDocsOptions {
            file_extension: ".mdx".into(),
            index_page: Some("index.mdx".into()),
            ..ApiDocsOptions::default()
        })
        .generate_all(&create_generator(), &dir)
        .unwrap();

        assert!(dir.join("GenConfig.mdx").exists());
        assert!(dir.join("index.mdx").exists());
        assert!(!dir.join("GenConfig.md").exists());
    }

    #[test]
    fn generate_all_errors_without_schemas() {
        let sandbox = create_empty_sandbox();

        let error = ApiDocsRenderer::default()
            .generate_all(&SchemaGenerator::default(), sandbox.path().join("docs"))
            .unwrap_err();

        assert!(error.to_string().contains("At least one type"));
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

#[cfg(feature = "renderer_pkl_schema")]
mod pkl_schema {
    use super::*;
    use schematic::schema::pkl_schema::*;
    use std::collections::{BTreeSet, HashSet};
    use std::num::NonZeroU8;
    use std::path::Path;
    use std::time::Duration;

    /// A base that other configs flatten.
    #[derive(Clone, Config)]
    pub struct BaseConfig {
        /// A setting every config shares.
        #[setting(default = "shared")]
        shared: String,
        count: Option<u32>,
    }

    /// Flattens the base, alongside settings of its own.
    #[derive(Clone, Config)]
    #[config(allow_unknown_fields)]
    pub struct ExtendingConfig {
        #[setting(flatten, nested)]
        base: BaseConfig,
        own: bool,
    }

    /// Flattens the base, and nothing else.
    #[derive(Clone, Config)]
    #[config(allow_unknown_fields)]
    pub struct AmendingConfig {
        #[setting(flatten, nested)]
        base: BaseConfig,
    }

    /// Refers to a config that would otherwise amend its base.
    #[derive(Clone, Config)]
    pub struct UsesAmendingConfig {
        #[setting(nested)]
        amending: AmendingConfig,
    }

    #[derive(Clone, Config)]
    pub struct TreeConfig {
        name: String,
        #[setting(nested)]
        children: Vec<TreeConfig>,
    }

    /// A value, or a list of values.
    #[derive(Clone, Schematic)]
    #[serde(untagged)]
    pub enum Expr {
        Value(String),
        List(Vec<Expr>),
    }

    #[derive(Clone, Config)]
    pub struct ExternalTaggedConfig {
        name: String,
    }

    #[derive(Clone, Config)]
    enum ExternalTagged {
        Foo,
        Bar(bool),
        /// A pair of values.
        Baz(usize, String),
        #[setting(nested)]
        Qux(ExternalTaggedConfig),
    }

    #[derive(Clone, Config)]
    #[serde(tag = "type")]
    enum InternalTagged {
        Foo,
        #[setting(nested)]
        Qux(ExternalTaggedConfig),
    }

    #[derive(Clone, Config)]
    #[serde(tag = "type", content = "content")]
    enum AdjacentTagged {
        Foo,
        Bar(bool),
        #[setting(nested)]
        Qux(ExternalTaggedConfig),
    }

    #[derive(Clone, Config)]
    struct TaggedConfig {
        #[setting(nested)]
        external: ExternalTagged,
        #[setting(nested)]
        internal: Option<InternalTagged>,
        #[setting(nested)]
        adjacent: Option<AdjacentTagged>,
    }

    #[derive(Clone, Config)]
    struct IdentifiersConfig {
        #[setting(rename = "class")]
        class_name: String,
        #[setting(rename = "kebab-case")]
        kebab_case: Option<String>,
        #[setting(rename = "$schema")]
        schema: Option<String>,
        default: Option<String>,
        output: Option<String>,
    }

    #[derive(
        Clone, ConfigEnum, Debug, Default, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize,
    )]
    #[serde(rename_all = "kebab-case")]
    pub enum KeyEnum {
        #[default]
        Alpha,
        Beta,
    }

    /// Covers the conversion of each Rust type.
    #[derive(Clone, Config)]
    struct TypesConfig {
        int8: i8,
        int16: i16,
        int32: i32,
        int64: i64,
        int128: i128,
        isize: isize,
        uint8: u8,
        uint16: u16,
        uint32: u32,
        uint64: u64,
        uint128: u128,
        usize: usize,
        non_zero: Option<NonZeroU8>,
        #[setting(default = 1.0)]
        float32: f32,
        #[setting(default = 2.5)]
        float64: f64,
        character: char,
        #[setting(default = "quote \" and backslash \\ and \\(interpolation)")]
        escaped: String,
        fixed_array: [String; 3],
        set: HashSet<String>,
        btree_set: BTreeSet<u8>,
        pair: (String, u32),
        triple: (String, u32, bool),
        duration: Duration,
        maybe_duration: Option<Duration>,
        enum_map: HashMap<KeyEnum, String>,
        nullable_items: Vec<Option<BasicEnum>>,
        port: Port,
        ident: Ident,
        json: serde_json::Value,
        #[setting(default = "bar")]
        level: BasicEnum,
        #[deprecated = "Use `level` instead."]
        old_level: Option<BasicEnum>,
    }

    fn render(generator: SchemaGenerator, options: PklSchemaOptions) -> String {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join("module.pkl");

        generator
            .generate(&file, PklSchemaRenderer::new(options))
            .unwrap();

        fs::read_to_string(file).unwrap()
    }

    fn render_type<T: Schematic>() -> String {
        render_type_with::<T>(PklSchemaOptions::default())
    }

    fn render_type_with<T: Schematic>(options: PklSchemaOptions) -> String {
        let mut generator = SchemaGenerator::default();
        generator.add::<T>();

        render(generator, options)
    }

    /// Render every module in the generator, keyed by file name.
    fn render_all(
        generator: &SchemaGenerator,
        options: PklSchemaOptions,
    ) -> BTreeMap<String, String> {
        let sandbox = create_empty_sandbox();

        PklSchemaRenderer::new(options)
            .generate_all(generator, sandbox.path())
            .unwrap();

        fs::read_dir(sandbox.path())
            .unwrap()
            .map(|entry| {
                let path = entry.unwrap().path();

                (
                    path.file_name().unwrap().to_string_lossy().into_owned(),
                    fs::read_to_string(path).unwrap(),
                )
            })
            .collect()
    }

    #[test]
    fn defaults() {
        assert_snapshot!(render(create_generator(), PklSchemaOptions::default()));
    }

    #[test]
    fn partials() {
        let mut generator = create_generator();
        generator.add::<PartialGenConfig>();

        assert_snapshot!(render(generator, PklSchemaOptions::default()));
    }

    #[test]
    fn not_required() {
        assert_snapshot!(render(
            create_generator(),
            PklSchemaOptions {
                mark_struct_fields_required: false,
                ..PklSchemaOptions::default()
            }
        ));
    }

    #[test]
    fn all_field_shapes() {
        assert_snapshot!(render_type::<DocsConfig>());
    }

    #[test]
    fn rust_types() {
        assert_snapshot!(render_type::<TypesConfig>());
    }

    #[test]
    fn unit_enum() {
        assert_snapshot!(render_type::<BasicEnum>());
    }

    #[test]
    fn fallback_enum() {
        assert_snapshot!(render_type::<FallbackEnum>());
    }

    #[test]
    fn union() {
        assert_snapshot!(render_type::<Targets>());
    }

    #[test]
    fn tagged_enums() {
        let mut generator = SchemaGenerator::default();
        generator.add::<TaggedConfig>();

        let modules = render_all(&generator, PklSchemaOptions::default());

        assert_snapshot!(
            "tagged_enums_external",
            modules.get("ExternalTagged.pkl").unwrap()
        );
        assert_snapshot!(
            "tagged_enums_internal",
            modules.get("InternalTagged.pkl").unwrap()
        );
        assert_snapshot!(
            "tagged_enums_adjacent",
            modules.get("AdjacentTagged.pkl").unwrap()
        );
    }

    // A partial nullifies every field of the struct a variant holds, which
    // includes the tag an internally tagged variant adds to it, but serde
    // still requires the tag
    #[test]
    fn keeps_tags_of_partial_variants() {
        let mut generator = SchemaGenerator::default();
        // Registers the struct the variant holds without the tag
        generator.add::<PartialTaggedConfig>();
        generator.add::<PartialInternalTagged>();

        let output = render(generator, PklSchemaOptions::default());

        assert!(
            output.contains(
                "class PartialInternalTaggedQux {\n  name: String?\n\n  type: \"Qux\"\n}"
            )
        );
    }

    #[test]
    fn quotes_identifiers() {
        assert_snapshot!(render_type::<IdentifiersConfig>());
    }

    #[test]
    fn indent_char() {
        let output = render_type_with::<ExternalTagged>(PklSchemaOptions {
            indent_char: "\t".into(),
            ..PklSchemaOptions::default()
        });

        assert!(output.contains("class ExternalTaggedBar {\n\tBar: Boolean\n}"));
    }

    // A class holds a value, such as a variant's payload, which is only valid
    // in full, so its properties keep their types, and a tag its default
    #[test]
    fn not_required_leaves_classes_as_declared() {
        let output = render_type_with::<AdjacentTagged>(PklSchemaOptions {
            mark_struct_fields_required: false,
            ..PklSchemaOptions::default()
        });

        assert!(
            output.contains("class AdjacentTaggedBar {\n  content: Boolean\n\n  type: \"Bar\"\n}")
        );
    }

    // A struct that flattens another extends its module, which has to be
    // open for that
    #[test]
    fn extends_a_flattened_struct() {
        let mut generator = SchemaGenerator::default();
        generator.add::<ExtendingConfig>();

        let modules = render_all(&generator, PklSchemaOptions::default());

        assert_snapshot!(modules.get("ExtendingConfig.pkl").unwrap());
        assert!(
            modules
                .get("BaseConfig.pkl")
                .unwrap()
                .contains("\nopen module BaseConfig\n")
        );
    }

    // With nothing of its own to declare, the struct is the same shape as the
    // struct it flattens, so its module amends that one instead
    #[test]
    fn amends_a_flattened_struct_when_nothing_else_is_declared() {
        let mut generator = SchemaGenerator::default();
        generator.add::<AmendingConfig>();

        let modules = render_all(&generator, PklSchemaOptions::default());

        assert_snapshot!(modules.get("AmendingConfig.pkl").unwrap());
        assert!(
            modules
                .get("BaseConfig.pkl")
                .unwrap()
                .contains("\nmodule BaseConfig\n")
        );
    }

    // Pkl refuses to use a module that amends another as a type
    #[test]
    fn extends_instead_of_amending_when_used_as_a_type() {
        let mut generator = SchemaGenerator::default();
        generator.add::<UsesAmendingConfig>();

        let modules = render_all(&generator, PklSchemaOptions::default());

        assert!(
            modules
                .get("AmendingConfig.pkl")
                .unwrap()
                .contains("\nextends \"BaseConfig.pkl\"")
        );
        assert!(
            modules
                .get("BaseConfig.pkl")
                .unwrap()
                .contains("\nopen module BaseConfig\n")
        );
    }

    #[test]
    fn recursive_struct_imports_itself() {
        let output = render_type::<TreeConfig>();

        assert!(output.contains("import \"TreeConfig.pkl\""));
        assert!(output.contains("children: Listing<TreeConfig>"));
    }

    // Pkl rejects a type alias that refers to itself
    #[test]
    fn recursive_union_breaks_the_cycle() {
        assert_snapshot!(render_type::<Expr>());
    }

    #[test]
    fn generate_all_writes_every_module() {
        let modules = render_all(&create_generator(), PklSchemaOptions::default());

        assert_eq!(
            modules.keys().collect::<Vec<_>>(),
            vec![
                "AnotherConfig.pkl",
                "BasicEnum.pkl",
                "FallbackEnum.pkl",
                "GenConfig.pkl",
            ]
        );

        // A module is the same as the one a single generate renders
        assert_eq!(
            modules.get("GenConfig.pkl").unwrap(),
            &render(create_generator(), PklSchemaOptions::default())
        );
    }

    #[test]
    fn generate_all_errors_without_schemas() {
        let sandbox = create_empty_sandbox();

        let error = PklSchemaRenderer::default()
            .generate_all(&SchemaGenerator::default(), sandbox.path())
            .unwrap_err();

        assert!(error.to_string().contains("At least one type"));
    }

    #[test]
    fn errors_without_schemas() {
        let sandbox = create_empty_sandbox();

        let error = SchemaGenerator::default()
            .generate(
                sandbox.path().join("module.pkl"),
                PklSchemaRenderer::default(),
            )
            .unwrap_err();

        assert!(error.to_string().contains("At least one type"));
    }

    #[cfg(feature = "pkl")]
    mod loading {
        use super::*;

        #[derive(Clone, Config, Debug, PartialEq)]
        #[config(allow_unknown_fields)]
        pub struct ServerConfig {
            #[setting(default = 8080)]
            port: u16,
            host: Option<String>,
            timeout: Option<Duration>,
            pair: Option<(String, u32)>,
            tags: Vec<String>,
            labels: HashMap<String, String>,
            level: BasicEnum,
            #[setting(nested)]
            nested: Option<ServerNestedConfig>,
            #[setting(nested)]
            targets: Option<ServerTargets>,
            #[setting(flatten, nested)]
            base: ServerBaseConfig,
        }

        #[derive(Clone, Config, Debug, PartialEq)]
        pub struct ServerBaseConfig {
            #[setting(default = "shared")]
            shared: String,
        }

        #[derive(Clone, Config, Debug, PartialEq)]
        pub struct ServerNestedConfig {
            enabled: bool,
            name: Option<String>,
        }

        #[derive(Clone, Config, Debug, PartialEq)]
        #[serde(untagged)]
        pub enum ServerTargets {
            List(Vec<String>),
            Map(HashMap<String, String>),
        }

        fn write_modules(dir: &Path, options: PklSchemaOptions) {
            let mut generator = SchemaGenerator::default();
            generator.add::<ServerConfig>();

            PklSchemaRenderer::new(options)
                .generate_all(&generator, dir)
                .unwrap();
        }

        fn load(dir: &Path, config: &str) -> ServerConfig {
            let file = dir.join("config.pkl");

            fs::write(&file, format!("amends \"ServerConfig.pkl\"\n\n{config}")).unwrap();

            ConfigLoader::<ServerConfig>::new()
                .file(file)
                .unwrap()
                .load()
                .unwrap()
                .config
        }

        // Every property is nullable, so what the config leaves out falls
        // back to the default, instead of being set by the module
        #[test]
        fn loads_a_config_that_amends_the_modules() {
            let sandbox = create_empty_sandbox();

            write_modules(
                sandbox.path(),
                PklSchemaOptions {
                    mark_struct_fields_required: false,
                    ..PklSchemaOptions::default()
                },
            );

            let config = load(
                sandbox.path(),
                r#"
timeout = 1500.ms
pair = Pair("a", 1)
tags { "a"; "b" }
labels { ["x"] = "y" }
level = "bar"
nested { enabled = true }
targets = new Listing { "one"; "two" }
shared = "custom"
"#,
            );

            assert_eq!(config.port, 8080);
            assert_eq!(config.host, None);
            assert_eq!(config.timeout, Some(Duration::from_millis(1500)));
            assert_eq!(config.pair, Some(("a".into(), 1)));
            assert_eq!(config.tags, vec!["a".to_owned(), "b".to_owned()]);
            assert_eq!(
                config.labels,
                HashMap::from_iter([("x".to_owned(), "y".to_owned())])
            );
            assert_eq!(config.level, BasicEnum::Bar);
            assert_eq!(
                config.nested,
                Some(ServerNestedConfig {
                    enabled: true,
                    name: None,
                })
            );
            assert_eq!(
                config.targets,
                Some(ServerTargets::List(vec!["one".into(), "two".into()]))
            );
            assert_eq!(config.base.shared, "custom");
        }

        // A property's default is output by the module, and the enum's
        // default variant comes from its type
        #[test]
        fn loads_the_defaults_of_required_fields() {
            let sandbox = create_empty_sandbox();

            write_modules(sandbox.path(), PklSchemaOptions::default());

            let config = load(sandbox.path(), "");

            assert_eq!(config.port, 8080);
            assert_eq!(config.level, BasicEnum::Foo);
            assert_eq!(config.base.shared, "shared");
            assert!(config.tags.is_empty());
        }

        // A value outside the constraints of its type fails in Pkl, before it
        // reaches serde
        #[test]
        fn rejects_a_value_of_the_wrong_type() {
            let sandbox = create_empty_sandbox();

            write_modules(sandbox.path(), PklSchemaOptions::default());

            let file = sandbox.path().join("config.pkl");
            fs::write(&file, "amends \"ServerConfig.pkl\"\n\nport = 70000\n").unwrap();

            let error = ConfigLoader::<ServerConfig>::new()
                .file(file)
                .unwrap()
                .load()
                .err()
                .unwrap();

            assert!(format!("{error:?}").contains("UInt16"));
        }
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
