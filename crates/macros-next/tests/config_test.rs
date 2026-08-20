//! Exercises `#[derive(Config)]` as emitted by `schematic_core`.
//!
//! Core's own suite only snapshots pretty printed tokens, so this is the
//! first place the generated code is actually compiled and run.
#![allow(dead_code)]

// `schematic` re-exports the production derives, so the runtime pieces are
// imported by name (the trait aliased out of the way) and the derives come
// from this crate.
use schematic::{Config as ConfigTrait, ConfigLoader, PartialConfig};
use schematic_macros_next::Config;
use serde::Serialize;
use std::collections::HashMap;

mod named_struct {
    use super::*;

    #[derive(Debug, Config)]
    pub struct Basic {
        boolean: bool,
        string: String,
        number: usize,
        vector: Vec<String>,
    }

    #[test]
    fn uses_native_defaults() {
        let result = ConfigLoader::<Basic>::new().load().unwrap();

        assert!(!result.config.boolean);
        assert_eq!(result.config.string, "");
        assert_eq!(result.config.number, 0);
        assert_eq!(result.config.vector, Vec::<String>::new());
    }

    #[derive(Debug, Config)]
    pub struct Defaults {
        #[setting(default = true)]
        boolean: bool,
        #[setting(default = "foo")]
        string: String,
        #[setting(default = 123)]
        number: usize,
        #[setting(default = 1.5)]
        float: f32,
    }

    #[test]
    fn uses_literal_defaults() {
        let result = ConfigLoader::<Defaults>::new().load().unwrap();

        assert!(result.config.boolean);
        assert_eq!(result.config.string, "foo");
        assert_eq!(result.config.number, 123);
        assert_eq!(result.config.float, 1.5);
    }

    #[test]
    fn implements_default_via_from_partial() {
        let config = Defaults::default();

        assert!(config.boolean);
        assert_eq!(config.string, "foo");
    }

    #[derive(Debug, Config)]
    pub struct Optionals {
        required: String,
        optional: Option<String>,
        #[setting(default = 10)]
        optional_with_default: Option<usize>,
    }

    // The outer `Option` doubles as the partial's, so it must not be
    // wrapped a second time.
    #[test]
    fn does_not_double_wrap_options() {
        let partial = PartialOptionals {
            required: Some("a".into()),
            optional: Some("b".into()),
            optional_with_default: None,
        };

        let config = Optionals::from_partial(partial);

        assert_eq!(config.required, "a");
        assert_eq!(config.optional, Some("b".into()));
    }

    #[test]
    fn applies_defaults_to_optionals() {
        let result = ConfigLoader::<Optionals>::new().load().unwrap();

        assert_eq!(result.config.optional, None);
        assert_eq!(result.config.optional_with_default, Some(10));
    }
}

mod merging {
    use super::*;

    #[derive(Debug, Config)]
    pub struct Merged {
        string: String,
        list: Vec<usize>,
    }

    #[test]
    fn later_layers_win() {
        let mut base = PartialMerged {
            string: Some("first".into()),
            list: Some(vec![1]),
        };

        base.merge(
            &(),
            PartialMerged {
                string: Some("second".into()),
                list: Some(vec![2]),
            },
        )
        .unwrap();

        assert_eq!(base.string, Some("second".into()));
        assert_eq!(base.list, Some(vec![2]));
    }

    #[test]
    fn none_does_not_clobber() {
        let mut base = PartialMerged {
            string: Some("first".into()),
            list: None,
        };

        base.merge(
            &(),
            PartialMerged {
                string: None,
                list: Some(vec![9]),
            },
        )
        .unwrap();

        assert_eq!(base.string, Some("first".into()));
        assert_eq!(base.list, Some(vec![9]));
    }
}

mod nested {
    use super::*;

    #[derive(Debug, Config)]
    pub struct Child {
        #[setting(default = "child")]
        name: String,
        value: usize,
    }

    #[derive(Debug, Config)]
    pub struct Parent {
        #[setting(nested)]
        one: Child,
        #[setting(nested)]
        many: Vec<Child>,
        #[setting(nested)]
        keyed: HashMap<String, Child>,
        #[setting(nested)]
        maybe: Option<Child>,
    }

    #[test]
    fn finalizes_nested_defaults() {
        let result = ConfigLoader::<Parent>::new().load().unwrap();

        assert_eq!(result.config.one.name, "child");
        assert!(result.config.many.is_empty());
        assert!(result.config.keyed.is_empty());
        assert!(result.config.maybe.is_none());
    }

