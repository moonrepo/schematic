//! Exercises `#[derive(ConfigEnum)]` as emitted by `schematic_core`.
//!
//! `ConfigEnum` reuses the same `Container` as `Config` and `Schematic`,
//! diverging only in what it renders.
#![allow(dead_code)]

use schematic::{ConfigEnum as ConfigEnumTrait, ConfigError, SchemaBuilder, SchemaType};
use schematic_macros::ConfigEnum;
use std::str::FromStr;

mod basic {
    use super::*;

    #[derive(Clone, Debug, PartialEq, ConfigEnum)]
    enum Level {
        Info,
        Error,
        Off,
    }

    #[test]
    fn lists_every_variant() {
        assert_eq!(
            Level::variants(),
            vec![Level::Info, Level::Error, Level::Off]
        );
    }

    // No default casing: a name is used exactly as written
    #[test]
    fn parses_from_a_string() {
        assert_eq!(Level::from_str("Info").unwrap(), Level::Info);
        assert_eq!(Level::from_str("Off").unwrap(), Level::Off);
    }

    #[test]
    fn formats_back_into_a_string() {
        assert_eq!(Level::Info.to_string(), "Info");
        assert_eq!(Level::Error.to_string(), "Error");
    }

    #[test]
    fn round_trips() {
        for variant in Level::variants() {
            assert_eq!(Level::from_str(&variant.to_string()).unwrap(), variant);
        }
    }

    #[test]
    fn errors_on_an_unknown_value() {
        let error = Level::from_str("nope").unwrap_err();

        assert!(matches!(error, ConfigError::EnumUnknownVariant(_)));
    }

    #[test]
    fn converts_from_owned_and_borrowed() {
        assert_eq!(Level::try_from("Info".to_string()).unwrap(), Level::Info);
        assert_eq!(Level::try_from(&"Info".to_string()).unwrap(), Level::Info);
        assert_eq!(Level::try_from("Info").unwrap(), Level::Info);
    }
}

mod casing {
    use super::*;

    #[derive(Clone, Debug, PartialEq, ConfigEnum)]
    #[config(rename_all = "kebab-case")]
    enum Level {
        VeryHigh,
        Low,
    }

    #[test]
    fn applies_rename_all() {
        assert_eq!(Level::from_str("very-high").unwrap(), Level::VeryHigh);
        assert_eq!(Level::VeryHigh.to_string(), "very-high");
        assert!(Level::from_str("VeryHigh").is_err());
    }

    #[derive(Clone, Debug, PartialEq, ConfigEnum)]
    enum Renamed {
        #[variant(rename = "custom")]
        Explicit,
        #[variant(alias = "second", alias = "third")]
        Aliased,
    }

    #[test]
    fn honours_explicit_renames() {
        assert_eq!(Renamed::from_str("custom").unwrap(), Renamed::Explicit);
        assert_eq!(Renamed::Explicit.to_string(), "custom");
    }

    #[test]
    fn accepts_aliases_when_parsing() {
        assert_eq!(Renamed::from_str("Aliased").unwrap(), Renamed::Aliased);
        assert_eq!(Renamed::from_str("second").unwrap(), Renamed::Aliased);
        assert_eq!(Renamed::from_str("third").unwrap(), Renamed::Aliased);

        // Formatting always uses the canonical value
        assert_eq!(Renamed::Aliased.to_string(), "Aliased");
    }
}

mod before_parse {
    use super::*;

    #[derive(Clone, Debug, PartialEq, ConfigEnum)]
    #[config(before_parse = "lowercase", rename_all = "lowercase")]
    enum Lower {
        Info,
        Error,
    }

    #[test]
    fn lowercases_the_input() {
        assert_eq!(Lower::from_str("INFO").unwrap(), Lower::Info);
        assert_eq!(Lower::from_str("Info").unwrap(), Lower::Info);
        assert_eq!(Lower::from_str("info").unwrap(), Lower::Info);
    }

    #[derive(Clone, Debug, PartialEq, ConfigEnum)]
    #[config(before_parse = "UPPERCASE", rename_all = "UPPERCASE")]
    enum Upper {
        Info,
    }

    #[test]
    fn uppercases_the_input() {
        assert_eq!(Upper::from_str("info").unwrap(), Upper::Info);
        assert_eq!(Upper::Info.to_string(), "INFO");
    }

    // Every case serde's `rename_all` accepts is accepted here too, so
    // loosely cased input can be normalized before it is matched.
    #[derive(Clone, Debug, PartialEq, ConfigEnum)]
    #[config(before_parse = "kebab-case", rename_all = "kebab-case")]
    enum Kebab {
        VeryHigh,
    }

