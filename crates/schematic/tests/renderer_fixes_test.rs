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

mod schema_fidelity {
    use super::*;
    use serde_json::json;

    #[derive(Config)]
    struct Fields {
        plain: String,
        optional: Option<String>,
        #[setting(required)]
        required: Option<String>,
    }

    // Serde fills in an omitted `Option` with `None`
    #[test]
    fn an_option_may_be_omitted() {
        assert_eq!(
            json_schema::<Fields>()["required"],
            json!(["plain", "required"])
        );

        let output = render::<Fields, _, _>(TypeScriptRenderer::default());

        assert!(output.contains("optional?: string | null;"), "{output}");
    }

    // A required setting needs a value, which `null` isn't
    #[test]
    fn a_required_option_must_be_a_value() {
        let schema = json_schema::<Fields>();

        assert_eq!(
            schema["properties"]["required"],
            json!({ "type": "string" })
        );
        assert_eq!(
            schema["properties"]["optional"]["anyOf"][1],
            json!({ "type": "null" })
        );

        let output = render::<Fields, _, _>(TypeScriptRenderer::default());

        assert!(output.contains("required: string;"), "{output}");
    }

    #[derive(Config)]
    struct Closed {
        a: String,
    }

    #[derive(Config)]
    #[config(allow_unknown_fields)]
    struct Open {
        a: String,
    }

    #[derive(Deserialize, Schematic)]
    struct SerdeOpen {
        a: String,
    }

    #[derive(Deserialize, Schematic)]
    #[serde(deny_unknown_fields)]
    struct SerdeClosed {
        a: String,
    }

    #[derive(Deserialize, Schematic)]
    #[serde(deny_unknown_fields)]
    struct FlattensOpen {
        #[serde(flatten)]
        inner: SerdeOpen,
        b: String,
    }

