//! Exercises the standalone `#[derive(Schematic)]` as emitted by
//! `schematic_core`. Unlike `Config`, this emits one impl and no partial.
#![allow(dead_code)]

use schematic::{Schema, SchemaBuilder, SchemaType};
use schematic_macros::Schematic;
use std::collections::HashMap;

fn build<T: schematic::Schematic>() -> Schema {
    SchemaBuilder::build_root::<T>()
}

fn field_names(schema: &Schema) -> Vec<&str> {
    let SchemaType::Struct(inner) = &schema.ty else {
        panic!("expected a struct, got {:?}", schema.ty);
    };

    inner.fields.keys().map(|key| key.as_str()).collect()
}

mod structs {
    use super::*;

    /// Some docs.
    #[derive(Schematic)]
    struct Basic {
        name: String,
        count: usize,
        tags: Vec<String>,
    }

    #[test]
    fn builds_a_named_struct() {
        let schema = build::<Basic>();

        assert_eq!(schema.name.as_deref(), Some("Basic"));
        assert_eq!(field_names(&schema), vec!["name", "count", "tags"]);
    }

    #[test]
    fn carries_the_container_comment() {
        assert_eq!(build::<Basic>().description.as_deref(), Some("Some docs."));
    }

    // No partial type is emitted, so nothing named `PartialBasic` exists.
    // This is the difference from `#[derive(Config)]`.
    #[test]
    fn does_not_emit_a_partial() {
        // Compiles only because `Basic` alone implements `Schematic`
        fn assert_schematic<T: schematic::Schematic>() {}
        assert_schematic::<Basic>();
    }

    #[derive(Schematic)]
    #[schematic(rename = "Renamed")]
    struct HasRename {
        value: bool,
    }

    #[test]
    fn honours_a_container_rename() {
        assert_eq!(build::<HasRename>().name.as_deref(), Some("Renamed"));
    }

    #[derive(Schematic)]
    struct Tuple(String, usize);

    #[test]
    fn builds_a_tuple_struct() {
        let schema = build::<Tuple>();

        assert!(matches!(schema.ty, SchemaType::Tuple(_)));
    }

    #[derive(Schematic)]
    struct Single(String);

    // A single unnamed value is transparent
    #[test]
    fn builds_a_newtype_struct() {
        assert_eq!(build::<Single>().ty, SchemaType::String(Box::default()));
    }
}

mod enums {
    use super::*;

    #[derive(Schematic)]
    enum Value {
        Text(String),
        Number(usize),
    }

    #[test]
    fn builds_a_union() {
        let schema = build::<Value>();

        let SchemaType::Union(inner) = &schema.ty else {
            panic!("expected a union");
        };

        assert_eq!(inner.variants_types.len(), 2);
    }

    #[derive(Schematic)]
    enum Level {
        Low,
        High,
    }

    #[test]
    fn builds_an_enumerable_from_units() {
        let schema = build::<Level>();

        let SchemaType::Enum(inner) = &schema.ty else {
            panic!("expected an enum");
        };

        assert_eq!(inner.values.len(), 2);
        assert_eq!(inner.variants.as_ref().unwrap().len(), 2);
    }
}

mod generics {
    use super::*;

    #[derive(Schematic)]
    struct Wrapper<T: schematic::Schematic> {
        inner: T,
        label: String,
    }

    // The impl has to carry the type parameters through, otherwise a
    // generic type cannot derive at all.
    #[test]
    fn supports_type_parameters() {
        let schema = build::<Wrapper<bool>>();

        assert_eq!(field_names(&schema), vec!["inner", "label"]);

        let SchemaType::Struct(inner) = &schema.ty else {
            panic!("expected a struct");
        };

        assert_eq!(
            inner.fields["inner"].schema.ty,
            SchemaType::Boolean(Box::default())
        );
    }

    #[derive(Schematic)]
    struct Bounded<T>
    where
        T: schematic::Schematic + Clone,
    {
        value: T,
    }

    // Schemas are keyed by name alone, so every instantiation has to resolve
    // to a distinct one or they collapse into a single definition.
    #[test]
    fn names_include_the_type_arguments() {
        assert_eq!(
            build::<Wrapper<bool>>().name.as_deref(),
            Some("WrapperBool")
        );
        assert_eq!(
            build::<Wrapper<String>>().name.as_deref(),
            Some("WrapperString")
        );
        assert_eq!(
            build::<Wrapper<Vec<String>>>().name.as_deref(),
            Some("WrapperVecString")
        );
    }

    #[derive(Schematic)]
    struct Named {
        value: bool,
    }

    // A type argument that names itself is used as-is
    #[test]
    fn names_use_the_arguments_own_schema_name() {
        assert_eq!(
            build::<Wrapper<Named>>().name.as_deref(),
            Some("WrapperNamed")
        );
    }

