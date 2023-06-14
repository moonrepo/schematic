#![allow(dead_code, deprecated)]

use schematic::schema::SchemaGenerator;
use schematic::*;
use starbase_sandbox::{assert_snapshot, create_empty_sandbox};
use std::collections::{HashMap, HashSet};
use std::fs;

derive_enum!(
    #[derive(ConfigEnum, Default)]
    pub enum BasicEnum {
        #[default]
        Foo,
        Bar,
        Baz,
    }
);

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
    vector: Vec<String>,
    map: HashMap<String, u64>,
    enums: BasicEnum,
    #[setting(nested)]
    nested: AnotherConfig,
}

fn create_generator() -> SchemaGenerator {
    let mut generator = SchemaGenerator::default();
    generator.add::<GenConfig>();
    generator
}

#[cfg(feature = "typescript")]
mod typescript {
    use super::*;
    use schematic::renderers::typescript::*;

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
