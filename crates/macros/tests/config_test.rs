//! Exercises `#[derive(Config)]` as emitted by `schematic_core`.
//!
//! Core's own suite only snapshots pretty printed tokens, so this is the
//! first place the generated code is actually compiled and run.
#![allow(dead_code)]

// `schematic` re-exports the production derives, so the runtime pieces are
// imported by name (the trait aliased out of the way) and the derives come
// from this crate.
use schematic::{Config as ConfigTrait, ConfigLoader, PartialConfig};
use schematic_macros::Config;
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

// https://github.com/moonrepo/schematic/issues/173
mod default_values {
    use super::*;
    use schematic::DefaultValueResult;
    use serde::Deserialize;

    #[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, schematic::Schematic)]
    pub enum LevelFilter {
        #[default]
        Off,
        Debug,
    }

    #[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, schematic::Schematic)]
    pub struct Point {
        pub x: usize,
        pub y: usize,
    }

    impl Point {
        pub const ORIGIN: Point = Point { x: 0, y: 0 };
    }

    pub mod limits {
        pub const MAX: usize = 99;
    }

    fn pick_level<C>(_: &C) -> DefaultValueResult<LevelFilter> {
        Ok(Some(LevelFilter::Debug))
    }

    #[derive(Debug, Config)]
    pub struct Values {
        #[setting(default = LevelFilter::Debug)]
        variant: LevelFilter,
        #[setting(default = Point::ORIGIN)]
        assoc_const: Point,
        #[setting(default = limits::MAX)]
        path_const: usize,
        #[setting(default = Point { x: 1, y: 2 })]
        literal: Point,
        #[setting(default = Point { x: 3, ..Point::ORIGIN })]
        literal_with_rest: Point,
    }

    #[test]
    fn uses_a_path_that_names_a_value() {
        let config = Values::default();

        assert_eq!(config.variant, LevelFilter::Debug);
        assert_eq!(config.assoc_const, Point { x: 0, y: 0 });
        assert_eq!(config.path_const, 99);
    }

    #[test]
    fn uses_a_struct_literal() {
        let config = Values::default();

        assert_eq!(config.literal, Point { x: 1, y: 2 });
        assert_eq!(config.literal_with_rest, Point { x: 3, y: 0 });
    }

    #[derive(Debug, Config)]
    pub struct Handlers {
        #[setting(default = pick_level)]
        via_fn: LevelFilter,
    }

    // A `snake_case` path is still called with the context, which is what
    // tells the two apart
    #[test]
    fn still_calls_a_handler_function() {
        assert_eq!(Handlers::default().via_fn, LevelFilter::Debug);
    }

    #[derive(Debug, Config)]
    pub struct Layered {
        #[setting(default = LevelFilter::Debug)]
        optional: Option<LevelFilter>,
        #[setting(default = LevelFilter::Debug)]
        boxed: Box<LevelFilter>,
        #[setting(default = vec![LevelFilter::Debug])]
        list: Vec<LevelFilter>,
    }

    #[test]
    fn wraps_a_value_in_its_layers() {
        let config = Layered::default();

        assert_eq!(config.optional, Some(LevelFilter::Debug));
        assert_eq!(*config.boxed, LevelFilter::Debug);
        assert_eq!(config.list, vec![LevelFilter::Debug]);
    }