    #[derive(Schematic)]
    struct TwoParams<T: schematic::Schematic, U: schematic::Schematic> {
        first: T,
        second: U,
    }

    #[test]
    fn names_append_every_type_argument() {
        assert_eq!(
            build::<TwoParams<bool, Named>>().name.as_deref(),
            Some("TwoParamsBoolNamed")
        );
    }

    // The collision this prevents: without the arguments in the name, both
    // instantiations claim "Wrapper" and the generator rejects the second.
    #[test]
    fn instantiations_coexist_in_a_generator() {
        use schematic::schema::SchemaGenerator;

        let mut generator = SchemaGenerator::default();

        generator.add::<Wrapper<bool>>();
        generator.add::<Wrapper<String>>();

        assert!(generator.schemas.contains_key("WrapperBool"));
        assert!(generator.schemas.contains_key("WrapperString"));
    }

    #[test]
    fn supports_where_clauses() {
        let schema = build::<Bounded<usize>>();

        let SchemaType::Struct(inner) = &schema.ty else {
            panic!("expected a struct");
        };

        assert!(matches!(
            inner.fields["value"].schema.ty,
            SchemaType::Integer(_)
        ));
    }
}

mod nested_types {
    use super::*;

    #[derive(Schematic)]
    struct Leaf {
        value: String,
    }

    #[derive(Schematic)]
    struct Tree {
        leaf: Leaf,
        leaves: Vec<Leaf>,
        keyed: HashMap<String, Leaf>,
        maybe: Option<Leaf>,
    }

    #[test]
    fn infers_nested_schemas() {
        let schema = build::<Tree>();
        let SchemaType::Struct(inner) = &schema.ty else {
            panic!("expected a struct");
        };

        assert_eq!(
            inner.fields["leaf"].schema.name.as_deref(),
            Some("Leaf"),
            "a bare nested type keeps its name"
        );
        assert!(matches!(
            inner.fields["leaves"].schema.ty,
            SchemaType::Array(_)
        ));
        assert!(matches!(
            inner.fields["keyed"].schema.ty,
            SchemaType::Object(_)
        ));
        assert!(inner.fields["maybe"].schema.ty.is_nullable());
    }

    #[derive(Schematic)]
    struct Recursive {
        children: Vec<Recursive>,
    }

    #[test]
    fn resolves_cycles_to_a_reference() {
        let schema = build::<Recursive>();
        let SchemaType::Struct(inner) = &schema.ty else {
            panic!("expected a struct");
        };
        let SchemaType::Array(array) = &inner.fields["children"].schema.ty else {
            panic!("expected an array");
        };

        assert_eq!(
            array.items_type.ty,
            SchemaType::Reference {
                name: "Recursive".into(),
                partial: false,
            }
        );
    }
}

mod field_metadata {
    use super::*;

    #[derive(Schematic)]
    struct Documented {
        /// A documented field.
        described: String,
        #[schema(rename = "renamed")]
        original: String,
        #[schema(exclude)]
        hidden: bool,
    }

    #[test]
    fn carries_field_metadata() {
        let schema = build::<Documented>();
        let SchemaType::Struct(inner) = &schema.ty else {
            panic!("expected a struct");
        };

        assert_eq!(
            inner.fields["described"].comment.as_deref(),
            Some("A documented field.")
        );
        assert!(inner.fields.contains_key("renamed"));
        assert!(!inner.fields.contains_key("original"));
        assert!(!inner.fields.contains_key("hidden"));
    }
}

// A container `#[serde(default)]` lets serde fall back per field, so every
// field may be omitted from the input and renders as optional
mod container_serde_default {
    use super::*;

    #[derive(Schematic)]
    #[serde(default)]
    pub struct AllOptional {
        pub name: String,
        pub count: usize,
    }

    #[derive(Schematic)]
    pub struct NoneOptional {
        pub name: String,
        pub count: usize,
    }

    fn optional_fields<T: schematic::Schematic>() -> Vec<(String, bool)> {
        let schema = SchemaBuilder::build_root::<T>();

        let SchemaType::Struct(inner) = schema.ty else {
            panic!("expected a struct");
        };

        inner
            .fields
            .iter()
            .map(|(name, field)| (name.to_owned(), field.optional))
            .collect()
    }

    #[test]
    fn a_container_default_makes_every_field_optional() {
        assert_eq!(
            optional_fields::<AllOptional>(),
            vec![("name".into(), true), ("count".into(), true)]
        );
    }

    #[test]
    fn without_it_fields_stay_required() {
        assert_eq!(
            optional_fields::<NoneOptional>(),
            vec![("name".into(), false), ("count".into(), false)]
        );
    }
}