    // A bare nested config merges recursively, while nested collections
    // replace by default.
    #[test]
    fn merges_bare_nested_but_replaces_collections() {
        let mut base = PartialParent {
            one: Some(PartialChild {
                name: Some("kept".into()),
                value: None,
            }),
            many: Some(vec![PartialChild {
                name: Some("old".into()),
                value: None,
            }]),
            ..Default::default()
        };

        base.merge(
            &(),
            PartialParent {
                one: Some(PartialChild {
                    name: None,
                    value: Some(7),
                }),
                many: Some(vec![PartialChild {
                    name: Some("new".into()),
                    value: None,
                }]),
                ..Default::default()
            },
        )
        .unwrap();

        let one = base.one.unwrap();
        assert_eq!(one.name, Some("kept".into()));
        assert_eq!(one.value, Some(7));

        let many = base.many.unwrap();
        assert_eq!(many.len(), 1);
        assert_eq!(many[0].name, Some("new".into()));
    }
}

mod wrappers {
    use super::*;
    use std::sync::Arc;

    #[derive(Debug, Config)]
    pub struct Wrapped {
        #[allow(clippy::box_collection)]
        boxed: Box<String>,
        shared: Arc<usize>,
        // Kept in the partial, since the inner type is unsized
        unsized_box: Box<str>,
    }

    // Sized wrappers are stripped from the partial and rebuilt in
    // `from_partial`, which is why partials don't need serde's `rc` feature.
    #[test]
    fn strips_and_rebuilds_wrappers() {
        let partial = PartialWrapped {
            boxed: Some("a".into()),
            shared: Some(3),
            unsized_box: Some("c".into()),
        };

        let config = Wrapped::from_partial(partial);

        assert_eq!(*config.boxed, "a".to_string());
        assert_eq!(*config.shared, 3);
        assert_eq!(&*config.unsized_box, "c");
    }
}

mod unnamed_struct {
    use super::*;

    #[derive(Debug, Config)]
    pub struct Single(String);

    #[derive(Debug, Config)]
    pub struct Pair(String, usize);

    #[test]
    fn supports_tuple_structs() {
        let single = Single::from_partial(PartialSingle(Some("a".into())));
        assert_eq!(single.0, "a");

        let pair = Pair::from_partial(PartialPair(Some("a".into()), Some(2)));
        assert_eq!(pair.0, "a");
        assert_eq!(pair.1, 2);
    }
}

mod enums {
    use super::*;

    #[derive(Debug, Config)]
    pub enum Value {
        Text(String),
        Number(usize),
        #[setting(default)]
        Flag(bool),
    }

    #[test]
    fn defaults_to_the_marked_variant() {
        assert!(matches!(PartialValue::default(), PartialValue::Flag(false)));

        let config = Value::from_partial(PartialValue::Text("a".into()));
        assert!(matches!(config, Value::Text(_)));
    }

    // Variant payloads get no synthetic `Option`
    #[test]
    fn payloads_are_not_optional() {
        let partial = PartialValue::Number(5);

        assert!(matches!(Value::from_partial(partial), Value::Number(5)));
    }

    #[derive(Debug, Config)]
    pub enum Level {
        Low,
        #[setting(default)]
        Medium,
        High,
    }

    #[test]
    fn supports_unit_enums() {
        assert!(matches!(PartialLevel::default(), PartialLevel::Medium));
        assert!(matches!(
            Level::from_partial(PartialLevel::High),
            Level::High
        ));
    }

    // Unit enums stay externally tagged, otherwise they only deserialize
    // from `null`.
    //
    // Names are used exactly as written: there is no default casing, so a
    // variant only changes shape when `rename_all` asks it to.
    #[test]
    fn unit_enums_deserialize_from_a_string() {
        let partial: PartialLevel = serde_json::from_str("\"High\"").unwrap();

        assert!(matches!(partial, PartialLevel::High));

        assert!(serde_json::from_str::<PartialLevel>("\"high\"").is_err());
    }
}

mod serde_attrs {
    use super::*;

    #[derive(Debug, Config, Serialize)]
    pub struct Renamed {
        #[setting(rename = "otherName")]
        name: String,
        #[serde(alias = "second", alias = "third")]
        first: String,
    }