    // A default is the lowest layer, so a configured value still wins
    #[test]
    fn a_source_still_overrides_it() {
        let result = ConfigLoader::<Values>::new()
            .code(r#"{"variant": "Off"}"#, "code.json")
            .unwrap()
            .load()
            .unwrap();

        assert_eq!(result.config.variant, LevelFilter::Off);
        assert_eq!(result.config.literal, Point { x: 1, y: 2 });
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
    use std::collections::BTreeMap;

    // A `serde` predicate describes the full type, where the field is a `T`.
    // The partial holds an `Option<T>`, so forwarding it there cannot compile.
    #[derive(Debug, Config, Serialize)]
    pub struct SkipSerializing {
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        map: BTreeMap<String, String>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        list: Vec<String>,
    }

    #[test]
    fn a_serde_predicate_is_left_to_the_full_type() {
        // The partial falls back to its own, which is what its shape allows
        let partial = PartialSkipSerializing::default();

        assert_eq!(serde_json::to_string(&partial).unwrap(), "{}");
    }

    #[test]
    fn a_serde_predicate_still_applies_to_the_full_type() {
        let config = SkipSerializing::default();

        assert_eq!(serde_json::to_string(&config).unwrap(), "{}");
    }

    fn is_empty_option(value: &Option<String>) -> bool {
        value.as_ref().is_none_or(|inner| inner.is_empty())
    }

    // The `setting` form does reach the partial, so it is written against the
    // partial's `Option`
    #[derive(Debug, Config, Serialize)]
    pub struct SkipSerializingSetting {
        #[setting(skip_serializing_if = "is_empty_option")]
        name: String,
    }

    #[test]
    fn a_setting_predicate_applies_to_the_partial() {
        let mut partial = PartialSkipSerializingSetting::default();

        assert_eq!(serde_json::to_string(&partial).unwrap(), "{}");

        partial.name = Some(String::new());
        assert_eq!(serde_json::to_string(&partial).unwrap(), "{}");

        partial.name = Some("abc".into());
        assert_eq!(
            serde_json::to_string(&partial).unwrap(),
            r#"{"name":"abc"}"#
        );
    }

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
        pub inner: T,
        pub label: String,
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

// A required setting has no default, so it must be provided by a layer,
// either a source or the environment, or the final validation fails.
mod required {
    use super::*;
    use schematic::{SchemaBuilder, SchemaType};

    #[derive(Debug, Config)]
    pub struct Required {
        #[setting(required)]
        name: String,
        #[setting(required)]
        list: Vec<String>,
        #[setting(required)]
        maybe: Option<String>,
        other: String,
    }

    fn load(code: &str) -> Result<Required, String> {
        ConfigLoader::<Required>::new()
            .code(code, "code.json")
            .unwrap()
            .load()
            .map(|result| result.config)
            .map_err(|error| error.to_full_string())
    }

    #[test]
    fn errors_when_missing() {
        assert_eq!(
            load("{}").unwrap_err(),
            "Failed to validate code.json. \n  name: this setting is required\n  list: this setting is required\n  maybe: this setting is required"
        );
    }

    #[test]
    fn errors_when_null() {
        assert_eq!(
            load(r#"{ "name": null, "list": [], "maybe": "a" }"#).unwrap_err(),
            "Failed to validate code.json. \n  name: this setting is required"
        );
    }

    // Present is enough, even when empty, as with serde
    #[test]
    fn loads_when_provided() {
        let config = load(r#"{ "name": "", "list": [], "maybe": "a" }"#).unwrap();

        assert_eq!(config.name, "");
        assert!(config.list.is_empty());
        assert_eq!(config.maybe.as_deref(), Some("a"));
        assert_eq!(config.other, "");
    }

    // Each layer is validated on its own, but only the merged result must
    // provide the setting
    #[test]
    fn may_be_provided_by_any_layer() {
        let result = ConfigLoader::<Required>::new()
            .code(r#"{ "name": "a" }"#, "one.json")
            .unwrap()
            .code(r#"{ "list": ["b"], "maybe": "c" }"#, "two.json")
            .unwrap()
            .load()
            .unwrap();

        assert_eq!(result.config.name, "a");
        assert_eq!(result.config.list, vec!["b".to_string()]);
    }

    #[test]
    fn still_implements_default() {
        let config = Required::default();

        assert_eq!(config.name, "");
        assert!(config.list.is_empty());
        assert_eq!(config.maybe, None);
    }

    #[derive(Debug, Config)]
    pub struct FromEnv {
        #[setting(required, env = "SCHEMATIC_MACROS_REQUIRED_TOKEN")]
        token: String,
    }

    #[test]
    fn may_be_provided_by_the_environment() {
        assert!(ConfigLoader::<FromEnv>::new().load().is_err());

        unsafe { std::env::set_var("SCHEMATIC_MACROS_REQUIRED_TOKEN", "secret") };

        let result = ConfigLoader::<FromEnv>::new().load();

        unsafe { std::env::remove_var("SCHEMATIC_MACROS_REQUIRED_TOKEN") };

        assert_eq!(result.unwrap().config.token, "secret");
    }

    #[derive(Debug, Config)]
    pub struct Grandchild {
        #[setting(required)]
        name: String,
    }

    #[derive(Debug, Config)]
    pub struct Child {
        #[setting(required)]
        name: String,
        #[setting(nested)]
        inner: Grandchild,
    }

    #[derive(Debug, Config)]
    pub struct Parent {
        #[setting(nested)]
        one: Child,
        #[setting(nested)]
        boxed: Box<Child>,
        #[setting(nested)]
        maybe: Option<Child>,
        #[setting(nested)]
        many: Vec<Child>,
    }

    fn load_parent(code: &str) -> Result<Parent, String> {
        ConfigLoader::<Parent>::new()
            .code(code, "code.json")
            .unwrap()
            .load()
            .map(|result| result.config)
            .map_err(|error| error.to_full_string())
    }

    // A bare nested config exists in the full type whether or not a layer
    // provided it, so its settings are required either way, while an
    // optional or collected config is only checked when present
    #[test]
    fn checks_a_bare_nested_config_no_layer_provided() {
        assert_eq!(
            load_parent("{}").unwrap_err(),
            "Failed to validate code.json. \n  one.name: this setting is required\n  one.inner.name: this setting is required\n  boxed.name: this setting is required\n  boxed.inner.name: this setting is required"
        );
    }

    #[test]
    fn checks_nested_configs_that_are_present() {
        assert_eq!(
            load_parent(
                r#"{
                    "one": { "name": "a", "inner": { "name": "b" } },
                    "boxed": { "name": "c", "inner": { "name": "d" } },
                    "maybe": { "inner": { "name": "e" } },
                    "many": [{ "name": "f", "inner": { "name": "g" } }, { "name": "h", "inner": {} }]
                }"#
            )
            .unwrap_err(),
            "Failed to validate code.json. \n  maybe.name: this setting is required\n  many[1].inner.name: this setting is required"
        );
    }

    #[test]
    fn loads_nested_configs_when_provided() {
        let config = load_parent(
            r#"{
                "one": { "name": "a", "inner": { "name": "b" } },
                "boxed": { "name": "c", "inner": { "name": "d" } }
            }"#,
        )
        .unwrap();

        assert_eq!(config.one.name, "a");
        assert_eq!(config.one.inner.name, "b");
        assert_eq!(config.boxed.inner.name, "d");
        assert!(config.maybe.is_none());
    }

    #[derive(Debug, Config)]
    #[serde(default)]
    pub struct ContainerDefault {
        #[setting(required)]
        name: String,
        other: String,
    }

    #[test]
    fn is_never_optional_in_the_schema() {
        let schema = SchemaBuilder::build_root::<ContainerDefault>();

        let SchemaType::Struct(inner) = schema.ty else {
            panic!("expected a struct");
        };

        assert!(!inner.fields["name"].optional);
        assert!(inner.fields["other"].optional);
    }
}

// `partial(...)` may be repeated, within one attribute or across several,
// and every occurrence is forwarded to the partial in the order declared.
// https://github.com/moonrepo/schematic/issues/187
mod repeated_partial {
    use super::*;

    #[derive(Debug, Config)]
    #[config(partial(derive(Eq)))]
    #[config(partial(serde(rename_all = "UPPERCASE")))]
    pub struct Repeated {
        #[setting(partial(serde(alias = "nom")), partial(serde(alias = "nombre")))]
        name: String,
        #[setting(partial(serde(alias = "c")))]
        #[setting(partial(serde(alias = "cnt")))]
        count: usize,
    }

    #[derive(Debug, Config)]
    pub enum RepeatedVariant {
        #[setting(partial(serde(alias = "a")))]
        #[setting(partial(serde(alias = "alpha")))]
        A(String),
        B(bool),
    }

    fn load(code: &str) -> Repeated {
        ConfigLoader::<Repeated>::new()
            .code(code, "code.json")
            .unwrap()
            .load()
            .unwrap()
            .config
    }

    #[test]
    fn applies_every_container_occurrence() {
        fn is_eq<T: Eq>() {}

        is_eq::<PartialRepeated>();

        let config = load(r#"{ "NAME": "a", "COUNT": 1 }"#);

        assert_eq!(config.name, "a");
        assert_eq!(config.count, 1);
    }

    #[test]
    fn applies_every_field_occurrence() {
        let config = load(r#"{ "nom": "a", "c": 1 }"#);

        assert_eq!(config.name, "a");
        assert_eq!(config.count, 1);

        let config = load(r#"{ "nombre": "b", "cnt": 2 }"#);

        assert_eq!(config.name, "b");
        assert_eq!(config.count, 2);
    }

    #[test]
    fn applies_every_variant_occurrence() {
        for code in [r#"{ "a": "x" }"#, r#"{ "alpha": "x" }"#] {
            let config = ConfigLoader::<RepeatedVariant>::new()
                .code(code, "code.json")
                .unwrap()
                .load()
                .unwrap()
                .config;

            assert!(matches!(config, RepeatedVariant::A(value) if value == "x"));
        }
    }
}

// A literal union, like `"foo" | "bar" | boolean`: externally tagged, so a
// unit variant is written as its name, with every variant that holds a value
// untagged, so it's written as that value alone. The partial deserializes it
// by hand, so a mismatch lists what was expected.
mod literal_union {
    use super::*;

    #[derive(Debug, Config, PartialEq)]
    #[serde(rename_all = "kebab-case")]
    pub enum Mode {
        #[serde(alias = "f")]
        Foo,
        #[setting(default)]
        Bar,
        Baz,
        #[serde(untagged)]
        Enabled(bool),
    }

    fn parse(code: &str) -> Result<PartialMode, String> {
        serde_json::from_str(code).map_err(|error| error.to_string())
    }

    #[test]
    fn deserializes_named_values() {
        assert_eq!(parse(r#""foo""#).unwrap(), PartialMode::Foo);
        assert_eq!(parse(r#""f""#).unwrap(), PartialMode::Foo);
        assert_eq!(parse(r#""baz""#).unwrap(), PartialMode::Baz);
    }

    #[test]
    fn deserializes_untagged_values() {
        assert_eq!(parse("true").unwrap(), PartialMode::Enabled(true));
        assert_eq!(parse("false").unwrap(), PartialMode::Enabled(false));
    }

    // Serialize is still derived, and writes what deserialize reads
    #[test]
    fn round_trips() {
        for partial in [PartialMode::Foo, PartialMode::Enabled(true)] {
            let code = serde_json::to_string(&partial).unwrap();

            assert_eq!(parse(&code).unwrap(), partial);
        }

        assert_eq!(
            serde_json::to_string(&PartialMode::Bar).unwrap(),
            r#""bar""#
        );
        assert_eq!(
            serde_json::to_string(&PartialMode::Enabled(false)).unwrap(),
            "false"
        );
    }

    #[test]
    fn lists_the_named_values_on_failure() {
        assert_eq!(
            parse(r#""qux""#).unwrap_err(),
            "failed to parse as any variant of PartialMode:\n- expected one of \"foo\", \"bar\", \"baz\", found \"qux\"\n- enabled: expected a boolean, found \"qux\""
        );

        assert_eq!(
            parse("1").unwrap_err(),
            "failed to parse as any variant of PartialMode:\n- expected one of \"foo\", \"bar\", \"baz\", found 1\n- enabled: expected a boolean, found 1"
        );
    }

    #[derive(Debug, Config, PartialEq)]
    pub enum Single {
        Auto,
        #[serde(untagged)]
        Count(usize),
    }

    #[test]
    fn names_a_single_value_on_its_own() {
        assert_eq!(
            serde_json::from_str::<PartialSingle>(r#""auto-ish""#)
                .unwrap_err()
                .to_string(),
            "failed to parse as any variant of PartialSingle:\n- expected \"Auto\", found \"auto-ish\"\n- Count: expected a 64-bit unsigned integer, found \"auto-ish\""
        );
    }

    // Untagged variants are attempted in order, after every named value, so
    // a string catch-all doesn't shadow them
    #[derive(Debug, Config, PartialEq)]
    pub enum CatchAll {
        Known,
        #[serde(untagged)]
        Missing,
        #[serde(untagged)]
        Other(String),
    }

    #[test]
    fn matches_named_values_before_untagged_ones() {
        assert_eq!(
            serde_json::from_str::<PartialCatchAll>(r#""Known""#).unwrap(),
            PartialCatchAll::Known
        );
        assert_eq!(
            serde_json::from_str::<PartialCatchAll>(r#""other""#).unwrap(),
            PartialCatchAll::Other("other".into())
        );
    }

    // An untagged unit is `null`, as it is to serde
    #[test]
    fn deserializes_an_untagged_unit_from_null() {
        assert_eq!(
            serde_json::from_str::<PartialCatchAll>("null").unwrap(),
            PartialCatchAll::Missing
        );
    }

    #[derive(Debug, Config, PartialEq)]
    pub enum Skipped {
        Shown,
        #[serde(skip_deserializing)]
        Hidden,
        #[serde(untagged)]
        Count(usize),
    }

    #[test]
    fn never_deserializes_a_skipped_variant() {
        assert_eq!(
            serde_json::from_str::<PartialSkipped>(r#""Hidden""#)
                .unwrap_err()
                .to_string(),
            "failed to parse as any variant of PartialSkipped:\n- expected \"Shown\", found \"Hidden\"\n- Count: expected a 64-bit unsigned integer, found \"Hidden\""
        );
    }

    // A tagged variant that holds a value is written as a map, so the enum
    // isn't a literal union, and serde's derive is kept
    #[derive(Debug, Config, PartialEq)]
    pub enum Mixed {
        Unit,
        Tagged(String),
        #[serde(untagged)]
        Untagged(bool),
    }

    #[test]
    fn keeps_the_derive_when_a_tagged_variant_holds_a_value() {
        assert_eq!(
            serde_json::from_str::<PartialMixed>(r#"{ "Tagged": "a" }"#).unwrap(),
            PartialMixed::Tagged("a".into())
        );
        assert_eq!(
            serde_json::from_str::<PartialMixed>("true").unwrap(),
            PartialMixed::Untagged(true)
        );
    }

    #[derive(Debug, Config)]
    pub struct Settings {
        #[setting(nested)]
        mode: Mode,
        #[setting(nested)]
        maybe: Option<Mode>,
    }

    fn load(codes: &[&str]) -> Result<Settings, String> {
        let mut loader = ConfigLoader::<Settings>::new();

        for code in codes {
            loader.code(*code, "code.json").unwrap();
        }

        loader
            .load()
            .map(|result| result.config)
            .map_err(|error| error.to_full_string())
    }

    #[test]
    fn loads_through_the_loader() {
        let config = load(&[]).unwrap();

        assert_eq!(config.mode, Mode::Bar);
        assert_eq!(config.maybe, None);

        let config = load(&[r#"{ "mode": true, "maybe": "baz" }"#]).unwrap();

        assert_eq!(config.mode, Mode::Enabled(true));
        assert_eq!(config.maybe, Some(Mode::Baz));
    }

    // A later layer replaces the value, whichever form either is written in
    #[test]
    fn replaces_across_layers() {
        let config = load(&[r#"{ "mode": "foo" }"#, r#"{ "mode": false }"#]).unwrap();

        assert_eq!(config.mode, Mode::Enabled(false));

        let config = load(&[r#"{ "mode": true }"#, r#"{ "mode": "baz" }"#]).unwrap();

        assert_eq!(config.mode, Mode::Baz);
    }

    #[test]
    fn reports_the_setting_that_failed() {
        assert_eq!(
            load(&[r#"{ "mode": "qux" }"#]).unwrap_err(),
            "Failed to parse code.json. mode: failed to parse as any variant of PartialMode:\n- expected one of \"foo\", \"bar\", \"baz\", found \"qux\"\n- enabled: expected a boolean, found \"qux\" at line 1 column 17"
        );
    }

    #[derive(Debug, Config, PartialEq)]
    #[serde(rename_all = "kebab-case")]
    pub enum Toggle {
        Local,
        Remote,
        #[serde(untagged)]
        #[setting(default = true)]
        Enabled(bool),
    }

    // A value marks the variant as the default, and is what it holds
    #[test]
    fn defaults_to_an_untagged_value() {
        assert_eq!(Toggle::default(), Toggle::Enabled(true));
    }

    // Like the empty partial of a struct, the partial's `Default` holds the
    // default of each type, and `default_values` applies the value
    #[test]
    fn leaves_the_value_out_of_the_partial_default() {
        assert_eq!(PartialToggle::default(), PartialToggle::Enabled(false));
        assert_eq!(
            PartialToggle::default().default_values(&()).unwrap(),
            Some(PartialToggle::Enabled(true))
        );
    }

    #[derive(Debug, Config)]
    pub struct ToggleSettings {
        #[setting(nested)]
        toggle: Toggle,
    }

    #[test]
    fn loads_the_default_value() {
        let load = |codes: &[&str]| {
            let mut loader = ConfigLoader::<ToggleSettings>::new();

            for code in codes {
                loader.code(*code, "code.json").unwrap();
            }

            loader.load().unwrap().config.toggle
        };

        assert_eq!(load(&[]), Toggle::Enabled(true));
        assert_eq!(load(&[r#"{ "toggle": "local" }"#]), Toggle::Local);
        assert_eq!(load(&[r#"{ "toggle": false }"#]), Toggle::Enabled(false));
    }

    #[test]
    fn records_the_default_value_in_the_schema() {
        use schematic::SchemaBuilder;
        use schematic::schema::LiteralValue;

        assert_eq!(
            SchemaBuilder::build_root::<Toggle>().get_default(),
            Some(&LiteralValue::Bool(true))
        );
    }
}

// A variant holding a value takes `#[setting(default = value)]`, which
// accepts what a setting's does.
mod variant_default_values {
    use super::*;
    use schematic::DefaultValueResult;

    fn some_count(_: &()) -> DefaultValueResult<usize> {
        Ok(Some(3))
    }

    fn no_count(_: &()) -> DefaultValueResult<usize> {
        Ok(None)
    }

    #[derive(Debug, Config, PartialEq)]
    pub enum Text {
        #[setting(default = "abc")]
        Value(String),
        Other(usize),
    }

    #[derive(Debug, Config, PartialEq)]
    pub enum Counted {
        #[setting(default = some_count)]
        Count(usize),
        Other(bool),
    }

    #[derive(Debug, Config, PartialEq)]
    pub enum Uncounted {
        #[setting(default = no_count)]
        Count(usize),
        Other(bool),
    }

    #[derive(Debug, Config, PartialEq)]
    pub enum Wrapped {
        #[setting(default = 5)]
        Maybe(Option<usize>),
        Other(bool),
    }

    #[derive(Debug, Config, PartialEq)]
    pub enum Listed {
        #[setting(default = vec![1, 2])]
        List(Vec<usize>),
        Other(bool),
    }

    #[test]
    fn converts_a_string() {
        assert_eq!(Text::default(), Text::Value("abc".into()));
    }

    #[test]
    fn calls_a_handler_function() {
        assert_eq!(Counted::default(), Counted::Count(3));
    }

    // As with a setting, `None` falls back to the default of the type
    #[test]
    fn falls_back_when_a_handler_returns_none() {
        assert_eq!(Uncounted::default(), Uncounted::Count(0));
    }

    #[test]
    fn wraps_the_value_in_its_option() {
        assert_eq!(Wrapped::default(), Wrapped::Maybe(Some(5)));
    }

    #[test]
    fn writes_a_collection_whole() {
        assert_eq!(Listed::default(), Listed::List(vec![1, 2]));
    }

    // A unit variant has no value, so a boolean marks it, as before
    #[derive(Debug, Config, PartialEq)]
    pub enum Units {
        #[setting(default = false)]
        First,
        #[setting(default = true)]
        Second,
    }

    #[test]
    fn marks_a_unit_variant_with_a_boolean() {
        assert_eq!(Units::default(), Units::Second);
    }
}

// A nested setting may hold collections of collections, which finalize,
// merge, and validate all walk, like moon's map of file groups to inputs
mod nested_collections {
    use super::*;

    #[derive(Debug, Config, PartialEq)]
    pub struct Item {
        #[setting(required)]
        name: String,
        #[setting(default = 1)]
        weight: usize,
    }

    #[derive(Debug, Config)]
    pub struct Groups {
        #[setting(nested = Item)]
        groups: HashMap<String, Vec<Item>>,
        #[setting(nested)]
        grid: Vec<Vec<Item>>,
    }

    fn load(code: &str) -> Result<Groups, String> {
        ConfigLoader::<Groups>::new()
            .code(code, "code.json")
            .unwrap()
            .load()
            .map(|result| result.config)
            .map_err(|error| error.to_full_string())
    }

    #[test]
    fn finalizes_every_item() {
        let config =
            load(r#"{ "groups": { "a": [{ "name": "x" }] }, "grid": [[{ "name": "y" }]] }"#)
                .unwrap();

        assert_eq!(
            config.groups["a"],
            vec![Item {
                name: "x".into(),
                weight: 1
            }]
        );
        assert_eq!(config.grid[0][0].weight, 1);
    }

    #[test]
    fn reports_an_item_at_its_path() {
        assert_eq!(
            load(r#"{ "groups": { "a": [{ "name": "x" }, {}] }, "grid": [[], [{}]] }"#)
                .unwrap_err(),
            "Failed to validate code.json. \n  groups.a[1].name: this setting is required\n  grid[1][0].name: this setting is required"
        );
    }
}

mod raw_identifiers {
    use super::*;
    use std::env;

    #[derive(Debug, Config)]
    #[config(env_prefix = "RAW_IDENT_")]
    pub struct Raw {
        r#type: String,
        #[setting(validate = schematic::validate::not_empty)]
        r#ref: String,
    }

    #[test]
    fn strips_the_raw_prefix_from_names() {
        let settings = Raw::settings();

        assert_eq!(
            settings.keys().map(String::as_str).collect::<Vec<_>>(),
            vec!["ref", "type"]
        );
        assert_eq!(settings["type"].env_key.as_deref(), Some("RAW_IDENT_TYPE"));
    }

    #[test]
    fn reads_the_unprefixed_env_var() {
        unsafe { env::set_var("RAW_IDENT_TYPE", "foo") };

        let result = ConfigLoader::<Raw>::new()
            .code(r#"{ "ref": "bar" }"#, "code.json")
            .unwrap()
            .load();

        unsafe { env::remove_var("RAW_IDENT_TYPE") };

        let config = result.unwrap().config;

        assert_eq!(config.r#type, "foo");
        assert_eq!(config.r#ref, "bar");
    }

    #[test]
    fn reports_the_unprefixed_name_when_validating() {
        let error = ConfigLoader::<Raw>::new()
            .code(r#"{ "ref": "" }"#, "code.json")
            .unwrap()
            .load()
            .err()
            .unwrap();

        assert!(error.to_full_string().contains("  ref: "));
    }
}

mod suffixed_defaults {
    use super::*;
    use schematic::schema::{LiteralValue, SchemaType};

    #[derive(Debug, Config)]
    pub struct Suffixed {
        #[setting(default = 5u8)]
        small: u8,
        #[setting(default = 5i32)]
        signed: i32,
        #[setting(default = 1.5f32)]
        float: f32,
    }

    #[test]
    fn uses_the_value() {
        let config = Suffixed::default();

        assert_eq!(config.small, 5);
        assert_eq!(config.signed, 5);
        assert_eq!(config.float, 1.5);
    }

    #[test]
    fn records_the_default_in_the_schema() {
        let schema = schematic::SchemaBuilder::build_root::<Suffixed>();
        let SchemaType::Struct(inner) = &schema.ty else {
            panic!();
        };

        assert_eq!(
            inner.fields["small"].get_default(),
            Some(&LiteralValue::UInt(5))
        );
        assert_eq!(
            inner.fields["signed"].get_default(),
            Some(&LiteralValue::Int(5))
        );
    }
}

mod renamed_partial_schema {
    use super::*;
    use schematic::Schematic;

    #[derive(Debug, Config)]
    #[serde(rename = "Kid")]
    pub struct Child {
        name: String,
    }

    #[test]
    fn names_the_partial_after_the_rename() {
        assert_eq!(Child::schema_name().as_deref(), Some("Kid"));
        assert_eq!(PartialChild::schema_name().as_deref(), Some("PartialKid"));
    }
}

mod layers {
    use super::*;
    use std::collections::{LinkedList, VecDeque};
    use std::net::IpAddr;

    #[derive(Clone, Debug, Config, PartialEq, Serialize)]
    pub struct Item {
        #[setting(default = 1)]
        weight: usize,
    }

    #[derive(Debug, Config)]
    pub struct Collections {
        #[setting(nested)]
        deque: VecDeque<Item>,
        #[setting(nested)]
        linked: LinkedList<Item>,
        arcs: VecDeque<std::sync::Arc<String>>,
    }

    #[test]
    fn keeps_items_of_any_known_collection() {
        let config = ConfigLoader::<Collections>::new()
            .code(
                r#"{ "deque": [{ "weight": 2 }, {}], "linked": [{}], "arcs": ["a"] }"#,
                "code.json",
            )
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(
            config.deque,
            VecDeque::from([Item { weight: 2 }, Item { weight: 1 }])
        );
        assert_eq!(config.linked, LinkedList::from([Item { weight: 1 }]));
        assert_eq!(
            config.arcs,
            VecDeque::from([std::sync::Arc::new("a".into())])
        );
    }

    // Nothing is imported by name, so every path resolves as written
    #[allow(clippy::vec_box)]
    #[derive(Debug, Config)]
    pub struct Qualified {
        name: std::sync::Arc<String>,
        shared: std::rc::Rc<String>,
        #[setting(nested)]
        map: std::collections::BTreeMap<String, Item>,
        #[setting(nested)]
        list: std::vec::Vec<std::boxed::Box<Item>>,
    }

    #[test]
    fn resolves_qualified_paths() {
        let config = ConfigLoader::<Qualified>::new()
            .code(
                r#"{ "name": "a", "shared": "b", "map": { "x": {} }, "list": [{}] }"#,
                "code.json",
            )
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(*config.name, "a");
        assert_eq!(*config.shared, "b");
        assert_eq!(config.map["x"].weight, 1);
        assert_eq!(config.list[0].weight, 1);
    }

    #[derive(Debug, Config)]
    pub struct Unsized {
        #[setting(default = "abc")]
        name: Box<str>,
        #[setting(default = "def")]
        maybe: Option<Box<str>>,
        list: Vec<Box<str>>,
    }

    #[test]
    fn keeps_wrappers_of_unsized_types() {
        let config = ConfigLoader::<Unsized>::new()
            .code(r#"{ "list": ["x"] }"#, "code.json")
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(&*config.name, "abc");
        assert_eq!(config.maybe.as_deref(), Some("def"));
        assert_eq!(config.list, vec![Box::from("x")]);
    }

    #[derive(Debug, Config)]
    pub struct ParsedDefaults {
        #[setting(default = "127.0.0.1")]
        host: Option<IpAddr>,
        #[setting(default = "127.0.0.1:8080")]
        addr: Option<std::net::SocketAddr>,
    }

    #[test]
    fn parses_a_string_default_that_has_no_try_from() {
        let config = ParsedDefaults::default();

        assert_eq!(config.host, Some("127.0.0.1".parse::<IpAddr>().unwrap()));
        assert_eq!(config.addr, Some("127.0.0.1:8080".parse().unwrap()));
    }
}

mod recursion {
    use super::*;

    #[derive(Debug, Config, PartialEq)]
    pub struct Linked {
        #[setting(validate = schematic::validate::not_empty)]
        name: String,
        #[setting(nested)]
        next: Option<Box<Self>>,
    }

    fn load(code: &str) -> Result<Linked, String> {
        ConfigLoader::<Linked>::new()
            .code(code, "code.json")
            .unwrap()
            .load()
            .map(|result| result.config)
            .map_err(|error| error.to_full_string())
    }

    #[test]
    fn loads_a_boxed_self() {
        let config =
            load(r#"{ "name": "a", "next": { "name": "b", "next": { "name": "c" } } }"#).unwrap();

        assert_eq!(config.next.as_ref().unwrap().name, "b");
        assert_eq!(config.next.unwrap().next.unwrap().name, "c");
    }

    #[test]
    fn merges_a_boxed_self() {
        let config = ConfigLoader::<Linked>::new()
            .code(r#"{ "name": "a", "next": { "name": "b" } }"#, "a.json")
            .unwrap()
            .code(r#"{ "next": { "next": { "name": "c" } } }"#, "b.json")
            .unwrap()
            .load()
            .unwrap()
            .config;

        let next = config.next.unwrap();

        assert_eq!(next.name, "b");
        assert_eq!(next.next.unwrap().name, "c");
    }

    #[test]
    fn validates_a_boxed_self() {
        assert!(
            load(r#"{ "name": "a", "next": { "name": "" } }"#)
                .unwrap_err()
                .contains("next.name: ")
        );
    }

    #[derive(Debug, Config)]
    pub struct Tree {
        #[setting(nested)]
        children: Vec<Tree>,
        #[setting(nested)]
        linked: Option<Linked>,
    }

    #[test]
    fn collects_settings_without_recursing_forever() {
        let settings = Tree::settings();

        // A config's settings are collected once within itself, and the
        // setting that would repeat them again is left without any
        let children = settings["children"].nested.as_ref().unwrap();

        assert!(children.contains_key("linked"));
        assert!(children["children"].nested.is_none());

        let linked = settings["linked"].nested.as_ref().unwrap();

        assert!(linked.contains_key("name"));
        assert!(linked["next"].nested.is_none());
    }
}

mod nested_from_another_module {
    use super::*;

    mod child {
        use super::*;

        #[derive(Debug, Config)]
        pub struct Child {
            #[setting(default = "x")]
            pub name: String,
        }
    }

    // Neither `Child` nor `PartialChild` is imported
    #[derive(Debug, Config)]
    pub struct Parent {
        #[setting(nested)]
        child: child::Child,
        #[setting(nested)]
        children: Vec<child::Child>,
    }

    #[test]
    fn resolves_the_nested_type_by_path() {
        let config = ConfigLoader::<Parent>::new()
            .code(r#"{ "children": [{}] }"#, "code.json")
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(config.child.name, "x");
        assert_eq!(config.children[0].name, "x");
        assert!(
            Parent::settings()["child"]
                .nested
                .as_ref()
                .unwrap()
                .contains_key("name")
        );
    }
}

mod variant_fixes {
    use super::*;

    #[derive(Clone, Debug, Config, PartialEq, Serialize)]
    pub struct FileSource {
        #[setting(default = "a.txt")]
        path: String,
    }

    #[derive(Clone, Debug, Config, PartialEq, Serialize)]
    pub enum Source {
        #[setting(default, nested)]
        File(FileSource),
        Url(String),
    }

    #[derive(Debug, Config)]
    pub struct Parent {
        #[setting(nested)]
        source: Source,
    }

    #[test]
    fn defaults_a_nested_variant_to_its_default_values() {
        assert_eq!(
            Source::default(),
            Source::File(FileSource {
                path: "a.txt".into()
            })
        );
        assert_eq!(Parent::default().source, Source::default());
    }

    #[derive(Clone, Debug, Config, PartialEq, Serialize)]
    pub enum Pair {
        #[setting(required)]
        Both(Option<String>, Option<u32>),
    }

    #[test]
    fn requires_values_of_different_types() {
        let error = ConfigLoader::<Holder>::new()
            .code(r#"{ "pair": { "Both": ["a", null] } }"#, "code.json")
            .unwrap()
            .load()
            .err()
            .unwrap()
            .to_full_string();

        assert!(
            error.contains("pair.Both: this setting is required"),
            "{error}"
        );
    }

    #[derive(Debug, Config)]
    pub struct Holder {
        #[setting(nested)]
        pair: Option<Pair>,
    }
}

mod flattened_validation {
    use super::*;

    #[derive(Clone, Debug, Config, PartialEq, Serialize)]
    pub struct Common {
        #[setting(validate = schematic::validate::not_empty)]
        label: String,
    }

    #[derive(Debug, Config)]
    pub struct Flattened {
        own: bool,
        #[setting(nested)]
        #[serde(flatten)]
        common: Common,
    }

    #[test]
    fn reports_at_the_path_written() {
        let error = ConfigLoader::<Flattened>::new()
            .code(r#"{ "own": true, "label": "" }"#, "code.json")
            .unwrap()
            .load()
            .err()
            .unwrap()
            .to_full_string();

        assert!(error.contains("\n  label: "), "{error}");
        assert!(!error.contains("common"), "{error}");
    }
}

mod serde_default_function {
    use super::*;

    fn default_port() -> u16 {
        8080
    }

    fn default_host() -> Option<String> {
        Some("localhost".into())
    }

    #[derive(Debug, Config)]
    pub struct Server {
        #[serde(default = "default_port")]
        port: u16,
        #[serde(default = "default_host")]
        host: Option<String>,
    }

    #[test]
    fn uses_the_function_as_the_default() {
        let config = ConfigLoader::<Server>::new()
            .code("{}", "code.json")
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(config.port, 8080);
        assert_eq!(config.host.as_deref(), Some("localhost"));
        assert_eq!(Server::default().port, 8080);
    }
}

// Default values and environment variables are applied once, wherever a
// config is nested, which a merge that appends would otherwise repeat.
mod applied_once {
    use super::*;
    use schematic::merge::append_vec;
    use std::env;

    fn add_tag(mut value: Vec<String>, _: &()) -> schematic::TransformResult<Vec<String>> {
        value.push("t".into());
        Ok(value)
    }

    #[derive(Clone, Debug, Config, PartialEq, Serialize)]
    pub struct Child {
        #[setting(default = vec!["d".to_owned()], merge = append_vec)]
        list: Vec<String>,
        #[setting(transform = add_tag)]
        tags: Vec<String>,
    }

    #[derive(Debug, Config)]
    pub struct Parent {
        #[setting(nested)]
        child: Child,
        #[setting(nested)]
        maybe: Option<Child>,
        #[setting(nested)]
        items: Vec<Child>,
    }

    #[derive(Debug, Config)]
    pub struct Grandparent {
        #[setting(nested)]
        parent: Parent,
    }

    fn load<T: ConfigTrait>(code: &str) -> T {
        ConfigLoader::<T>::new()
            .code(code, "code.json")
            .unwrap()
            .load()
            .unwrap()
            .config
    }

    #[test]
    fn applies_defaults_of_a_nested_config_once() {
        let config = load::<Parent>(r#"{ "child": { "list": ["f"] } }"#);

        assert_eq!(config.child.list, vec!["d", "f"]);
        assert_eq!(config.child.tags, vec!["t"]);
    }

    #[test]
    fn applies_defaults_once_at_any_depth() {
        let config = load::<Grandparent>(r#"{ "parent": { "child": { "list": ["f"] } } }"#);

        assert_eq!(config.parent.child.list, vec!["d", "f"]);
        assert_eq!(config.parent.child.tags, vec!["t"]);
    }

    #[test]
    fn matches_default_without_sources() {
        let config = load::<Grandparent>("{}");

        assert_eq!(config.parent.child.list, vec!["d"]);
        assert_eq!(Grandparent::default().parent.child.list, vec!["d"]);
    }

    #[test]
    fn applies_defaults_of_an_optional_config_once() {
        let config = load::<Grandparent>(r#"{ "parent": { "maybe": { "list": ["f"] } } }"#);
        let maybe = config.parent.maybe.unwrap();

        assert_eq!(maybe.list, vec!["d", "f"]);
        assert_eq!(maybe.tags, vec!["t"]);
    }

    #[test]
    fn applies_defaults_of_each_item_once() {
        let config = load::<Grandparent>(r#"{ "parent": { "items": [{}, { "list": ["f"] }] } }"#);

        assert_eq!(config.parent.items[0].list, vec!["d"]);
        assert_eq!(config.parent.items[1].list, vec!["d", "f"]);
        assert_eq!(config.parent.items[1].tags, vec!["t"]);
    }

    // Each test that reads the environment has its own types and prefixes,
    // as tests run in parallel and share the process environment

    #[derive(Clone, Debug, Config, PartialEq, Serialize)]
    #[config(env_prefix = "ONCE_ENV_")]
    pub struct EnvChild {
        #[setting(default = vec!["d".to_owned()], merge = append_vec, parse_env = schematic::env::split_comma)]
        list: Vec<String>,
    }

    #[derive(Debug, Config)]
    pub struct EnvParent {
        #[setting(nested)]
        child: EnvChild,
        #[setting(nested)]
        maybe: Option<EnvChild>,
    }

    #[derive(Debug, Config)]
    pub struct EnvGrandparent {
        #[setting(nested)]
        parent: EnvParent,
    }

    #[test]
    fn applies_environment_variables_once() {
        unsafe { env::set_var("ONCE_ENV_LIST", "e") };

        let config = ConfigLoader::<EnvGrandparent>::new()
            .code(
                r#"{ "parent": { "child": { "list": ["f"] }, "maybe": {} } }"#,
                "code.json",
            )
            .unwrap()
            .load();

        unsafe { env::remove_var("ONCE_ENV_LIST") };

        let config = config.unwrap().config;

        assert_eq!(config.parent.child.list, vec!["d", "f", "e"]);
        assert_eq!(config.parent.maybe.unwrap().list, vec!["d", "e"]);
    }

    #[derive(Clone, Debug, Config, PartialEq, Serialize)]
    #[config(env_prefix = "ONCE_OWN_A_")]
    pub struct OwnChildA {
        #[setting(default = vec!["d".to_owned()], merge = append_vec, parse_env = schematic::env::split_comma)]
        list: Vec<String>,
    }

    #[derive(Debug, Config)]
    pub struct PrefixedParentA {
        #[setting(nested, env_prefix = "ONCE_PARENT_A_")]
        child: OwnChildA,
    }

    #[test]
    fn applies_an_own_prefix_once_and_over_a_parent_prefix() {
        unsafe {
            env::set_var("ONCE_OWN_A_LIST", "own");
            env::set_var("ONCE_PARENT_A_LIST", "parent");
        };

        let config = ConfigLoader::<PrefixedParentA>::new().load();

        unsafe {
            env::remove_var("ONCE_OWN_A_LIST");
            env::remove_var("ONCE_PARENT_A_LIST");
        };

        assert_eq!(config.unwrap().config.child.list, vec!["d", "own"]);
    }

    #[derive(Clone, Debug, Config, PartialEq, Serialize)]
    #[config(env_prefix = "ONCE_OWN_B_")]
    pub struct OwnChildB {
        #[setting(default = vec!["d".to_owned()], merge = append_vec, parse_env = schematic::env::split_comma)]
        list: Vec<String>,
    }

    #[derive(Debug, Config)]
    pub struct PrefixedParentB {
        #[setting(nested, env_prefix = "ONCE_PARENT_B_")]
        child: OwnChildB,
    }

    #[test]
    fn reads_a_parent_prefix_when_the_own_is_missing() {
        unsafe { env::set_var("ONCE_PARENT_B_LIST", "parent") };

        let config = ConfigLoader::<PrefixedParentB>::new().load();

        unsafe { env::remove_var("ONCE_PARENT_B_LIST") };

        assert_eq!(config.unwrap().config.child.list, vec!["d", "parent"]);
    }

    #[derive(Clone, Debug, Config, PartialEq, Serialize)]
    pub enum Source {
        #[setting(default, nested)]
        File(Child),
        Url(String),
    }

    #[derive(Debug, Config)]
    pub struct HasSource {
        #[setting(nested)]
        source: Source,
    }

    #[test]
    fn applies_defaults_of_a_default_variant_once() {
        let config = load::<HasSource>(r#"{ "source": { "File": { "list": ["f"] } } }"#);

        assert_eq!(
            config.source,
            Source::File(Child {
                list: vec!["d".into(), "f".into()],
                tags: vec!["t".into()],
            })
        );

        let config = load::<HasSource>("{}");

        // `Default` holds the same defaults, though it runs no transforms
        let Source::File(default) = HasSource::default().source else {
            panic!();
        };

        assert_eq!(default.list, vec!["d"]);
        assert_eq!(
            config.source,
            Source::File(Child {
                list: vec!["d".into()],
                tags: vec!["t".into()],
            })
        );
    }

    #[test]
    fn applies_defaults_of_a_variant_switched_back_to() {
        let config = ConfigLoader::<HasSource>::new()
            .code(r#"{ "source": { "Url": "x" } }"#, "a.json")
            .unwrap()
            .code(r#"{ "source": { "File": { "list": ["f"] } } }"#, "b.json")
            .unwrap()
            .load()
            .unwrap()
            .config;

        let Source::File(child) = config.source else {
            panic!();
        };

        assert_eq!(child.list, vec!["d", "f"]);
    }
}

mod generic_nested {
    use super::generics::{Setting, Wrapper};
    use super::*;

    #[derive(Debug, Config)]
    pub struct Holder {
        #[setting(nested)]
        bare: Wrapper<String>,
        #[setting(nested)]
        maybe: Option<Wrapper<u8>>,
        #[setting(nested)]
        list: Vec<Wrapper<bool>>,
        #[setting(nested)]
        map: HashMap<String, Wrapper<usize>>,
        #[setting(nested = Wrapper)]
        explicit: Wrapper<String>,
    }

    #[test]
    fn loads_generic_nested_configs() {
        let config = ConfigLoader::<Holder>::new()
            .code(
                r#"{
                    "bare": { "inner": "a", "label": "b" },
                    "maybe": { "inner": 1 },
                    "list": [{ "inner": true }],
                    "map": { "k": { "inner": 2 } },
                    "explicit": { "label": "e" }
                }"#,
                "code.json",
            )
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(config.bare.inner, "a");
        assert_eq!(config.bare.label, "b");
        assert_eq!(config.maybe.unwrap().inner, 1);
        assert!(config.list[0].inner);
        assert_eq!(config.map["k"].inner, 2);
        assert_eq!(config.explicit.label, "e");
    }

    #[test]
    fn references_their_schemas_by_instantiation() {
        let schema = schematic::SchemaBuilder::build_root::<Holder>();
        let schematic::SchemaType::Struct(inner) = &schema.ty else {
            panic!();
        };

        assert_eq!(
            inner.fields["bare"].schema.name.as_deref(),
            Some("WrapperString")
        );
        assert_eq!(
            inner.fields["explicit"].schema.name.as_deref(),
            Some("WrapperString")
        );
    }

    #[test]
    fn collects_their_settings() {
        let settings = Holder::settings();

        assert!(
            settings["bare"]
                .nested
                .as_ref()
                .unwrap()
                .contains_key("inner")
        );
        assert!(
            settings["maybe"]
                .nested
                .as_ref()
                .unwrap()
                .contains_key("label")
        );
    }

    // A type parameter can be the nested config itself. The partial's derives
    // bound every type parameter, so it needs the same bounds as any other,
    // its partial is debugged and compared like any setting, and it shares
    // the parent's context.
    #[derive(Debug, Config)]
    pub struct Outer<C: ConfigTrait + Setting>
    where
        C::Partial: std::fmt::Debug + PartialEq + PartialConfig<Context = ()>,
    {
        #[setting(nested)]
        inner: C,
        name: String,
    }

    #[derive(Clone, Debug, Config, PartialEq, Serialize, serde::Deserialize)]
    pub struct Inner {
        inner: String,
    }

    #[test]
    fn nests_a_type_parameter() {
        let config = ConfigLoader::<Outer<Inner>>::new()
            .code(r#"{ "inner": { "inner": "x" }, "name": "n" }"#, "code.json")
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(config.inner.inner, "x");
        assert_eq!(config.name, "n");
    }

    // A generic config that holds itself through a `Box`
    #[derive(Debug, Config)]
    pub struct Tree<T: Setting> {
        value: T,
        #[setting(nested)]
        next: Option<Box<Tree<T>>>,
    }

    #[test]
    fn nests_a_generic_self() {
        let config = ConfigLoader::<Tree<u8>>::new()
            .code(r#"{ "value": 1, "next": { "value": 2 } }"#, "code.json")
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(config.value, 1);
        assert_eq!(config.next.unwrap().value, 2);
    }
}

// A nested config whose context differs from its parent's, such as one
// from a library, which can't know the application's context.
mod nested_context {
    use super::*;
    use schematic::{DefaultValueResult, ValidateError, ValidateResult};

    #[derive(Clone, Debug, Config, PartialEq, Serialize)]
    pub struct LibConfig {
        #[setting(default = 1)]
        level: usize,
        #[setting(merge = schematic::merge::append_vec)]
        tags: Vec<String>,
    }

    #[derive(Default)]
    pub struct PluginContext {
        name: String,
        max: usize,
    }

    fn default_name(context: &PluginContext) -> DefaultValueResult<String> {
        Ok(Some(context.name.clone()))
    }

    fn below_max<D>(value: &usize, _: &D, context: &PluginContext, _: bool) -> ValidateResult {
        if *value > context.max {
            return Err(ValidateError::new(format!(
                "must be at most {}",
                context.max
            )));
        }

        Ok(())
    }

    #[derive(Clone, Debug, Config, PartialEq, Serialize)]
    #[config(context = PluginContext)]
    pub struct PluginConfig {
        #[setting(default = default_name)]
        name: String,
        #[setting(validate = below_max)]
        size: usize,
    }

    #[derive(Default)]
    pub struct AppContext {
        plugin: PluginContext,
    }

    fn plugin_context(context: &AppContext) -> &PluginContext {
        &context.plugin
    }

    #[derive(Clone, Debug, Config, PartialEq, Serialize)]
    #[config(context = AppContext)]
    pub enum Source {
        #[setting(nested, context = schematic::context::unit)]
        Lib(LibConfig),
        Url(String),
    }

    #[derive(Debug, Config)]
    #[config(context = AppContext)]
    pub struct AppConfig {
        #[setting(nested, context = schematic::context::unit)]
        lib: LibConfig,
        #[setting(nested, context = schematic::context::unit)]
        maybe: Option<LibConfig>,
        #[setting(nested, context = schematic::context::unit)]
        libs: Vec<LibConfig>,
        #[setting(nested, context = plugin_context)]
        plugin: PluginConfig,
        #[setting(nested)]
        source: Option<Source>,
    }

    fn app_context() -> AppContext {
        AppContext {
            plugin: PluginContext {
                name: "from context".into(),
                max: 10,
            },
        }
    }

    fn load(sources: &[&str]) -> Result<AppConfig, String> {
        let mut loader = ConfigLoader::<AppConfig>::new();

        for (index, code) in sources.iter().enumerate() {
            loader.code(*code, format!("{index}.json")).unwrap();
        }

        loader
            .load_with_context(&app_context())
            .map(|result| result.config)
            .map_err(|error| error.to_full_string())
    }

    #[test]
    fn nests_a_config_with_a_unit_context() {
        let config = load(&[
            r#"{ "lib": { "tags": ["a"] }, "maybe": {}, "libs": [{ "level": 3 }] }"#,
            r#"{ "lib": { "tags": ["b"] }, "source": { "Lib": {} } }"#,
        ])
        .unwrap();

        assert_eq!(config.lib.level, 1);
        assert_eq!(config.lib.tags, vec!["a", "b"]);
        assert_eq!(config.maybe.unwrap().level, 1);
        assert_eq!(config.libs[0].level, 3);
        assert_eq!(
            config.source,
            Some(Source::Lib(LibConfig {
                level: 1,
                tags: vec![]
            }))
        );
    }

    #[test]
    fn maps_the_parent_context_for_defaults() {
        let config = load(&["{}"]).unwrap();

        assert_eq!(config.plugin.name, "from context");
    }

    #[test]
    fn maps_the_parent_context_for_validation() {
        let error = load(&[r#"{ "plugin": { "size": 11 } }"#]).unwrap_err();

        assert!(error.contains("plugin.size: must be at most 10"), "{error}");
    }

    #[test]
    fn defaults_without_a_context() {
        let config = AppConfig::default();

        assert_eq!(config.lib.level, 1);
        assert_eq!(config.plugin.name, "");
    }
}

// A hand-written partial overrides the documented methods, not the hidden
// ones a derived partial does, and is still validated and read from the
// environment when nested within a derived config.
mod hand_written_nested {
    use super::*;
    use schematic::{ConfigError, HandlerError, Schematic};
    use std::env;

    #[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, Serialize)]
    #[serde(default)]
    pub struct PartialLevel {
        pub level: Option<u8>,
    }

    impl Schematic for PartialLevel {}

    impl PartialConfig for PartialLevel {
        type Context = ();

        fn merge(&mut self, _: &(), next: Self) -> Result<(), ConfigError> {
            if next.level.is_some() {
                self.level = next.level;
            }

            Ok(())
        }

        fn validate(&self, _: &(), _: bool) -> Result<(), ConfigError> {
            if self.level.unwrap_or_default() > 5 {
                return Err(ConfigError::Handler(Box::new(HandlerError::new(
                    "level too high",
                ))));
            }

            Ok(())
        }
    }

    #[derive(Debug, Default, PartialEq)]
    pub struct Level {
        pub level: u8,
    }

    impl Schematic for Level {}

    impl ConfigTrait for Level {
        type Partial = PartialLevel;

        fn from_partial(partial: PartialLevel) -> Self {
            Level {
                level: partial.level.unwrap_or_default(),
            }
        }
    }

    #[derive(Debug, Config)]
    pub struct UsesLevel {
        #[setting(nested)]
        bare: Level,
        #[setting(nested)]
        maybe: Option<Level>,
        #[setting(nested)]
        list: Vec<Level>,
    }

    // Reads the environment itself, as its own type, since tests run in
    // parallel and share the process environment
    #[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, Serialize)]
    #[serde(default)]
    pub struct PartialEnvLevel {
        pub level: Option<u8>,
    }

    impl Schematic for PartialEnvLevel {}

    // Reads the prefix a parent passes, which it says it does, as otherwise a
    // parent's `env_prefix` would have no effect, which fails to compile
    impl PartialConfig for PartialEnvLevel {
        type Context = ();

        const HAS_ENV_PREFIX: bool = true;

        fn env_values(&self) -> Result<Option<Self>, ConfigError> {
            self.env_values_with_prefix(None)
        }

        fn env_values_with_prefix(
            &self,
            prefix: Option<&str>,
        ) -> Result<Option<Self>, ConfigError> {
            let key = format!("{}LEVEL", prefix.unwrap_or("HAND_"));

            Ok(env::var(key).ok().map(|value| PartialEnvLevel {
                level: value.parse().ok(),
            }))
        }
    }

    #[derive(Debug, Default, PartialEq)]
    pub struct EnvLevel {
        pub level: u8,
    }

    impl Schematic for EnvLevel {}

    impl ConfigTrait for EnvLevel {
        type Partial = PartialEnvLevel;

        fn from_partial(partial: PartialEnvLevel) -> Self {
            EnvLevel {
                level: partial.level.unwrap_or_default(),
            }
        }
    }

    #[derive(Debug, Config)]
    pub struct PrefixesLevel {
        #[setting(nested, env_prefix = "HAND_PARENT_")]
        level: EnvLevel,
    }

    #[test]
    fn validates_a_hand_written_partial_when_nested() {
        let error = ConfigLoader::<UsesLevel>::new()
            .code(
                r#"{ "bare": { "level": 9 }, "maybe": { "level": 7 }, "list": [{}, { "level": 6 }] }"#,
                "code.json",
            )
            .unwrap()
            .load()
            .err()
            .unwrap()
            .to_full_string();

        assert!(error.contains("bare: level too high"), "{error}");
        assert!(error.contains("maybe: level too high"), "{error}");
        assert!(error.contains("list[1]: level too high"), "{error}");
    }

    #[test]
    fn reads_its_environment_under_a_parent_prefix() {
        unsafe { env::set_var("HAND_PARENT_LEVEL", "3") };

        let config = ConfigLoader::<PrefixesLevel>::new().load();

        unsafe { env::remove_var("HAND_PARENT_LEVEL") };

        assert_eq!(config.unwrap().config.level.level, 3);
    }
}

// A variant can hold a nested config alongside plain values, by marking the
// value itself rather than the whole variant.
mod mixed_variant_values {
    use super::*;

    #[derive(Clone, Debug, Config, PartialEq, Serialize)]
    pub struct Child {
        #[setting(default = 1)]
        level: usize,
        #[setting(validate = schematic::validate::not_empty)]
        name: String,
    }

    #[derive(Clone, Debug, Config, PartialEq, Serialize)]
    pub enum Mixed {
        Both(#[setting(nested)] Child, bool),
        Plain(String),
    }

    #[derive(Debug, Config)]
    pub struct Holder {
        #[setting(nested)]
        value: Option<Mixed>,
    }

    fn load(sources: &[&str]) -> Result<Holder, String> {
        let mut loader = ConfigLoader::<Holder>::new();

        for (index, code) in sources.iter().enumerate() {
            loader.code(*code, format!("{index}.json")).unwrap();
        }

        loader
            .load()
            .map(|result| result.config)
            .map_err(|error| error.to_full_string())
    }

    #[test]
    fn finalizes_only_the_nested_value() {
        let config = load(&[r#"{ "value": { "Both": [{ "name": "a" }, true] } }"#]).unwrap();

        assert_eq!(
            config.value,
            Some(Mixed::Both(
                Child {
                    level: 1,
                    name: "a".into()
                },
                true
            ))
        );
    }

    #[test]
    fn merges_the_nested_value_and_replaces_the_plain_one() {
        let config = load(&[
            r#"{ "value": { "Both": [{ "name": "a", "level": 5 }, true] } }"#,
            r#"{ "value": { "Both": [{ "name": "b" }, false] } }"#,
        ])
        .unwrap();

        assert_eq!(
            config.value,
            Some(Mixed::Both(
                Child {
                    level: 5,
                    name: "b".into()
                },
                false
            ))
        );
    }

    #[test]
    fn validates_the_nested_value_at_its_position() {
        let error = load(&[r#"{ "value": { "Both": [{ "name": "" }, true] } }"#]).unwrap_err();

        assert!(error.contains("value.Both[0].name: "), "{error}");
    }

    #[test]
    fn marks_only_the_nested_value_as_partial_in_the_schema() {
        use schematic::{SchemaBuilder, SchemaType};

        let schema = SchemaBuilder::build_root::<Mixed>();
        let SchemaType::Union(union) = &schema.ty else {
            panic!("{schema:?}");
        };

        let SchemaType::Struct(both) = &union.variants_types[0].ty else {
            panic!();
        };

        let SchemaType::Tuple(values) = &both.fields["Both"].schema.ty else {
            panic!();
        };

        let SchemaType::Struct(child) = &values.items_types[0].ty else {
            panic!();
        };

        assert!(child.partial);
        assert!(matches!(values.items_types[1].ty, SchemaType::Boolean(_)));
    }
}

// A skipped setting is never read or written, so its type needn't have a schema
mod skipped_without_schema {
    use super::*;
    use schematic::{SchemaBuilder, SchemaType};

    // Implements what the partial derives, but not `Schematic` or serde
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct Runtime {
        handle: usize,
    }

    #[derive(Debug, Config)]
    pub struct WithRuntime {
        name: String,
        #[setting(skip)]
        runtime: Runtime,
        #[serde(skip)]
        other: Runtime,
    }

    #[test]
    fn leaves_it_out_of_the_schema() {
        let schema = SchemaBuilder::build_root::<WithRuntime>();
        let SchemaType::Struct(inner) = &schema.ty else {
            panic!();
        };

        assert_eq!(inner.fields.keys().collect::<Vec<_>>(), vec!["name"]);
    }

    #[test]
    fn still_loads() {
        let config = ConfigLoader::<WithRuntime>::new()
            .code(r#"{ "name": "a" }"#, "code.json")
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(config.name, "a");
        assert_eq!(config.runtime, Runtime::default());
    }
}

// Serde reads a flattened map's keys from its struct without taking them, so a
// struct that denied unknown fields beside one rejected every key it collects
mod flattened_loading {
    use super::*;

    #[derive(Debug, Config)]
    pub struct Plugin {
        #[setting(default = "latest")]
        version: String,
    }

    #[derive(Debug, Config)]
    pub struct WithValues {
        own: String,
        #[setting(flatten)]
        extra: HashMap<String, String>,
    }

    #[derive(Debug, Config)]
    pub struct WithPlugins {
        own: String,
        #[setting(flatten, nested)]
        plugins: HashMap<String, Plugin>,
    }

    fn load<T: ConfigTrait>(code: &str) -> Result<T, String> {
        ConfigLoader::<T>::new()
            .code(code, "a.json")
            .unwrap()
            .load()
            .map(|result| result.config)
            .map_err(|error| error.to_full_string())
    }

    #[test]
    fn collects_into_a_flattened_map() {
        let config = load::<WithValues>(r#"{ "own": "x", "a": "1", "b": "2" }"#).unwrap();

        assert_eq!(config.own, "x");
        assert_eq!(
            config.extra,
            HashMap::from_iter([("a".into(), "1".into()), ("b".into(), "2".into())])
        );
    }

    #[test]
    fn collects_configs_into_a_flattened_map() {
        let config =
            load::<WithPlugins>(r#"{ "own": "x", "foo": {}, "bar": { "version": "2" } }"#).unwrap();

        assert_eq!(config.plugins["foo"].version, "latest");
        assert_eq!(config.plugins["bar"].version, "2");
    }

    #[derive(Debug, Config)]
    pub struct Child {
        child_flag: bool,
    }

    #[derive(Debug, Config)]
    pub struct Parent {
        own: String,
        #[setting(flatten, nested)]
        child: Child,
    }

    // A plain flattened struct takes its keys, so the parent still denies the rest
    #[test]
    fn still_denies_unknown_fields_beside_a_flattened_struct() {
        let config = load::<Parent>(r#"{ "own": "x", "child_flag": true }"#).unwrap();

        assert!(config.child.child_flag);
        assert!(
            load::<Parent>(r#"{ "own": "x", "typo": true }"#)
                .unwrap_err()
                .contains("unknown field `typo`")
        );
    }

    #[derive(Debug, Config)]
    #[config(allow_unknown_fields)]
    pub struct OpenMiddle {
        middle_flag: bool,
        #[setting(flatten, nested)]
        child: Child,
    }

    #[derive(Debug, Config)]
    #[config(allow_unknown_fields)]
    pub struct OpenParent {
        own: String,
        #[setting(flatten, nested)]
        middle: OpenMiddle,
    }

    // What the compile-time check asks for, so that it loads
    #[test]
    fn loads_a_flattened_struct_within_another_that_allow_unknown_fields() {
        let config =
            load::<OpenParent>(r#"{ "own": "x", "middle_flag": true, "child_flag": true }"#)
                .unwrap();

        assert!(config.middle.middle_flag);
        assert!(config.middle.child.child_flag);
    }

    #[test]
    fn reports_what_a_partial_flattens() {
        const {
            assert!(!<PartialChild as PartialConfig>::FLATTENS);
            assert!(!<PartialChild as PartialConfig>::ALLOWS_UNKNOWN_FIELDS);
            assert!(<PartialParent as PartialConfig>::FLATTENS);
            assert!(!<PartialParent as PartialConfig>::ALLOWS_UNKNOWN_FIELDS);
            assert!(<PartialWithValues as PartialConfig>::FLATTENS);
            assert!(<PartialWithValues as PartialConfig>::ALLOWS_UNKNOWN_FIELDS);
            assert!(<PartialOpenMiddle as PartialConfig>::ALLOWS_UNKNOWN_FIELDS);
        }
    }
}

mod named_variants {
    use super::*;
    use schematic::merge::append_vec;
    use schematic::{ValidateError, ValidateResult};
    use std::collections::BTreeMap;

    fn add_suffix(value: String, _: &()) -> schematic::TransformResult<String> {
        Ok(format!("{value}!"))
    }

    fn not_empty<D, C>(value: &str, _: &D, _: &C, _: bool) -> ValidateResult {
        if value.is_empty() {
            return Err(ValidateError::new("must not be empty"));
        }

        Ok(())
    }

    fn default_timeout(_: &()) -> schematic::DefaultValueResult<u32> {
        Ok(Some(30))
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    pub struct Child {
        #[setting(default = 3)]
        retries: u8,
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum Source {
        #[setting(default)]
        File {
            #[setting(default = "config.json")]
            path: String,
            #[setting(default = vec!["d".to_owned()], merge = append_vec)]
            tags: Vec<String>,
        },
        Url {
            #[setting(required, validate = not_empty)]
            address: Option<String>,
            #[setting(transform = add_suffix)]
            name: String,
            #[setting(nested)]
            child: Child,
            #[setting(default = default_timeout)]
            timeout: u32,
        },
    }

    #[derive(Debug, Config)]
    pub struct Parent {
        #[setting(nested)]
        source: Source,
    }

    fn load<T: ConfigTrait>(codes: &[&str]) -> Result<T, String> {
        let mut loader = ConfigLoader::<T>::new();

        for (index, code) in codes.iter().enumerate() {
            loader.code(*code, format!("{index}.json")).unwrap();
        }

        loader
            .load()
            .map(|result| result.config)
            .map_err(|error| error.to_full_string())
    }

    // Each setting is optional within the partial, like a struct's
    #[test]
    fn settings_are_optional_in_the_partial() {
        let partial: PartialSource = serde_json::from_str(r#"{ "type": "file" }"#).unwrap();

        assert_eq!(
            partial,
            PartialSource::File {
                path: None,
                tags: None
            }
        );
        assert_eq!(
            PartialSource::default(),
            PartialSource::File {
                path: None,
                tags: None
            }
        );
    }

    #[test]
    fn defaults_to_the_marked_variant_with_its_defaults() {
        let expected = Source::File {
            path: "config.json".into(),
            tags: vec!["d".into()],
        };

        assert_eq!(Source::default(), expected);
        assert_eq!(load::<Source>(&[]).unwrap(), expected);
        assert_eq!(Parent::default().source, expected);
        assert_eq!(load::<Parent>(&[]).unwrap().source, expected);
    }

    // The defaults layer holds those of the variant a source picks, which
    // nested configs and transforms are applied over
    #[test]
    fn applies_the_defaults_of_another_variant() {
        let config =
            load::<Source>(&[r#"{ "type": "url", "address": "x", "name": "n" }"#]).unwrap();

        assert_eq!(
            config,
            Source::Url {
                address: Some("x".into()),
                name: "n!".into(),
                child: Child { retries: 3 },
                timeout: 30,
            }
        );
    }

    #[test]
    fn merges_settings_of_the_same_variant() {
        let config = load::<Parent>(&[
            r#"{ "source": { "type": "file", "path": "a.json", "tags": ["x"] } }"#,
            r#"{ "source": { "type": "file", "tags": ["y"] } }"#,
        ])
        .unwrap();

        // The default is merged under every layer once
        assert_eq!(
            config.source,
            Source::File {
                path: "a.json".into(),
                tags: vec!["d".into(), "x".into(), "y".into()],
            }
        );
    }

    #[test]
    fn replaces_another_variant() {
        let config = load::<Source>(&[
            r#"{ "type": "file", "path": "a.json" }"#,
            r#"{ "type": "url", "address": "x" }"#,
        ])
        .unwrap();

        assert!(matches!(config, Source::Url { .. }));

        let config = load::<Source>(&[
            r#"{ "type": "url", "address": "x" }"#,
            r#"{ "type": "file", "path": "b.json" }"#,
        ])
        .unwrap();

        assert_eq!(
            config,
            Source::File {
                path: "b.json".into(),
                tags: vec!["d".into()],
            }
        );
    }

    #[test]
    fn denies_unknown_fields() {
        let error = load::<Source>(&[r#"{ "type": "file", "nope": true }"#]).unwrap_err();

        assert!(error.contains("unknown field `nope`"), "{error}");
    }

    // Reported under the variant, as a struct's are under the struct
    #[test]
    fn validates_settings_under_the_variant() {
        let error = load::<Parent>(&[r#"{ "source": { "type": "url" } }"#]).unwrap_err();

        assert!(
            error.contains("source.url.address: this setting is required"),
            "{error}"
        );

        let error =
            load::<Parent>(&[r#"{ "source": { "type": "url", "address": "" } }"#]).unwrap_err();

        assert!(
            error.contains("source.url.address: must not be empty"),
            "{error}"
        );
    }

    #[test]
    fn lists_settings_of_each_variant() {
        let settings = Source::settings();

        assert_eq!(
            settings["file"]
                .nested
                .as_ref()
                .unwrap()
                .keys()
                .collect::<Vec<_>>(),
            vec!["path", "tags"]
        );
        assert!(
            settings["url"].nested.as_ref().unwrap()["child"]
                .nested
                .as_ref()
                .unwrap()
                .contains_key("retries")
        );
    }

    // Externally tagged, alongside unit and tuple variants
    #[derive(Clone, Debug, Config, PartialEq)]
    #[serde(rename_all_fields = "camelCase")]
    pub enum Mode {
        #[setting(default)]
        Off,
        Level(u8),
        On {
            #[setting(default = 5)]
            max_level: u8,
            #[serde(alias = "lvl")]
            level: Option<u8>,
        },
        #[serde(rename_all = "kebab-case")]
        Custom {
            some_value: String,
        },
    }

    #[test]
    fn supports_external_tags() {
        assert_eq!(Mode::default(), Mode::Off);
        assert_eq!(
            load::<Mode>(&[r#"{ "On": { "level": 2 } }"#]).unwrap(),
            Mode::On {
                max_level: 5,
                level: Some(2)
            }
        );
        assert_eq!(
            load::<Mode>(&[r#"{ "On": { "maxLevel": 1, "lvl": 2 } }"#]).unwrap(),
            Mode::On {
                max_level: 1,
                level: Some(2)
            }
        );
        assert_eq!(
            load::<Mode>(&[r#"{ "Custom": { "some-value": "x" } }"#]).unwrap(),
            Mode::Custom {
                some_value: "x".into()
            }
        );
        assert_eq!(
            load::<Mode>(&[r#"{ "Level": 3 }"#]).unwrap(),
            Mode::Level(3)
        );
    }

    // A unit variant holds no type, so it's reported as its name, as in an
    // enum of only unit variants
    #[test]
    fn lists_unit_variants_by_name() {
        let settings = Mode::settings();

        assert_eq!(settings["Off"].type_alias, "Off");
        assert_eq!(settings["Level"].type_alias, "u8");
    }

    // Untagged, where each variant is attempted in order
    #[derive(Clone, Debug, Config, PartialEq)]
    #[serde(untagged)]
    pub enum Dependency {
        #[setting(default)]
        Version(String),
        Detailed {
            version: String,
            #[setting(default = true)]
            optional: bool,
            #[serde(flatten)]
            extra: BTreeMap<String, String>,
        },
    }

    #[derive(Debug, Config)]
    pub struct Manifest {
        #[setting(nested)]
        deps: HashMap<String, Dependency>,
    }

    #[test]
    fn supports_untagged() {
        let config = load::<Manifest>(&[
            r#"{ "deps": { "a": "1", "b": { "version": "2" }, "c": { "version": "3", "optional": false, "features": "x" } } }"#,
        ])
        .unwrap();

        assert_eq!(config.deps["a"], Dependency::Version("1".into()));
        assert_eq!(
            config.deps["b"],
            Dependency::Detailed {
                version: "2".into(),
                optional: true,
                extra: BTreeMap::new(),
            }
        );
        assert_eq!(
            config.deps["c"],
            Dependency::Detailed {
                version: "3".into(),
                optional: false,
                extra: BTreeMap::from_iter([("features".into(), "x".into())]),
            }
        );
    }

    #[test]
    fn lists_each_untagged_failure() {
        let error = serde_json::from_str::<PartialDependency>("[1]")
            .unwrap_err()
            .to_string();

        assert!(error.contains("Version"), "{error}");
        assert!(error.contains("Detailed"), "{error}");
    }

    // A literal union may hold a named variant too
    #[derive(Clone, Debug, Config, PartialEq)]
    pub enum Toggle {
        #[setting(default)]
        Auto,
        #[serde(untagged)]
        Custom {
            #[setting(default = 1)]
            weight: u8,
        },
    }

    #[test]
    fn supports_literal_unions() {
        assert_eq!(load::<Toggle>(&[r#""Auto""#]).unwrap(), Toggle::Auto);
        assert_eq!(
            load::<Toggle>(&[r#"{}"#]).unwrap(),
            Toggle::Custom { weight: 1 }
        );

        let error = serde_json::from_str::<PartialToggle>(r#"{ "nope": 1 }"#)
            .unwrap_err()
            .to_string();

        assert!(error.contains("unknown field `nope`"), "{error}");
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    #[config(allow_unknown_fields)]
    #[serde(tag = "kind", content = "data")]
    pub enum Open {
        #[setting(default)]
        A { value: Option<String> },
    }

    #[test]
    fn supports_allow_unknown_fields_and_adjacent_tags() {
        assert_eq!(
            load::<Open>(&[r#"{ "kind": "A", "data": { "value": "x", "other": 1 } }"#]).unwrap(),
            Open::A {
                value: Some("x".into())
            }
        );
    }

    // Generics are threaded through the struct an untagged variant
    // deserializes through
    #[derive(Clone, Debug, Config, PartialEq)]
    #[serde(untagged)]
    pub enum Generic<T>
    where
        T: Clone
            + Default
            + serde::de::DeserializeOwned
            + schematic::Schematic
            + Serialize
            + PartialEq,
    {
        #[setting(default)]
        Plain(T),
        Wrapped {
            inner: T,
            label: Option<String>,
        },
    }

    #[test]
    fn supports_generics() {
        assert_eq!(load::<Generic<u32>>(&["5"]).unwrap(), Generic::Plain(5));
        assert_eq!(
            load::<Generic<u32>>(&[r#"{ "inner": 5, "label": "x" }"#]).unwrap(),
            Generic::Wrapped {
                inner: 5,
                label: Some("x".into())
            }
        );
    }

    // A setting named like a generated local doesn't shadow it
    #[derive(Clone, Debug, Config, PartialEq)]
    pub enum Shadowing {
        #[setting(default)]
        A {
            #[setting(default = "c")]
            context: String,
            #[setting(validate = not_empty)]
            validate: String,
            r#type: Option<String>,
        },
    }

    #[test]
    fn keeps_generated_locals_apart() {
        assert_eq!(
            load::<Shadowing>(&[r#"{ "A": { "validate": "v", "type": "t" } }"#]).unwrap(),
            Shadowing::A {
                context: "c".into(),
                validate: "v".into(),
                r#type: Some("t".into()),
            }
        );
    }
}

mod named_variant_env {
    use super::*;
    use schematic::merge::append_vec;
    use std::env;

    fn load<T: ConfigTrait>(code: &str) -> Result<T, String> {
        ConfigLoader::<T>::new()
            .code(code, "code.json")
            .unwrap()
            .load()
            .map(|result| result.config)
            .map_err(|error| error.to_full_string())
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    pub struct Child {
        #[setting(env = "NAMED_ENV_TOKEN")]
        token: Option<String>,
        #[setting(
            default = vec!["d".to_owned()],
            merge = append_vec,
            env = "NAMED_ENV_LIST",
            parse_env = schematic::env::split_comma
        )]
        list: Vec<String>,
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    pub enum Auth {
        #[setting(nested)]
        Tuple(Child),
        #[setting(default)]
        Named {
            #[setting(nested)]
            child: Child,
            #[setting(nested)]
            maybe: Option<Child>,
            // Named like the `EnvManager` that finalizing binds
            #[setting(nested)]
            env: Child,
        },
    }

    #[derive(Debug, Config)]
    pub struct Parent {
        #[setting(nested)]
        auth: Auth,
        #[setting(nested)]
        items: Vec<Auth>,
    }

    // The environment layer holds the variables of the variant a source
    // picks, including those of a nested config within it, as a struct's does
    #[test]
    fn reads_the_env_of_a_nested_config() {
        unsafe {
            env::set_var("NAMED_ENV_TOKEN", "env");
            env::set_var("NAMED_ENV_LIST", "e");
        };

        let root = load::<Auth>(r#"{ "Named": { "child": { "list": ["f"] } } }"#);
        let tuple = load::<Auth>(r#"{ "Tuple": {} }"#);
        let parent = load::<Parent>(r#"{ "auth": { "Named": {} }, "items": [{ "Named": {} }] }"#);
        let default = Auth::default();

        unsafe {
            env::remove_var("NAMED_ENV_TOKEN");
            env::remove_var("NAMED_ENV_LIST");
        };

        let from_env = |list: &[&str]| Child {
            token: Some("env".into()),
            list: list.iter().map(|item| item.to_string()).collect(),
        };

        // Over the source and its defaults, and each merged once
        assert_eq!(
            root.unwrap(),
            Auth::Named {
                child: from_env(&["d", "f", "e"]),
                maybe: Some(from_env(&["d", "e"])),
                env: from_env(&["d", "e"]),
            }
        );
        assert_eq!(tuple.unwrap(), Auth::Tuple(from_env(&["d", "e"])));

        let parent = parent.unwrap();
        let named = Auth::Named {
            child: from_env(&["d", "e"]),
            maybe: Some(from_env(&["d", "e"])),
            env: from_env(&["d", "e"]),
        };

        assert_eq!(parent.auth, named);
        assert_eq!(parent.items, vec![named]);

        // Defaults never read the environment
        let empty = Child {
            token: None,
            list: vec!["d".into()],
        };

        assert_eq!(
            default,
            Auth::Named {
                child: empty.clone(),
                maybe: None,
                env: empty,
            }
        );
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    pub struct PortChild {
        #[setting(env = "NAMED_ENV_PORT")]
        port: Option<u16>,
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    pub enum PortSource {
        #[setting(default)]
        Named {
            #[setting(nested)]
            child: PortChild,
        },
    }

    #[test]
    fn names_the_variable_that_fails_to_parse() {
        unsafe { env::set_var("NAMED_ENV_PORT", "abc") };

        let result = load::<PortSource>(r#"{ "Named": {} }"#);

        unsafe { env::remove_var("NAMED_ENV_PORT") };

        assert!(result.unwrap_err().contains("NAMED_ENV_PORT"));
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    pub struct UnitChild {
        #[setting(env = "NAMED_ENV_MAPPED")]
        value: Option<String>,
    }

    #[derive(Default)]
    pub struct AppContext {
        _name: String,
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    #[config(context = AppContext)]
    pub enum Mapped {
        #[setting(default)]
        Named {
            #[setting(nested, context = schematic::context::unit)]
            child: UnitChild,
        },
    }

    // The environment is merged with the context the child takes
    #[test]
    fn reads_the_env_of_a_nested_config_with_a_mapped_context() {
        unsafe { env::set_var("NAMED_ENV_MAPPED", "mapped") };

        let result = load::<Mapped>(r#"{ "Named": {} }"#);

        unsafe { env::remove_var("NAMED_ENV_MAPPED") };

        assert_eq!(
            result.unwrap(),
            Mapped::Named {
                child: UnitChild {
                    value: Some("mapped".into())
                }
            }
        );
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    #[config(env_prefix = "NV_RETRY_")]
    pub struct Retry {
        #[setting(default = 3)]
        attempts: u8,
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    #[config(env_prefix = "NV_SRC_")]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum Source {
        #[setting(default)]
        File {
            #[setting(default = "config.json")]
            path: String,
            #[setting(
                default = vec!["d".to_owned()],
                merge = append_vec,
                parse_env = schematic::env::split_comma
            )]
            tags: Vec<String>,
        },
        Url {
            #[setting(required)]
            address: Option<String>,
            #[setting(env = "NV_URL_TOKEN")]
            token: Option<String>,
            // Shares `NV_SRC_PATH` with `File`, as only one is ever picked
            path: Option<String>,
            #[setting(nested, env_prefix = "NV_URL_RETRY_")]
            retry: Retry,
        },
    }

    #[derive(Debug, Config)]
    pub struct Sources {
        #[setting(nested)]
        source: Source,
        #[setting(nested)]
        items: Vec<Source>,
    }

    // A variant's settings read the variables they derive from the enum's
    // prefix, or name, over every source, but only within the variant that a
    // source or the default picked
    #[test]
    fn reads_the_env_of_the_picked_variant() {
        unsafe {
            env::set_var("NV_SRC_PATH", "env.json");
            env::set_var("NV_SRC_TAGS", "e");
            env::set_var("NV_SRC_ADDRESS", "addr");
            env::set_var("NV_URL_TOKEN", "tok");
            env::set_var("NV_URL_RETRY_ATTEMPTS", "7");
        };

        let file = load::<Source>(r#"{ "type": "file", "tags": ["f"] }"#);
        let url = load::<Source>(r#"{ "type": "url" }"#);
        let default = ConfigLoader::<Source>::new().load();
        let parent = load::<Sources>(r#"{ "items": [{ "type": "url" }, { "type": "file" }] }"#);

        unsafe {
            env::remove_var("NV_SRC_PATH");
            env::remove_var("NV_SRC_TAGS");
            env::remove_var("NV_SRC_ADDRESS");
            env::remove_var("NV_URL_TOKEN");
            env::remove_var("NV_URL_RETRY_ATTEMPTS");
        };

        let file_from_env = |tags: &[&str]| Source::File {
            path: "env.json".into(),
            tags: tags.iter().map(|tag| tag.to_string()).collect(),
        };
        // `address` is required, which the environment satisfies, and the
        // parent's prefix reaches the nested config
        let url_from_env = Source::Url {
            address: Some("addr".into()),
            token: Some("tok".into()),
            path: Some("env.json".into()),
            retry: Retry { attempts: 7 },
        };

        // Each merged once, over the default and the source
        assert_eq!(file.unwrap(), file_from_env(&["d", "f", "e"]));
        assert_eq!(url.unwrap(), url_from_env);
        assert_eq!(default.unwrap().config, file_from_env(&["d", "e"]));

        let parent = parent.unwrap();

        assert_eq!(parent.source, file_from_env(&["d", "e"]));
        assert_eq!(parent.items, vec![url_from_env, file_from_env(&["d", "e"])]);
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    #[config(env_prefix = "NV_REQ_")]
    pub enum RequiredSource {
        #[setting(default)]
        Url {
            #[setting(required)]
            address: Option<String>,
        },
    }

    // Without the variable, a required setting is still reported
    #[test]
    fn reports_a_required_setting_the_env_leaves_out() {
        let error = load::<RequiredSource>(r#"{ "Url": {} }"#).unwrap_err();

        assert!(error.contains("Url.address"), "{error}");
    }

    #[test]
    fn lists_the_env_of_each_variant() {
        let settings = Source::settings();
        let url = settings["url"].nested.as_ref().unwrap();
        let file = settings["file"].nested.as_ref().unwrap();

        assert_eq!(url["address"].env_key.as_deref(), Some("NV_SRC_ADDRESS"));
        assert_eq!(url["token"].env_key.as_deref(), Some("NV_URL_TOKEN"));
        assert_eq!(url["path"].env_key.as_deref(), Some("NV_SRC_PATH"));
        assert_eq!(file["path"].env_key.as_deref(), Some("NV_SRC_PATH"));
        assert_eq!(file["tags"].env_key.as_deref(), Some("NV_SRC_TAGS"));
        assert_eq!(url["retry"].env_key, None);
    }
}

mod env_picked_variant {
    use super::*;
    use std::env;

    #[derive(Clone, Debug, Config, PartialEq)]
    pub struct Redis {
        #[setting(default = 6379, env = "EPV_REDIS_PORT")]
        port: u16,
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    #[config(env_prefix = "EPV_AUTH_")]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum Auth {
        #[setting(default)]
        None,
        #[serde(alias = "tok")]
        Token {
            #[setting(required)]
            token: Option<String>,
            #[setting(default = 60)]
            ttl: u32,
        },
        #[setting(nested)]
        Redis(Redis),
    }

    #[derive(Debug, Config)]
    pub struct App {
        #[setting(nested)]
        auth: Auth,
        #[setting(nested)]
        maybe: Option<Auth>,
        #[setting(nested)]
        items: Vec<Auth>,
    }

    fn load<T: ConfigTrait>(code: Option<&str>) -> Result<schematic::ConfigLoadResult<T>, String> {
        let mut loader = ConfigLoader::<T>::new();

        if let Some(code) = code {
            loader.code(code, "code.json").unwrap();
        }

        loader.load().map_err(|error| error.to_full_string())
    }

    fn token(ttl: u32) -> Auth {
        Auth::Token {
            token: Some("secret".into()),
            ttl,
        }
    }

    // The tag's variable picks a variant, which the environment layer holds
    // with its own variables, and the defaults layer with its defaults, as
    // both are read against it. Every case shares the variables, so they're
    // one test.
    #[test]
    fn picks_the_variant_its_tag_names() {
        unsafe {
            env::set_var("EPV_AUTH_TYPE", "token");
            env::set_var("EPV_AUTH_TOKEN", "secret");
        };

        // Over a source that picked another, or the same, or none
        let other = load::<Auth>(Some(r#"{ "type": "none" }"#));
        let same = load::<Auth>(Some(r#"{ "type": "token", "ttl": 5 }"#));
        let none = load::<Auth>(None);
        // Optional and collected configs are picked too, each item for itself
        let app = load::<App>(Some(
            r#"{ "items": [{ "type": "none" }, { "type": "token", "ttl": 1 }] }"#,
        ));

        unsafe { env::set_var("EPV_AUTH_TYPE", "tok") };
        let alias = load::<Auth>(None);

        unsafe {
            env::set_var("EPV_AUTH_TYPE", "redis");
            env::set_var("EPV_REDIS_PORT", "1");
        };
        let nested = load::<Auth>(None);

        unsafe { env::set_var("EPV_AUTH_TYPE", "nope") };
        let unknown = load::<Auth>(None);

        unsafe {
            env::remove_var("EPV_AUTH_TYPE");
            env::remove_var("EPV_AUTH_TOKEN");
            env::remove_var("EPV_REDIS_PORT");
        };

        let other = other.unwrap();

        assert_eq!(other.config, token(60));
        assert!(matches!(
            other.layers.last().map(|layer| &layer.source),
            Some(schematic::Source::EnvVars)
        ));
        assert_eq!(same.unwrap().config, token(5));
        assert_eq!(none.unwrap().config, token(60));

        let app = app.unwrap().config;

        assert_eq!(app.auth, token(60));
        assert_eq!(app.maybe, Some(token(60)));
        assert_eq!(app.items, vec![token(60), token(1)]);

        assert_eq!(alias.unwrap().config, token(60));
        assert_eq!(nested.unwrap().config, Auth::Redis(Redis { port: 1 }));

        let Err(unknown) = unknown else {
            panic!("expected an unknown variant to fail");
        };

        assert!(unknown.contains("EPV_AUTH_TYPE"), "{unknown}");
        assert!(
            unknown.contains("Unknown variant `nope`, expected one of `none`, `token`, `redis`."),
            "{unknown}"
        );
    }

    // Without the variable, the source's variant stands
    #[derive(Clone, Debug, Config, PartialEq)]
    #[config(env_prefix = "EPV_UNSET_")]
    #[serde(tag = "type")]
    pub enum Unset {
        #[setting(default)]
        A,
        B {
            b: Option<String>,
        },
    }

    #[test]
    fn keeps_the_variant_of_a_source_without_the_variable() {
        let result = load::<Unset>(Some(r#"{ "type": "B" }"#)).unwrap();

        assert_eq!(result.config, Unset::B { b: None });
        assert!(
            !result
                .layers
                .iter()
                .any(|layer| matches!(layer.source, schematic::Source::EnvVars))
        );
    }
}

// The defaults and environment layers are read against what the sources
// picked, so a picked variant, an optional config, and a parent's prefix all
// go through the layers, rather than being applied when finalizing.
mod self_aware_layers {
    use super::*;
    use schematic::merge::append_vec;
    use std::env;

    fn load<T: ConfigTrait>(codes: &[&str]) -> Result<T, String> {
        let mut loader = ConfigLoader::<T>::new();

        for (index, code) in codes.iter().enumerate() {
            loader.code(*code, format!("{index}.json")).unwrap();
        }

        loader
            .load()
            .map(|result| result.config)
            .map_err(|error| error.to_full_string())
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    #[config(env_prefix = "SAL_ONCE_")]
    #[serde(tag = "type")]
    pub enum Picked {
        #[setting(default)]
        A,
        B {
            #[setting(default = vec!["d".to_owned()], merge = append_vec, parse_env = schematic::env::split_comma)]
            list: Vec<String>,
        },
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    #[config(env_prefix = "SAL_ONCE_CHILD_")]
    pub struct OnceChild {
        #[setting(default = vec!["d".to_owned()], merge = append_vec, parse_env = schematic::env::split_comma)]
        list: Vec<String>,
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    pub enum Holds {
        #[setting(default)]
        Nothing,
        #[setting(nested)]
        Child(OnceChild),
        #[setting(nested)]
        Maybe(Option<OnceChild>),
    }

    #[derive(Debug, Config)]
    pub struct OnceParent {
        #[setting(nested)]
        picked: Picked,
        #[setting(nested)]
        maybe: Option<Picked>,
        #[setting(nested)]
        holds: Holds,
        #[setting(nested)]
        holds_maybe: Holds,
        #[setting(nested)]
        child: Option<OnceChild>,
    }

    // Each is in a layer, read once, with the merge applied once between each
    // layer, for a variant a source picked, a config held by one, and an
    // optional config. Every case shares the variables, so they're one test.
    #[test]
    fn applies_defaults_and_env_once() {
        unsafe {
            env::set_var("SAL_ONCE_LIST", "e");
            env::set_var("SAL_ONCE_CHILD_LIST", "e");
        };

        let config = load::<OnceParent>(&[r#"{
            "picked": { "type": "B", "list": ["f"] },
            "maybe": { "type": "B", "list": ["f"] },
            "holds": { "Child": { "list": ["f"] } },
            "holds_maybe": { "Maybe": { "list": ["f"] } },
            "child": { "list": ["f"] }
        }"#]);

        unsafe {
            env::remove_var("SAL_ONCE_LIST");
            env::remove_var("SAL_ONCE_CHILD_LIST");
        };

        let config = config.unwrap();
        let list = vec!["d".to_owned(), "f".into(), "e".into()];

        assert_eq!(config.picked, Picked::B { list: list.clone() });
        assert_eq!(config.maybe, Some(Picked::B { list: list.clone() }));
        assert_eq!(config.holds, Holds::Child(OnceChild { list: list.clone() }));
        assert_eq!(
            config.holds_maybe,
            Holds::Maybe(Some(OnceChild { list: list.clone() }))
        );
        assert_eq!(config.child, Some(OnceChild { list }));
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    #[serde(tag = "type")]
    pub enum Switched {
        #[setting(default)]
        A {
            #[setting(default = 1)]
            a: u8,
        },
        B {
            #[setting(default = 2)]
            b: u8,
        },
    }

    // The defaults are of the variant the layers end on, and go under all of
    // them together, so a source picking another first doesn't drop them
    #[test]
    fn applies_the_defaults_of_a_variant_a_later_source_picks() {
        let config = load::<Switched>(&[r#"{ "type": "B", "b": 5 }"#, r#"{ "type": "A" }"#]);

        assert_eq!(config.unwrap(), Switched::A { a: 1 });
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    #[config(env_prefix = "SAL_OWN_")]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum Store {
        #[setting(default)]
        Memory,
        Disk {
            path: Option<String>,
            #[setting(default = 10)]
            size: u32,
        },
    }

    #[derive(Debug, Config)]
    pub struct HasStore {
        #[setting(nested, env_prefix = "SAL_PARENT_")]
        store: Store,
    }

    // A parent's prefix reaches an enum's tag and its settings, alongside the
    // enum's own, which is read first, as for a struct
    #[test]
    fn reads_a_parent_prefix_within_an_enum() {
        unsafe {
            env::set_var("SAL_PARENT_TYPE", "disk");
            env::set_var("SAL_PARENT_PATH", "/parent");
            env::set_var("SAL_PARENT_SIZE", "5");
            env::set_var("SAL_OWN_SIZE", "7");
        };

        let config = load::<HasStore>(&[]);

        unsafe {
            env::remove_var("SAL_PARENT_TYPE");
            env::remove_var("SAL_PARENT_PATH");
            env::remove_var("SAL_PARENT_SIZE");
            env::remove_var("SAL_OWN_SIZE");
        };

        assert_eq!(
            config.unwrap().store,
            Store::Disk {
                path: Some("/parent".into()),
                size: 7,
            }
        );
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    #[config(env_prefix = "SAL_OPT_")]
    pub enum Cache {
        #[setting(default)]
        Local {
            dir: Option<String>,
            #[setting(default = 1)]
            level: u8,
        },
        Off,
    }

    #[derive(Debug, Config)]
    pub struct HasCache {
        #[setting(nested)]
        cache: Option<Cache>,
    }

    // As with an optional struct, a variable can provide a missing enum, as
    // its default variant, which then has its defaults
    #[test]
    fn provides_a_missing_optional_enum_from_its_default_variant() {
        let without = load::<HasCache>(&[]);

        unsafe { env::set_var("SAL_OPT_DIR", "/tmp") };

        let with = load::<HasCache>(&[]);
        let off = load::<HasCache>(&[r#"{ "cache": "Off" }"#]);

        unsafe { env::remove_var("SAL_OPT_DIR") };

        assert_eq!(without.unwrap().cache, None);
        assert_eq!(
            with.unwrap().cache,
            Some(Cache::Local {
                dir: Some("/tmp".into()),
                level: 1,
            })
        );
        assert_eq!(off.unwrap().cache, Some(Cache::Off));
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    #[config(env_prefix = "SAL_MIX_")]
    pub struct MixChild {
        #[setting(default = 2)]
        level: u8,
        name: Option<String>,
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    pub enum Mixed {
        #[setting(default)]
        Both(#[setting(nested)] MixChild, bool),
        Neither,
    }

    #[derive(Debug, Config)]
    pub struct HasMixed {
        #[setting(nested)]
        mixed: Mixed,
    }

    // A layer would have to restate the plain value beside the config, which
    // would replace it, so the config reads its variables when finalized
    #[test]
    fn reads_the_env_of_a_config_beside_a_plain_value_when_finalized() {
        unsafe { env::set_var("SAL_MIX_NAME", "env") };

        let config = load::<HasMixed>(&[r#"{ "mixed": { "Both": [{ "name": "src" }, true] } }"#]);

        unsafe { env::remove_var("SAL_MIX_NAME") };

        assert_eq!(
            config.unwrap().mixed,
            Mixed::Both(
                MixChild {
                    level: 2,
                    name: Some("env".into()),
                },
                true
            )
        );
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    pub struct Item {
        #[setting(default = 1)]
        weight: u8,
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    #[config(env_prefix = "SAL_SAME_")]
    #[serde(tag = "type", content = "value", rename_all = "snake_case")]
    pub enum Group {
        #[setting(default)]
        Empty,
        #[setting(nested)]
        Many(Vec<Item>),
    }

    // A tag's variable naming the variant a source already picked leaves it
    // as the source gave it, rather than replacing it with an empty one
    #[test]
    fn keeps_the_values_of_a_variant_its_tag_already_names() {
        unsafe { env::set_var("SAL_SAME_TYPE", "many") };

        let same = load::<Group>(&[r#"{ "type": "many", "value": [{}, { "weight": 3 }] }"#]);
        let other = load::<Group>(&[r#"{ "type": "empty" }"#]);

        unsafe { env::remove_var("SAL_SAME_TYPE") };

        assert_eq!(
            same.unwrap(),
            Group::Many(vec![Item { weight: 1 }, Item { weight: 3 }])
        );
        assert_eq!(other.unwrap(), Group::Many(vec![]));
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    pub struct Linked {
        #[setting(default = 1)]
        weight: u8,
        #[setting(nested)]
        next: Option<Box<Self>>,
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    pub enum Tree {
        #[setting(default)]
        Leaf {
            #[setting(default = 1)]
            weight: u8,
        },
        #[setting(nested)]
        Node(Box<Self>),
        #[setting(nested)]
        Maybe(Option<Box<Self>>),
    }

    // The defaults follow a recursive config as deep as the sources go
    #[test]
    fn applies_defaults_at_every_depth_of_a_recursive_config() {
        let linked = load::<Linked>(&[r#"{ "next": { "next": {} } }"#]).unwrap();

        assert_eq!(linked.weight, 1);
        assert_eq!(linked.next.as_ref().unwrap().weight, 1);
        assert_eq!(linked.next.unwrap().next.unwrap().weight, 1);

        let tree = load::<Tree>(&[r#"{ "Node": { "Maybe": { "Leaf": {} } } }"#]).unwrap();

        assert_eq!(
            tree,
            Tree::Node(Box::new(Tree::Maybe(Some(Box::new(Tree::Leaf {
                weight: 1
            })))))
        );
        assert_eq!(Tree::default(), Tree::Leaf { weight: 1 });
    }

    #[derive(Clone, Debug, Config, PartialEq)]
    pub enum Mode {
        #[setting(default = "auto")]
        Named(String),
        Off,
    }

    #[derive(Debug, Config)]
    pub struct HasMode {
        #[setting(nested)]
        mode: Mode,
    }

    // The layers are merged from the first, so the enum's type default never
    // goes over the value its defaults hold
    #[test]
    fn applies_a_default_variant_value_when_nested() {
        assert_eq!(
            load::<HasMode>(&[]).unwrap().mode,
            Mode::Named("auto".into())
        );
        assert_eq!(HasMode::default().mode, Mode::Named("auto".into()));
    }
}
