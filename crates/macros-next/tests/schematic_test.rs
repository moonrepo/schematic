//! Exercises the standalone `#[derive(Schematic)]` as emitted by
//! `schematic_core`. Unlike `Config`, this emits one impl and no partial.
#![allow(dead_code)]

use schematic::{Schema, SchemaBuilder, SchemaType};
use schematic_macros_next::Schematic;
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
