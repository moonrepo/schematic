//! Regressions for rendered output that the target language rejected, or
//! that described something other than what serde accepts. Each output is
//! parsed by a real parser where one is available, as a snapshot alone can't
//! say whether it's valid.
#![allow(dead_code)]

use schematic::schema::*;
use schematic::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn render<T: Schematic, O, R: SchemaRenderer<O>>(renderer: R) -> String {
    let mut generator = SchemaGenerator::default();
    generator.add::<T>();

    let mut renderer = renderer;

    renderer.render(generator.schemas).unwrap()
}

fn json_schema<T: Schematic>() -> serde_json::Value {
    json_schema_with::<T>(JsonSchemaOptions::default())
}

fn json_schema_with<T: Schematic>(options: JsonSchemaOptions) -> serde_json::Value {
    serde_json::from_str(&render::<T, _, _>(JsonSchemaRenderer::new(options))).unwrap()
}

fn template_options() -> TemplateOptions {
    TemplateOptions {
        comments: false,
        ..TemplateOptions::default()
    }
}

mod recursion {
    use super::*;

    #[derive(Schematic)]
    pub struct Linked {
        name: String,
        next: Option<Box<Linked>>,
    }

    #[derive(Deserialize, Schematic, Serialize)]
    pub struct Shared {
        value: String,
    }

    #[derive(Schematic)]
    pub struct Holder {
        boxed: Box<Shared>,
        plain: Shared,
    }

    #[test]
    fn a_boxed_self_is_a_cycle() {
        let schema = json_schema::<Linked>();

        assert_eq!(
            schema["properties"]["next"]["anyOf"][0]["$ref"],
            serde_json::json!("#")
        );
    }

    #[test]
    fn a_boxed_type_is_referenced_by_name() {
        let schema = json_schema::<Holder>();

        assert_eq!(
            schema["properties"]["boxed"]["$ref"],
            serde_json::json!("#/definitions/Shared")
        );
        assert_eq!(
            schema["properties"]["plain"]["$ref"],
            serde_json::json!("#/definitions/Shared")
        );
    }

    #[derive(Config)]
    pub struct TreeConfig {
        #[setting(nested)]
        children: Vec<TreeConfig>,
    }

    #[test]
    fn a_reference_to_the_root_is_the_document() {
        let schema = json_schema::<TreeConfig>();

        assert_eq!(
            schema["properties"]["children"]["items"]["$ref"],
            serde_json::json!("#")
        );
        assert!(schema.get("definitions").is_none());
    }
}

mod json_schema_shapes {
    use super::*;

    #[derive(Config)]
    pub struct BaseConfig {
        shared: String,
        count: usize,
    }

    #[derive(Config)]
    pub struct ExtendingConfig {
        own: bool,
        #[setting(flatten, nested)]
        base: BaseConfig,
    }

    #[test]
    fn merges_the_properties_of_a_flattened_struct() {
        let schema = json_schema::<ExtendingConfig>();
        let properties = schema["properties"].as_object().unwrap();

        assert!(properties.contains_key("own"));
        assert!(properties.contains_key("shared"));
        assert!(properties.contains_key("count"));
        assert_eq!(schema["additionalProperties"], serde_json::json!(false));
    }

    #[derive(Schematic)]
    pub struct Keys {
        numbers: HashMap<u32, String>,
        signed: HashMap<i32, String>,
        flags: HashMap<bool, String>,
        value: serde_json::Value,
    }

    #[test]
    fn describes_map_keys_as_strings() {
        let schema = json_schema::<Keys>();

        assert_eq!(
            schema["properties"]["numbers"]["propertyNames"],
            serde_json::json!({ "type": "string", "pattern": "^\\d+$" })
        );
        assert_eq!(
            schema["properties"]["signed"]["propertyNames"]["pattern"],
            serde_json::json!("^-?\\d+$")
        );
        assert_eq!(
            schema["properties"]["flags"]["propertyNames"]["enum"],
            serde_json::json!(["true", "false"])
        );
    }

    #[test]
    fn an_unknown_value_accepts_anything() {
        let schema = json_schema::<Keys>();

        assert_eq!(schema["properties"]["value"], serde_json::json!({}));
    }