    #[test]
    fn honours_renames_and_aliases() {
        let partial: PartialRenamed =
            serde_json::from_str(r#"{"otherName": "a", "second": "b"}"#).unwrap();

        assert_eq!(partial.name, Some("a".into()));
        assert_eq!(partial.first, Some("b".into()));
    }

    #[test]
    fn rejects_unknown_fields() {
        assert!(serde_json::from_str::<PartialRenamed>(r#"{"nope": 1}"#).is_err());
    }
}

// `rename_all` has to reach the derive-time name, not only the partial's
// serde attribute, or the schema and `settings()` would describe a key that
// serde does not accept. There is no default case: without `rename_all` the
// Rust name is used exactly as written.
mod casing {
    use super::*;
    use schematic::{SchemaBuilder, SchemaType};

    fn field_names<T: schematic::Schematic>() -> Vec<String> {
        let schema = SchemaBuilder::build_root::<T>();
        let SchemaType::Struct(inner) = &schema.ty else {
            panic!("expected a struct");
        };

        inner.fields.keys().cloned().collect()
    }

    #[derive(Debug, Config)]
    pub struct Untouched {
        some_field_name: String,
    }

    #[test]
    fn leaves_names_alone_without_rename_all() {
        assert!(serde_json::from_str::<PartialUntouched>(r#"{"some_field_name": "a"}"#).is_ok());
        assert!(serde_json::from_str::<PartialUntouched>(r#"{"someFieldName": "a"}"#).is_err());

        assert_eq!(field_names::<Untouched>(), vec!["some_field_name"]);
        assert_eq!(
            Untouched::settings().keys().collect::<Vec<_>>(),
            vec!["some_field_name"]
        );
    }

    #[derive(Debug, Config)]
    #[config(rename_all = "camelCase")]
    pub struct Camel {
        some_field_name: String,
        another_one: usize,
        #[setting(rename = "kept_as_is")]
        explicitly_renamed: bool,
    }

    #[test]
    fn applies_rename_all_to_serde() {
        let partial: PartialCamel =
            serde_json::from_str(r#"{"someFieldName": "a", "anotherOne": 1}"#).unwrap();

        assert_eq!(partial.some_field_name, Some("a".into()));
        assert_eq!(partial.another_one, Some(1));
    }

    #[test]
    fn applies_rename_all_to_derive_time_names() {
        assert_eq!(
            field_names::<Camel>(),
            vec!["someFieldName", "anotherOne", "kept_as_is"]
        );
        // `settings()` is a `BTreeMap`, so it reports alphabetically
        assert_eq!(
            Camel::settings().keys().collect::<Vec<_>>(),
            vec!["anotherOne", "kept_as_is", "someFieldName"]
        );
    }

    // An explicit rename is used exactly as written, never re-cased
    #[test]
    fn does_not_case_an_explicit_rename() {
        assert!(serde_json::from_str::<PartialCamel>(r#"{"kept_as_is": true}"#).is_ok());
        assert!(serde_json::from_str::<PartialCamel>(r#"{"keptAsIs": true}"#).is_err());
    }

    #[derive(Debug, Config, Serialize)]
    #[serde(rename_all = "kebab-case")]
    pub struct ViaSerde {
        some_field_name: String,
    }

    #[test]
    fn honours_rename_all_from_the_serde_attribute() {
        assert!(serde_json::from_str::<PartialViaSerde>(r#"{"some-field-name": "a"}"#).is_ok());
        assert_eq!(field_names::<ViaSerde>(), vec!["some-field-name"]);
    }

    #[derive(Debug, Config)]
    #[config(rename_all = "kebab-case")]
    pub enum Variants {
        SomeVariant(String),
        #[setting(default)]
        AnotherOne(bool),
    }

    #[test]
    fn applies_rename_all_to_variants() {
        let partial: PartialVariants = serde_json::from_str(r#"{"some-variant": "a"}"#).unwrap();

        assert!(matches!(partial, PartialVariants::SomeVariant(_)));
        assert!(serde_json::from_str::<PartialVariants>(r#"{"SomeVariant": "a"}"#).is_err());

        assert_eq!(
            Variants::settings().keys().collect::<Vec<_>>(),
            vec!["another-one", "some-variant"]
        );
    }

    #[derive(Debug, Config)]
    #[config(rename_all = "SCREAMING_SNAKE_CASE")]
    pub struct Screaming {
        some_field_name: String,
    }

    #[test]
    fn supports_every_serde_case() {
        assert!(serde_json::from_str::<PartialScreaming>(r#"{"SOME_FIELD_NAME": "a"}"#).is_ok());
        assert_eq!(field_names::<Screaming>(), vec!["SOME_FIELD_NAME"]);
    }
}

// A generic config needs its type arguments carried into the partial type
// and every impl. `PartialConfig` requires `DeserializeOwned`, so the derive
// states that bound outright rather than letting serde infer a conflicting
// `Deserialize<'de>` one.
mod generics {
    use super::*;
    use schematic::{Schema, SchemaBuilder, SchemaType};
    use serde::de::DeserializeOwned;

    pub trait Setting:
        Clone
        + std::fmt::Debug
        + Default
        + PartialEq
        + Serialize
        + DeserializeOwned
        + schematic::Schematic
    {
    }

    impl<T> Setting for T where
        T: Clone
            + std::fmt::Debug
            + Default
            + PartialEq
            + Serialize
            + DeserializeOwned
            + schematic::Schematic
    {
    }

    #[derive(Debug, Config)]
    pub struct Wrapper<T: Setting> {
        inner: T,
        label: String,
    }

    #[test]
    fn builds_a_generic_partial() {
        let partial: PartialWrapper<usize> =
            serde_json::from_str(r#"{"inner": 5, "label": "a"}"#).unwrap();

        assert_eq!(partial.inner, Some(5));
        assert_eq!(partial.label, Some("a".into()));
    }

    #[test]
    fn constructs_the_full_type() {
        let config = Wrapper::<usize>::from_partial(PartialWrapper {
            inner: Some(5),
            label: Some("a".into()),
        });

        assert_eq!(config.inner, 5);
        assert_eq!(config.label, "a");
    }

    #[test]
    fn loads_through_the_loader() {
        let result = ConfigLoader::<Wrapper<usize>>::new().load().unwrap();

        assert_eq!(result.config.inner, 0);
        assert_eq!(result.config.label, "");
    }

    #[test]
    fn merges_a_generic_partial() {
        let mut base = PartialWrapper::<usize> {
            inner: Some(1),
            label: None,
        };

        base.merge(
            &(),
            PartialWrapper {
                inner: Some(2),
                label: Some("b".into()),
            },
        )
        .unwrap();

        assert_eq!(base.inner, Some(2));
        assert_eq!(base.label, Some("b".into()));
    }

    #[test]
    fn implements_default() {
        assert_eq!(Wrapper::<usize>::default().inner, 0);
    }

    // Each instantiation resolves to a distinct schema name, for both the
    // full type and its partial
    #[test]
    fn names_schemas_per_instantiation() {
        assert_eq!(
            SchemaBuilder::build_root::<Wrapper<usize>>()
                .name
                .as_deref(),
            Some("WrapperUsize")
        );
        assert_eq!(
            SchemaBuilder::build_root::<Wrapper<String>>()
                .name
                .as_deref(),
            Some("WrapperString")
        );
        assert_eq!(
            SchemaBuilder::build_root::<PartialWrapper<usize>>()
                .name
                .as_deref(),
            Some("PartialWrapperUsize")
        );
    }

    #[test]
    fn builds_a_generic_schema() {
        let schema: Schema = SchemaBuilder::build_root::<Wrapper<bool>>();
        let SchemaType::Struct(inner) = &schema.ty else {
            panic!("expected a struct");
        };

        assert_eq!(
            inner.fields["inner"].schema.ty,
            SchemaType::Boolean(Box::default())
        );
    }

    #[derive(Debug, Config)]
    pub struct Pair<T: Setting, U: Setting>(T, U);

    #[test]
    fn supports_multiple_parameters_on_a_tuple_struct() {
        let config = Pair::<usize, String>::from_partial(PartialPair(Some(1), Some("a".into())));

        assert_eq!(config.0, 1);
        assert_eq!(config.1, "a");
    }

    #[derive(Debug, Config)]
    pub enum Either<T: Setting> {
        Value(T),
        #[setting(default)]
        Nothing,
    }

    #[test]
    fn supports_generic_enums() {
        assert!(matches!(
            PartialEither::<usize>::default(),
            PartialEither::Nothing
        ));

        let config = Either::<usize>::from_partial(PartialEither::Value(3));

        assert!(matches!(config, Either::Value(3)));
    }

    #[derive(Debug, Config)]
    pub struct Bounded<T>
    where
        T: Setting,
    {
        value: T,
    }

    #[test]
    fn supports_where_clauses() {
        let config = Bounded::<bool>::from_partial(PartialBounded { value: Some(true) });

        assert!(config.value);
    }
}