    #[test]
    fn supports_kebab_case() {
        for input in [
            "very_high",
            "VeryHigh",
            "VERY_HIGH",
            "very high",
            "veryHigh",
        ] {
            assert_eq!(
                Kebab::from_str(input).unwrap(),
                Kebab::VeryHigh,
                "`{input}` did not normalize"
            );
        }
    }

    #[derive(Clone, Debug, PartialEq, ConfigEnum)]
    #[config(before_parse = "camelCase")]
    enum Camel {
        #[variant(rename = "veryHigh")]
        VeryHigh,
    }

    #[test]
    fn supports_camel_case() {
        for input in ["very_high", "VeryHigh", "very-high"] {
            assert_eq!(Camel::from_str(input).unwrap(), Camel::VeryHigh);
        }
    }

    #[derive(Clone, Debug, PartialEq, ConfigEnum)]
    #[config(
        before_parse = "SCREAMING_SNAKE_CASE",
        rename_all = "SCREAMING_SNAKE_CASE"
    )]
    enum Screaming {
        VeryHigh,
    }

    #[test]
    fn supports_screaming_snake_case() {
        for input in ["very-high", "veryHigh", "very_high"] {
            assert_eq!(Screaming::from_str(input).unwrap(), Screaming::VeryHigh);
        }

        assert_eq!(Screaming::VeryHigh.to_string(), "VERY_HIGH");
    }

    #[derive(Clone, Debug, PartialEq, ConfigEnum)]
    #[config(before_parse = "PascalCase")]
    enum Pascal {
        VeryHigh,
    }

    #[test]
    fn supports_pascal_case() {
        for input in ["very-high", "very_high", "veryHigh"] {
            assert_eq!(Pascal::from_str(input).unwrap(), Pascal::VeryHigh);
        }
    }

    // Digit boundaries are preserved, matching the derive-time helper
    #[derive(Clone, Debug, PartialEq, ConfigEnum)]
    #[config(before_parse = "snake_case", rename_all = "snake_case")]
    enum Digits {
        Version2,
    }

    #[test]
    fn keeps_digits_attached() {
        assert_eq!(Digits::Version2.to_string(), "version2");
        assert_eq!(Digits::from_str("Version2").unwrap(), Digits::Version2);
    }
}

mod fallback {
    use super::*;

    #[derive(Clone, Debug, PartialEq, ConfigEnum)]
    #[config(rename_all = "kebab-case")]
    enum Value {
        Known,
        Other,
        #[variant(fallback)]
        Custom(String),
    }

    #[test]
    fn absorbs_unknown_values() {
        assert_eq!(Value::from_str("known").unwrap(), Value::Known);
        assert_eq!(
            Value::from_str("anything").unwrap(),
            Value::Custom("anything".into())
        );
    }

    // The fallback is matched last, so named variants still win
    #[test]
    fn does_not_shadow_named_variants() {
        assert_eq!(Value::from_str("other").unwrap(), Value::Other);
    }

    #[test]
    fn formats_from_the_inner_value() {
        assert_eq!(Value::Custom("abc".into()).to_string(), "abc");
        assert_eq!(Value::Known.to_string(), "known");
    }

    #[test]
    fn round_trips_through_the_fallback() {
        let value = Value::from_str("unmatched").unwrap();

        assert_eq!(Value::from_str(&value.to_string()).unwrap(), value);
    }

    #[test]
    fn variants_includes_a_default_fallback() {
        assert_eq!(
            Value::variants(),
            vec![Value::Known, Value::Other, Value::Custom(String::new())]
        );
    }
}

mod schema {
    use super::*;

    /// Some docs.
    #[derive(Clone, Debug, PartialEq, ConfigEnum, Default)]
    #[config(rename_all = "kebab-case")]
    enum Level {
        #[default]
        VeryHigh,
        Low,
    }

    #[test]
    fn builds_an_enumerable_schema() {
        let schema = SchemaBuilder::build_root::<Level>();

        assert_eq!(schema.name.as_deref(), Some("Level"));
        assert_eq!(schema.description.as_deref(), Some("Some docs."));

        let SchemaType::Enum(inner) = &schema.ty else {
            panic!("expected an enum, got {:?}", schema.ty);
        };

        assert_eq!(
            inner.variants.as_ref().unwrap().keys().collect::<Vec<_>>(),
            vec!["very-high", "low"]
        );
    }