    // Closed is what serde enforces, and a config's partial denies unknown
    // fields unless told otherwise
    #[test]
    fn closes_only_what_denies_unknown_fields() {
        assert_eq!(
            json_schema::<Closed>()["additionalProperties"],
            json!(false)
        );
        assert_eq!(json_schema::<Open>().get("additionalProperties"), None);
        assert_eq!(json_schema::<SerdeOpen>().get("additionalProperties"), None);
        assert_eq!(
            json_schema::<SerdeClosed>()["additionalProperties"],
            json!(false)
        );

        // Serde can't deny the keys a flattened struct accepts
        assert_eq!(
            json_schema::<FlattensOpen>().get("additionalProperties"),
            None
        );
        assert!(serde_json::from_str::<SerdeOpen>(r#"{ "a": "x", "z": 1 }"#).is_ok());
    }

    #[derive(Schematic)]
    struct Integers {
        a: u8,
        b: i16,
        c: u64,
        d: i64,
    }

    #[test]
    fn describes_an_integer_and_its_range() {
        let schema = json_schema::<Integers>();

        assert_eq!(
            schema["properties"]["a"],
            json!({ "type": "integer", "minimum": 0, "maximum": 255 })
        );
        assert_eq!(
            schema["properties"]["b"],
            json!({ "type": "integer", "minimum": -32768, "maximum": 32767 })
        );
        // A 64-bit bound can't be held exactly by a JSON number
        assert_eq!(
            schema["properties"]["c"],
            json!({ "type": "integer", "minimum": 0 })
        );
        assert_eq!(schema["properties"]["d"], json!({ "type": "integer" }));
    }

    struct Narrowed;

    impl Schematic for Narrowed {
        fn build_schema(mut schema: SchemaBuilder) -> Schema {
            schema.integer(IntegerType {
                kind: IntegerKind::U8,
                min: Some(10),
                max: Some(500),
                ..IntegerType::default()
            })
        }
    }

    #[derive(Schematic)]
    struct UsesNarrowed {
        value: Narrowed,
    }

    #[test]
    fn an_explicit_bound_narrows_but_cant_widen() {
        assert_eq!(
            json_schema::<UsesNarrowed>()["properties"]["value"],
            json!({ "type": "integer", "minimum": 10, "maximum": 255 })
        );
    }

    #[derive(Deserialize, Schematic)]
    #[serde(transparent)]
    struct Port {
        value: u16,
    }

    #[derive(Schematic)]
    struct UsesPort {
        port: Port,
    }

    #[test]
    fn a_transparent_struct_is_its_field() {
        let schema = json_schema::<UsesPort>();
        let port = match schema["properties"]["port"].get("$ref") {
            Some(reference) => {
                let name = reference.as_str().unwrap().rsplit('/').next().unwrap();

                schema["definitions"][name].clone()
            }
            None => schema["properties"]["port"].clone(),
        };

        assert_eq!(
            port,
            json!({ "type": "integer", "minimum": 0, "maximum": 65535 })
        );
        assert_eq!(serde_json::from_str::<Port>("8080").unwrap().value, 8080);
    }
}

// Serde writes a flattened struct's settings alongside the struct's own, so a
// template has to as well, or loading it fails on the unknown key
mod flattened_templates {
    use super::*;

    #[derive(Config)]
    pub struct Middle {
        #[setting(default = true)]
        middle_flag: bool,
        #[setting(nested)]
        middle_table: MiddleTable,
    }

    #[derive(Config)]
    pub struct MiddleTable {
        #[setting(default = 3)]
        retries: usize,
    }

    #[derive(Config)]
    pub struct Optional {
        #[setting(default = 7)]
        optional_count: usize,
    }

    #[derive(Config)]
    pub struct Flattening {
        #[setting(default = "own")]
        own: String,
        #[setting(flatten, nested)]
        middle: Middle,
        #[setting(flatten, nested)]
        optional: Option<Optional>,
    }

    // Loading fails on a key that isn't a setting, and the template's own
    // layer shows it set each flattened setting, rather than leaving it to
    // fall back to its default
    fn assert_loads(code: &str, file: &str) {
        let result = ConfigLoader::<Flattening>::new()
            .code(code, file)
            .unwrap()
            .load()
            .unwrap_or_else(|error| panic!("{}\n{code}", error.to_full_string()));

        let layer = &result.layers[1].partial;
        let middle = layer.middle.as_ref().expect(code);

        assert_eq!(layer.own.as_deref(), Some("own"), "{code}");
        assert_eq!(middle.middle_flag, Some(true), "{code}");
        assert_eq!(
            middle.middle_table.as_ref().and_then(|table| table.retries),
            Some(3),
            "{code}"
        );
        assert_eq!(
            layer
                .optional
                .as_ref()
                .and_then(|optional| optional.optional_count),
            Some(7),
            "{code}"
        );
    }

    #[test]
    fn yaml_writes_flattened_settings_alongside() {
        let yaml = render::<Flattening, _, _>(YamlTemplateRenderer::new(template_options()));
        let parsed: serde_norway::Value = serde_norway::from_str(&yaml).unwrap();

        assert_eq!(parsed["middle_flag"].as_bool(), Some(true), "{yaml}");
        assert_eq!(
            parsed["middle_table"]["retries"].as_u64(),
            Some(3),
            "{yaml}"
        );
        assert_loads(&yaml, "template.yml");
    }

    #[test]
    fn toml_writes_flattened_settings_alongside() {
        let toml = render::<Flattening, _, _>(TomlTemplateRenderer::new(template_options()));
        let parsed: toml::Table = toml::from_str(&toml).unwrap();

        // Previously a table of its own, which isn't a setting
        assert!(!parsed.contains_key("middle"), "{toml}");
        assert_eq!(parsed["middle_flag"].as_bool(), Some(true), "{toml}");
        assert_eq!(
            parsed["middle_table"]["retries"].as_integer(),
            Some(3),
            "{toml}"
        );
        assert_loads(&toml, "template.toml");
    }

    #[test]
    fn json_writes_flattened_settings_alongside() {
        let json = render::<Flattening, _, _>(JsonTemplateRenderer::new(template_options()));
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["optional_count"], serde_json::json!(7), "{json}");
        assert_loads(&json, "template.json");
    }

    #[test]
    fn jsonc_writes_flattened_settings_alongside() {
        let jsonc = render::<Flattening, _, _>(JsoncTemplateRenderer::new(TemplateOptions {
            comments: true,
            ..TemplateOptions::default()
        }));

        assert_loads(&jsonc, "template.jsonc");
    }

    #[test]
    fn pkl_writes_flattened_settings_alongside() {
        let pkl = render::<Flattening, _, _>(PklTemplateRenderer::new(template_options()));

        for line in ["middle_flag = true", "retries = 3", "optional_count = 7"] {
            assert!(pkl.contains(line), "{line}\n{pkl}");
        }

        assert!(!pkl.contains("middle {"), "{pkl}");
    }

    // Serde can't deny unknown fields through a flattened struct that flattens
    // another, so these allow them, as only then does it load
    #[derive(Config)]
    #[config(allow_unknown_fields)]
    pub struct Base {
        #[setting(default = "base")]
        base_name: String,
    }

    #[derive(Config)]
    #[config(allow_unknown_fields)]
    pub struct DeepMiddle {
        #[setting(default = true)]
        deep_flag: bool,
        #[setting(flatten, nested)]
        base: Base,
    }

    #[derive(Config)]
    #[config(allow_unknown_fields)]
    pub struct Deep {
        #[setting(default = "own")]
        own: String,
        #[setting(flatten, nested)]
        middle: DeepMiddle,
    }

    #[test]
    fn inlines_a_flattened_struct_within_another() {
        for (code, file) in [
            (
                render::<Deep, _, _>(YamlTemplateRenderer::new(template_options())),
                "template.yml",
            ),
            (
                render::<Deep, _, _>(TomlTemplateRenderer::new(template_options())),
                "template.toml",
            ),
        ] {
            let result = ConfigLoader::<Deep>::new()
                .code(code.as_str(), file)
                .unwrap()
                .load()
                .unwrap_or_else(|error| panic!("{}\n{code}", error.to_full_string()));
            let middle = result.layers[1].partial.middle.as_ref().expect(&code);

            assert_eq!(middle.deep_flag, Some(true), "{code}");
            assert_eq!(
                middle
                    .base
                    .as_ref()
                    .and_then(|base| base.base_name.as_deref()),
                Some("base"),
                "{code}"
            );
        }
    }
}