    #[test]
    fn puts_definitions_where_references_point() {
        let schema = json_schema_with::<json_schema_shapes::ExtendingConfig>(JsonSchemaOptions {
            definitions_path: "#/$defs/".into(),
            ..Default::default()
        });

        assert!(schema["$defs"]["BaseConfig"].is_object());
        assert!(schema.get("definitions").is_none());
    }

    #[derive(Schematic)]
    pub struct UsesShared {
        a: recursion::Shared,
        b: recursion::Shared,
    }

    #[test]
    fn inlines_subschemas_when_requested() {
        let schema = json_schema_with::<UsesShared>(JsonSchemaOptions {
            inline_subschemas: true,
            ..Default::default()
        });

        assert_eq!(
            schema["properties"]["a"]["properties"]["value"]["type"],
            serde_json::json!("string")
        );
        assert!(schema.get("definitions").is_none());
    }

    #[derive(Schematic, Serialize)]
    #[serde(tag = "type")]
    pub enum Tagged {
        One(recursion::Shared),
        Two(recursion::Shared),
    }

    #[derive(Schematic)]
    pub struct HasTagged {
        tagged: Tagged,
        plain: recursion::Shared,
    }

    #[test]
    fn an_internally_tagged_variant_keeps_its_tag() {
        let schema = json_schema::<HasTagged>();
        let tagged = &schema["definitions"]["Tagged"]["anyOf"];

        assert_eq!(
            tagged[0]["properties"]["type"]["const"],
            serde_json::json!("One")
        );
        assert_eq!(
            tagged[1]["properties"]["type"]["const"],
            serde_json::json!("Two")
        );

        // The shared definition is left untagged
        assert!(
            schema["definitions"]["Shared"]["properties"]
                .get("type")
                .is_none()
        );
    }
}

mod defaults {
    use super::*;

    #[derive(Clone, ConfigEnum, Debug, Default, Deserialize, PartialEq, Serialize)]
    pub enum Mode {
        #[default]
        Foo,
        Bar,
    }

    #[derive(Config)]
    pub struct Defaults {
        maybe: Option<Mode>,
        #[setting(default = "Bar")]
        first: Mode,
        second: Mode,
    }

    #[test]
    fn an_option_has_no_default_of_its_type() {
        let schema = json_schema::<Defaults>();

        assert!(schema["properties"]["maybe"].get("default").is_none());
    }

    #[test]
    fn a_field_default_stays_on_the_field() {
        let schema = json_schema::<Defaults>();

        assert_eq!(
            schema["properties"]["first"]["default"],
            serde_json::json!("Bar")
        );
        assert_eq!(
            schema["properties"]["second"]["default"],
            serde_json::json!("Foo")
        );

        // The type itself keeps its own default
        let mut generator = SchemaGenerator::default();
        generator.add::<Defaults>();

        assert_eq!(
            generator.schemas["Mode"].get_default(),
            Some(&LiteralValue::String("Foo".into()))
        );
    }

    #[test]
    fn templates_render_a_field_default() {
        let yaml = render::<Defaults, _, _>(YamlTemplateRenderer::new(template_options()));

        assert!(yaml.contains("first: \"Bar\""), "{yaml}");
        assert!(yaml.contains("second: \"Foo\""), "{yaml}");
    }
}

mod skipped {
    use super::*;

    #[derive(Clone, Debug, Default, Deserialize, PartialEq, Schematic, Serialize)]
    pub enum Level {
        #[default]
        Auto,
        #[serde(skip_deserializing)]
        Internal,
        Manual,
    }

    #[derive(Config)]
    pub struct Skipped {
        #[serde(skip_deserializing)]
        computed: String,
        level: Level,
    }

    #[test]
    fn hides_what_serde_never_deserializes() {
        let schema = json_schema::<Skipped>();

        assert!(schema["properties"].get("computed").is_none());

        let ts = render::<Skipped, _, _>(TypeScriptRenderer::default());

        assert!(!ts.contains("Internal"), "{ts}");
        assert!(!ts.contains("computed"), "{ts}");
    }
}

mod typescript {
    use super::*;

    #[derive(Config)]
    #[serde(rename_all = "kebab-case")]
    pub struct Kebab {
        dash_name: String,
        #[setting(default = "it's")]
        quoted: String,
    }

    #[test]
    fn quotes_names_that_arent_identifiers() {
        let ts = render::<Kebab, _, _>(TypeScriptRenderer::default());

        assert!(ts.contains("'dash-name': string;"), "{ts}");
        assert!(ts.contains("@default 'it\\'s'"), "{ts}");
    }