    // The `#[default]` attribute from `#[derive(Default)]` marks the default
    #[test]
    fn records_the_default_variant() {
        let schema = SchemaBuilder::build_root::<Level>();

        assert_eq!(
            schema.get_default(),
            Some(&schematic::schema::LiteralValue::String("very-high".into()))
        );
    }

    #[derive(Clone, Debug, PartialEq, ConfigEnum)]
    enum WithFallback {
        Known,
        #[variant(fallback)]
        Other(String),
    }

    // A fallback accepts any string, so it widens the schema
    #[test]
    fn renders_a_fallback_as_a_string() {
        let schema = SchemaBuilder::build_root::<WithFallback>();
        let SchemaType::Enum(inner) = &schema.ty else {
            panic!("expected an enum");
        };
        let variants = inner.variants.as_ref().unwrap();

        assert!(matches!(
            variants["Known"].schema.ty,
            SchemaType::Literal(_)
        ));
        assert!(matches!(variants["Other"].schema.ty, SchemaType::String(_)));

        // Only the literal variants contribute values
        assert_eq!(inner.values.len(), 1);
    }
}

// `derive_enum!` injects `#[serde(rename_all = "kebab-case")]`, which core
// reads like any other container attribute. That keeps the documented
// migration path producing the same values as the production derive, even
// though core itself applies no default casing.
mod derive_enum_helper {
    use super::*;
    use schematic::derive_enum;

    derive_enum!(
        #[derive(ConfigEnum, Default)]
        pub enum Level {
            #[default]
            VeryHigh,
            Low,
        }
    );

    #[test]
    fn picks_up_the_injected_rename_all() {
        assert_eq!(Level::from_str("very-high").unwrap(), Level::VeryHigh);
        assert_eq!(Level::VeryHigh.to_string(), "very-high");
    }

    #[test]
    fn serializes_consistently_with_the_schema() {
        // serde uses the injected attribute, the schema uses the same names
        assert_eq!(
            serde_json::to_string(&Level::VeryHigh).unwrap(),
            "\"very-high\""
        );

        let schema = SchemaBuilder::build_root::<Level>();
        let SchemaType::Enum(inner) = &schema.ty else {
            panic!("expected an enum");
        };

        assert_eq!(
            inner.variants.as_ref().unwrap().keys().collect::<Vec<_>>(),
            vec!["very-high", "low"]
        );
    }
}

// A generic `ConfigEnum` only makes sense with a fallback, since unit
// variants carry no data. The type argument has to reach every impl, and
// `Display` writes the fallback through its own `Display` rather than
// requiring it to be a `&str`.
mod generics {
    use super::*;

    #[derive(Clone, Debug, PartialEq, ConfigEnum)]
    #[config(rename_all = "kebab-case")]
    enum Value<T>
    where
        T: Clone + Default + std::fmt::Display + for<'a> TryFrom<&'a str> + schematic::Schematic,
    {
        Known,
        Other,
        #[variant(fallback)]
        Custom(T),
    }

    #[test]
    fn parses_named_variants() {
        assert_eq!(Value::<String>::from_str("known").unwrap(), Value::Known);
        assert_eq!(Value::<String>::from_str("other").unwrap(), Value::Other);
    }

    #[test]
    fn parses_through_a_generic_fallback() {
        assert_eq!(
            Value::<String>::from_str("anything").unwrap(),
            Value::Custom("anything".into())
        );
    }

    #[test]
    fn formats_a_generic_fallback() {
        assert_eq!(Value::<String>::Custom("abc".into()).to_string(), "abc");
        assert_eq!(Value::<String>::Known.to_string(), "known");
    }

    #[test]
    fn lists_variants() {
        assert_eq!(
            Value::<String>::variants(),
            vec![Value::Known, Value::Other, Value::Custom(String::new())]
        );
    }

    // The schema name carries the type argument, so instantiations don't
    // collide in a generator
    #[test]
    fn names_schemas_per_instantiation() {
        assert_eq!(
            SchemaBuilder::build_root::<Value<String>>().name.as_deref(),
            Some("ValueString")
        );
    }

    #[test]
    fn builds_a_generic_schema() {
        let schema = SchemaBuilder::build_root::<Value<String>>();
        let SchemaType::Enum(inner) = &schema.ty else {
            panic!("expected an enum, got {:?}", schema.ty);
        };
        let variants = inner.variants.as_ref().unwrap();

        assert_eq!(
            variants.keys().collect::<Vec<_>>(),
            vec!["known", "other", "custom"]
        );
        assert!(matches!(
            variants["custom"].schema.ty,
            SchemaType::String(_)
        ));
    }
}
