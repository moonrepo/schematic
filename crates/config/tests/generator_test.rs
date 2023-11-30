#![allow(dead_code, deprecated)]

use schematic::schema::SchemaGenerator;
use schematic::*;
use starbase_sandbox::{assert_snapshot, create_empty_sandbox};
use std::collections::{HashMap, HashSet};
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
    opt: Option<String>,
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
    yaml_value: serde_yaml::Value,
}

fn create_generator() -> SchemaGenerator {
    let mut generator = SchemaGenerator::default();
    generator.add::<GenConfig>();
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
            exclude_references: HashSet::from_iter(["BasicEnum".into(), "AnotherType".into()]),
            ..TypeScriptOptions::default()
        }));
    }

    #[test]
    fn external_types() {
        assert_snapshot!(generate(TypeScriptOptions {
            external_types: HashMap::from_iter([(
                "./externals".into(),
                HashSet::from_iter(["BasicEnum".into(), "AnotherType".into()])
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