    #[derive(Schematic, Serialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum Loud {
        VeryLoud,
    }

    #[test]
    fn quotes_enum_members() {
        let ts = render::<Loud, _, _>(TypeScriptRenderer::new(TypeScriptOptions {
            enum_format: EnumFormat::ValuedEnum,
            ..Default::default()
        }));

        assert!(ts.contains("'very-loud' = 'very-loud',"), "{ts}");
    }

    #[derive(Schematic)]
    pub struct Collides {
        own: bool,
        #[serde(flatten)]
        base: HashMap<String, String>,
    }

    #[test]
    fn names_the_base_apart_from_a_flattened_field() {
        let ts = render::<Collides, _, _>(TypeScriptRenderer::default());

        assert!(ts.contains("export type CollidesBase = "), "{ts}");
        assert!(ts.contains("export interface CollidesBase2 {"), "{ts}");
        assert!(ts.contains("CollidesBase2 & CollidesBase"), "{ts}");
    }
}

mod templates {
    use super::*;

    #[derive(Config)]
    pub struct Inner {
        count: usize,
        shared: String,
    }

    #[derive(Clone, Debug, Default, Deserialize, PartialEq, Schematic, Serialize)]
    #[serde(tag = "kind", content = "value")]
    pub enum Adjacent {
        #[default]
        Unit,
        Value(String),
    }

    #[derive(Config)]
    pub struct Document {
        #[setting(nested)]
        maybe: Option<Inner>,
        value: serde_json::Value,
        #[setting(default = "it's \"quoted\" \\ back")]
        escaped: String,
        adjacent: Adjacent,
        #[setting(skip)]
        zzz_hidden: String,
        #[setting(nested)]
        zzz_empty: Empty,
    }

    #[derive(Config)]
    pub struct Empty {
        #[setting(skip)]
        nothing: String,
    }

    #[test]
    fn renders_valid_toml() {
        let toml = render::<Document, _, _>(TomlTemplateRenderer::new(template_options()));
        let parsed: toml::Table = toml::from_str(&toml).unwrap_or_else(|error| {
            panic!("{error}\n{toml}");
        });

        assert_eq!(parsed["maybe"]["count"].as_integer(), Some(0));
        assert_eq!(parsed["escaped"].as_str(), Some("it's \"quoted\" \\ back"));
    }

    #[test]
    fn renders_valid_yaml() {
        let yaml = render::<Document, _, _>(YamlTemplateRenderer::new(template_options()));
        let parsed: serde_norway::Value = serde_norway::from_str(&yaml).unwrap_or_else(|error| {
            panic!("{error}\n{yaml}");
        });

        assert_eq!(parsed["adjacent"]["kind"].as_str(), Some("Unit"));
        assert_eq!(parsed["escaped"].as_str(), Some("it's \"quoted\" \\ back"));
    }

    #[test]
    fn renders_valid_json() {
        let json = render::<Document, _, _>(JsonTemplateRenderer::new(template_options()));
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap_or_else(|error| {
            panic!("{error}\n{json}");
        });

        assert_eq!(parsed["escaped"].as_str(), Some("it's \"quoted\" \\ back"));
        assert_eq!(parsed["zzz_empty"], serde_json::json!({}));
    }
}

mod externals {
    use super::*;

    #[derive(Schematic)]
    pub struct Externals {
        delta: chrono::Duration,
        local: chrono::NaiveDateTime,
        decimal: rust_decimal::Decimal,
        set: indexmap::IndexSet<String>,
    }

    #[test]
    fn match_what_serde_encodes() {
        let schema = json_schema::<Externals>();
        let properties = &schema["properties"];

        assert_eq!(properties["delta"]["type"], serde_json::json!("array"));
        assert_eq!(properties["delta"]["items"].as_array().unwrap().len(), 2);
        assert!(properties["local"].get("format").is_none());
        assert_eq!(
            properties["decimal"]["anyOf"][1]["type"],
            serde_json::json!("number")
        );
        assert_eq!(properties["set"]["uniqueItems"], serde_json::json!(true));

        // And check the encoding itself, so this fails if it ever changes
        assert_eq!(
            serde_json::to_value(chrono::Duration::seconds(90)).unwrap(),
            serde_json::json!([90, 0])
        );
    }
}
