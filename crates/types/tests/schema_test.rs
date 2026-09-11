#![allow(dead_code)]

use schematic_types::*;

mod references {
    use super::*;

    // `SchemaType` is internally tagged, which serde cannot apply to a
    // newtype variant holding a plain string, so `Reference` is a struct
    // variant. Every recursive config depends on this serializing.
    #[test]
    fn serializes_a_reference() {
        let schema = Schema::reference("Foo");

        let json = serde_json::to_string(&schema).unwrap();

        assert_eq!(json, r#"{"ty":{"type":"Reference","name":"Foo"}}"#);
        assert_eq!(serde_json::from_str::<Schema>(&json).unwrap(), schema);
    }

    #[test]
    fn serializes_a_nested_reference() {
        let schema = Schema::object(ObjectType::new(
            Schema::string(StringType::default()),
            Schema::reference("Cycle"),
        ));

        let json = serde_json::to_string(&schema).unwrap();

        assert_eq!(serde_json::from_str::<Schema>(&json).unwrap(), schema);
    }

    pub struct Cycle;

    impl Schematic for Cycle {
        fn schema_name() -> Option<String> {
            Some("Cycle".into())
        }

        fn build_schema(mut schema: SchemaBuilder) -> Schema {
            schema.structure(StructType::new([(
                "inner".into(),
                schema.infer::<Vec<Cycle>>(),
            )]))
        }
    }

    #[test]
    fn round_trips_a_cyclic_schema() {
        let schema = SchemaBuilder::build_root::<Cycle>();
        let json = serde_json::to_string(&schema).unwrap();

        assert_eq!(serde_json::from_str::<Schema>(&json).unwrap(), schema);
    }

    #[test]
    fn is_reference() {
        assert!(Schema::reference("Foo").is_reference());
        assert!(!Schema::null().is_reference());
    }

    #[test]
    fn displays_the_name() {
        let mut schema = Schema::reference("Foo");

        assert_eq!(schema.to_string(), "Foo");

        // A named reference renders its name, not the inner value
        schema.set_name("Bar");

        assert_eq!(schema.to_string(), "Bar");
    }
}

mod unsized_wrappers {
    use super::*;
    use std::borrow::Cow;
    use std::rc::Rc;
    use std::sync::Arc;

    // Partials keep `Box`/`Arc`/`Rc` when the inner type is unsized, so
    // those wrappers must accept `?Sized` for the partial to derive a schema.
    #[test]
    fn builds_through_unsized_wrappers() {
        assert_eq!(
            SchemaBuilder::build_root::<Box<str>>().ty,
            SchemaType::String(Box::default())
        );
        assert_eq!(
            SchemaBuilder::build_root::<Arc<str>>().ty,
            SchemaType::String(Box::default())
        );
        assert_eq!(
            SchemaBuilder::build_root::<Rc<str>>().ty,
            SchemaType::String(Box::default())
        );
        assert_eq!(
            SchemaBuilder::build_root::<Cow<'_, str>>().ty,
            SchemaType::String(Box::default())
        );
    }

    #[test]
    fn builds_through_unsized_slices() {
        let expected = SchemaType::Array(Box::new(ArrayType::new(Schema::string(
            StringType::default(),
        ))));

        assert_eq!(SchemaBuilder::build_root::<Box<[String]>>().ty, expected);
        assert_eq!(SchemaBuilder::build_root::<Arc<[String]>>().ty, expected);
        assert_eq!(SchemaBuilder::build_root::<[String]>().ty, expected);
        assert_eq!(SchemaBuilder::build_root::<&[String]>().ty, expected);
    }
}

mod defaults {
    use super::*;

    #[test]
    fn sets_and_gets_on_primitives() {
        for mut schema in [
            Schema::boolean(BooleanType::default()),
            Schema::float(FloatType::new_kind(FloatKind::F64)),
            Schema::integer(IntegerType::new_kind(IntegerKind::Usize)),
            Schema::string(StringType::default()),
        ] {
            let value = LiteralValue::String("abc".into());

            schema.set_default(value.clone());

            assert_eq!(schema.get_default(), Some(&value));
        }
    }

    // `Option<T>` builds a union, which holds no value of its own, so the
    // default has to reach the non-null variant.
    #[test]
    fn sets_and_gets_through_an_option() {
        let mut schema = SchemaBuilder::build_root::<Option<usize>>();
        let value = LiteralValue::UInt(123);

        schema.set_default(value.clone());

        assert_eq!(schema.get_default(), Some(&value));
    }

    #[test]
    fn sets_through_a_union_honouring_default_index() {
        let mut schema = Schema::union(UnionType::from_schemas(
            [
                Schema::string(StringType::default()),
                Schema::integer(IntegerType::new_kind(IntegerKind::Usize)),
            ],
            Some(1),
        ));
        let value = LiteralValue::UInt(7);

        schema.set_default(value.clone());

        assert_eq!(schema.get_default(), Some(&value));

        // It landed on the indexed variant, not the first one
        let SchemaType::Union(inner) = &schema.ty else {
            panic!("expected a union");
        };

        assert_eq!(inner.variants_types[0].get_default(), None);
        assert_eq!(inner.variants_types[1].get_default(), Some(&value));
    }

    #[test]
    fn sets_and_gets_on_an_enum() {
        let mut schema = Schema::enumerable(EnumType::new([
            LiteralValue::String("a".into()),
            LiteralValue::String("b".into()),
        ]));
        let value = LiteralValue::String("b".into());

        schema.set_default(value.clone());

        assert_eq!(schema.get_default(), Some(&value));
    }

    #[test]
    fn keeps_an_existing_enum_default_when_the_value_is_unknown() {
        let mut schema = Schema::enumerable(EnumType {
            default_index: Some(0),
            values: vec![LiteralValue::String("a".into())],
            variants: None,
        });

        schema.set_default(LiteralValue::String("nope".into()));

        assert_eq!(
            schema.get_default(),
            Some(&LiteralValue::String("a".into()))
        );
    }
}

mod enum_variants {
    use super::*;

    fn named(name: &str, ty: SchemaType) -> Schema {
        Schema {
            name: Some(name.into()),
            ty,
            ..Default::default()
        }
    }

    fn literal(name: &str) -> Schema {
        named(
            name,
            SchemaType::Literal(Box::new(LiteralType::new(LiteralValue::String(
                name.into(),
            )))),
        )
    }

    // A `#[setting(null)]` variant carries no literal, so `values` is
    // shorter than `variants` and `default_index` must resolve through
    // the variants to stay aligned.
    #[test]
    fn resolves_the_default_past_a_valueless_variant() {
        let ty = EnumType::from_schemas(
            [named("C", SchemaType::Null), literal("A"), literal("B")],
            Some(2),
        );

        assert_eq!(ty.variants.as_ref().unwrap().len(), 3);
        assert_eq!(ty.values.len(), 2);

        assert_eq!(
            Schema::enumerable(ty).get_default(),
            Some(&LiteralValue::String("B".into()))
        );
    }

    #[test]
    fn returns_no_default_when_the_variant_has_no_value() {
        let ty = EnumType::from_schemas([named("C", SchemaType::Null), literal("A")], Some(0));

        assert_eq!(Schema::enumerable(ty).get_default(), None);
    }

    #[test]
    fn resolves_the_default_from_values_without_variants() {
        let ty = EnumType {
            default_index: Some(1),
            values: vec![
                LiteralValue::String("a".into()),
                LiteralValue::String("b".into()),
            ],
            variants: None,
        };

        assert_eq!(
            Schema::enumerable(ty).get_default(),
            Some(&LiteralValue::String("b".into()))
        );
    }

    #[test]
    fn sets_the_default_by_variant_value() {
        let mut ty = EnumType::from_schemas([literal("A"), literal("B")], None);

        ty.set_default(LiteralValue::String("B".into()));

        assert_eq!(ty.default_index, Some(1));
    }

    #[test]
    #[should_panic(expected = "Enum variant schemas require a name")]
    fn panics_when_a_variant_is_unnamed() {
        EnumType::from_schemas(
            [Schema::literal_value(LiteralValue::String("A".into()))],
            None,
        );
    }
}

mod nullable {
    use super::*;

    #[test]
    fn inference_marks_options_as_nullable() {
        let schema = SchemaBuilder::build_root::<Option<String>>();

        assert!(schema.nullable);
        assert!(schema.ty.is_nullable());
    }

    #[test]
    fn round_trips_through_serde() {
        let mut schema = Schema::string(StringType::default());
        schema.nullify();

        let json = serde_json::to_string(&schema).unwrap();

        assert!(json.contains(r#""nullable":true"#));
        assert_eq!(serde_json::from_str::<Schema>(&json).unwrap(), schema);
    }

    #[test]
    fn omits_the_flag_when_not_nullable() {
        let json = serde_json::to_string(&Schema::string(StringType::default())).unwrap();

        assert!(!json.contains("nullable"));
    }

    #[test]
    fn nullify_is_idempotent() {
        let mut schema = Schema::string(StringType::default());

        schema.nullify();
        let once = schema.clone();

        schema.nullify();

        assert_eq!(schema, once);
    }

    // Inference builds a nullable union without raising the flag, so
    // `nullify` has to inspect the type, not just the flag.
    #[test]
    fn nullify_does_not_re_wrap_an_inferred_option() {
        let mut schema = SchemaBuilder::build_root::<Option<String>>();
        let before = schema.clone();

        schema.nullify();

        assert_eq!(schema, before);
    }

    #[test]
    fn nullify_does_not_re_wrap_a_named_nullable_union() {
        let mut schema = Schema::union(UnionType::new_any([
            Schema::string(StringType::default()),
            Schema::null(),
        ]));
        schema.set_name("MyUnion");

        schema.nullify();

        let SchemaType::Union(inner) = &schema.ty else {
            panic!("expected a union");
        };

        assert_eq!(inner.variants_types.len(), 2);
        assert!(!inner.variants_types[0].ty.is_nullable());
    }

    #[test]
    fn nullify_wraps_a_named_non_null_union() {
        let mut schema = Schema::union(UnionType::new_any([
            Schema::string(StringType::default()),
            Schema::integer(IntegerType::new_kind(IntegerKind::Usize)),
        ]));
        schema.set_name("MyUnion");

        schema.nullify();

        let SchemaType::Union(inner) = &schema.ty else {
            panic!("expected a union");
        };

        // The named union is preserved as a distinct variant
        assert_eq!(inner.variants_types.len(), 2);
        assert_eq!(inner.variants_types[0].name.as_deref(), Some("MyUnion"));
        assert!(inner.variants_types[1].is_null());
    }

    #[test]
    fn nullify_adds_null_to_an_unnamed_union() {
        let mut schema = Schema::union(UnionType::new_any([
            Schema::string(StringType::default()),
            Schema::integer(IntegerType::new_kind(IntegerKind::Usize)),
        ]));

        schema.nullify();

        let SchemaType::Union(inner) = &schema.ty else {
            panic!("expected a union");
        };

        assert_eq!(inner.variants_types.len(), 3);
        assert!(inner.has_null());
    }
}

mod partialize {
    use super::*;

    fn structure() -> Schema {
        Schema::structure(StructType::default())
    }

    fn is_partial(schema: &Schema) -> bool {
        match &schema.ty {
            SchemaType::Struct(inner) => inner.partial,
            SchemaType::Union(inner) => inner.partial,
            SchemaType::Reference { partial, .. } => *partial,
            _ => false,
        }
    }

    #[test]
    fn marks_a_struct() {
        let mut schema = structure();
        schema.partialize();

        assert!(is_partial(&schema));
    }

    #[test]
    fn recurses_into_arrays() {
        let mut schema = Schema::array(ArrayType::new(structure()));
        schema.partialize();

        let SchemaType::Array(inner) = &schema.ty else {
            panic!("expected an array");
        };

        assert!(is_partial(&inner.items_type));
    }

    #[test]
    fn recurses_into_object_values() {
        let mut schema = Schema::object(ObjectType::new(
            Schema::string(StringType::default()),
            structure(),
        ));
        schema.partialize();

        let SchemaType::Object(inner) = &schema.ty else {
            panic!("expected an object");
        };

        assert!(is_partial(&inner.value_type));
        // Keys are always scalars, so they are left alone
        assert!(!is_partial(&inner.key_type));
    }

    #[test]
    fn recurses_into_tuples() {
        let mut schema = Schema::tuple(TupleType::new([structure(), structure()]));
        schema.partialize();

        let SchemaType::Tuple(inner) = &schema.ty else {
            panic!("expected a tuple");
        };

        assert!(inner.items_types.iter().all(|item| is_partial(item)));
    }

    // A cycle resolves to a reference, which must point at the partial type
    // once partialized, or it names a type that was never rendered.
    #[test]
    fn marks_a_reference() {
        let mut schema = Schema::reference("Nested");
        schema.partialize();

        assert!(is_partial(&schema));
    }

    #[test]
    fn marks_a_reference_within_a_collection() {
        let mut schema = Schema::array(ArrayType::new(Schema::reference("Nested")));
        schema.partialize();

        let SchemaType::Array(inner) = &schema.ty else {
            panic!("expected an array");
        };

        assert!(is_partial(&inner.items_type));
    }

    #[test]
    fn recurses_into_nullable_unions() {
        let mut schema = Schema::union(UnionType::new_any([structure(), Schema::null()]));
        schema.partialize();

        let SchemaType::Union(inner) = &schema.ty else {
            panic!("expected a union");
        };

        assert!(inner.partial);
        assert!(is_partial(&inner.variants_types[0]));
    }

    #[test]
    fn round_trips_the_reference_flag() {
        let mut schema = Schema::reference("Nested");
        schema.partialize();

        let json = serde_json::to_string(&schema).unwrap();

        assert!(json.contains(r#""partial":true"#));
        assert_eq!(serde_json::from_str::<Schema>(&json).unwrap(), schema);
    }

    #[test]
    fn omits_the_reference_flag_when_not_partial() {
        let json = serde_json::to_string(&Schema::reference("Nested")).unwrap();

        assert!(!json.contains("partial"));
    }
}

mod nonnull {
    use super::*;

    #[test]
    fn returns_itself_for_a_plain_type() {
        let schema = Schema::string(StringType::default());

        assert_eq!(schema.get_nonnull_schema(), Some(&schema));
    }

    #[test]
    fn returns_nothing_for_a_null() {
        assert_eq!(Schema::null().get_nonnull_schema(), None);
    }

    #[test]
    fn finds_the_non_null_variant() {
        let schema = SchemaBuilder::build_root::<Option<String>>();
        let inner = schema.get_nonnull_schema().unwrap();

        assert_eq!(inner.ty, SchemaType::String(Box::default()));
    }

    // A variant may itself be a union, so taking the first non-null variant
    // is not enough — it has to resolve.
    #[test]
    fn resolves_through_a_nested_union() {
        let nested = Schema::union(UnionType::new_any([Schema::null(), Schema::null()]));
        let schema = Schema::union(UnionType::new_any([
            nested,
            Schema::string(StringType::default()),
        ]));

        let inner = schema.get_nonnull_schema().unwrap();

        assert_eq!(inner.ty, SchemaType::String(Box::default()));
    }

    #[test]
    fn returns_nothing_when_every_variant_is_null() {
        let nested = Schema::union(UnionType::new_any([Schema::null(), Schema::null()]));
        let schema = Schema::union(UnionType::new_any([nested, Schema::null()]));

        assert_eq!(schema.get_nonnull_schema(), None);
    }
}

mod reports_application {
    use super::*;

    #[test]
    fn add_field_reports_whether_it_applied() {
        let mut schema = Schema::structure(StructType::default());

        assert!(schema.add_field("a", Schema::null()));

        let mut schema = Schema::string(StringType::default());

        assert!(!schema.add_field("a", Schema::null()));
    }

    #[test]
    fn set_default_reports_whether_it_applied() {
        let value = LiteralValue::String("abc".into());

        assert!(Schema::string(StringType::default()).set_default(value.clone()));
        assert!(SchemaBuilder::build_root::<Option<String>>().set_default(value.clone()));

        // Nowhere to put a default
        assert!(!Schema::structure(StructType::default()).set_default(value.clone()));
        assert!(!Schema::array(ArrayType::new(Schema::null())).set_default(value.clone()));
        assert!(!Schema::null().set_default(value.clone()));

        // The enum does not declare this value
        assert!(!Schema::enumerable(EnumType::new([LiteralValue::Bool(true)])).set_default(value));
    }
}

// Every type below is asserted against what serde ACTUALLY emits, which is
// how `Duration` was caught claiming to be a string while encoding as a map.
mod std_coverage {
    use super::*;
    use std::collections::{BinaryHeap, LinkedList, VecDeque};
    use std::net::{IpAddr, SocketAddr, SocketAddrV4, SocketAddrV6};
    use std::num::{NonZeroI32, NonZeroU8, NonZeroUsize};
    use std::ops::{Range, RangeInclusive};
    use std::time::{Duration, SystemTime};

    fn ty<T: Schematic + ?Sized>() -> SchemaType {
        SchemaBuilder::build_root::<T>().ty
    }

    fn array_of(inner: SchemaType) -> SchemaType {
        SchemaType::Array(Box::new(ArrayType::new(Schema::new(inner))))
    }

    fn integer(kind: IntegerKind) -> SchemaType {
        SchemaType::Integer(Box::new(IntegerType::new_kind(kind)))
    }

    fn string_of(format: &str) -> SchemaType {
        SchemaType::String(Box::new(StringType {
            format: Some(format.into()),
            ..StringType::default()
        }))
    }

    #[test]
    fn collections_are_arrays() {
        let expected = array_of(integer(IntegerKind::U8));

        assert_eq!(ty::<VecDeque<u8>>(), expected);
        assert_eq!(ty::<LinkedList<u8>>(), expected);
        assert_eq!(ty::<BinaryHeap<u8>>(), expected);
    }

    #[test]
    fn non_zero_integers_keep_their_kind() {
        assert_eq!(
            ty::<NonZeroUsize>(),
            SchemaType::Integer(Box::new(IntegerType {
                min: Some(1),
                kind: IntegerKind::Usize,
                ..IntegerType::default()
            }))
        );
        assert_eq!(
            ty::<NonZeroU8>(),
            SchemaType::Integer(Box::new(IntegerType {
                min: Some(1),
                kind: IntegerKind::U8,
                ..IntegerType::default()
            }))
        );
        // Signed cannot express "not zero" as a bound
        assert_eq!(
            ty::<NonZeroI32>(),
            SchemaType::Integer(Box::new(IntegerType {
                kind: IntegerKind::I32,
                ..IntegerType::default()
            }))
        );
    }

    #[test]
    fn network_addresses_are_strings() {
        assert_eq!(ty::<IpAddr>(), string_of("ip"));
        assert_eq!(ty::<SocketAddr>(), string_of("socket-addr"));
        assert_eq!(ty::<SocketAddrV4>(), string_of("socket-addr"));
        assert_eq!(ty::<SocketAddrV6>(), string_of("socket-addr"));
    }

    fn field_names(ty: &SchemaType) -> Vec<&str> {
        let SchemaType::Struct(inner) = ty else {
            panic!("expected a struct, got {ty:?}");
        };

        inner.fields.keys().map(|key| key.as_str()).collect()
    }

    #[test]
    fn duration_models_the_map_serde_emits() {
        let ty = ty::<Duration>();

        assert_eq!(field_names(&ty), vec!["secs", "nanos"]);

        // Matches `{"secs":3,"nanos":500}`
        let value = serde_json::to_value(Duration::new(3, 500)).unwrap();
        let SchemaType::Struct(inner) = &ty else {
            panic!("expected a struct");
        };

        for key in value.as_object().unwrap().keys() {
            assert!(inner.fields.contains_key(key), "missing `{key}`");
        }
    }

    #[test]
    fn system_time_models_the_map_serde_emits() {
        let ty = ty::<SystemTime>();

        assert_eq!(
            field_names(&ty),
            vec!["secs_since_epoch", "nanos_since_epoch"]
        );

        let value = serde_json::to_value(SystemTime::UNIX_EPOCH).unwrap();
        let SchemaType::Struct(inner) = &ty else {
            panic!("expected a struct");
        };

        for key in value.as_object().unwrap().keys() {
            assert!(inner.fields.contains_key(key), "missing `{key}`");
        }
    }

    #[test]
    fn ranges_model_the_map_serde_emits() {
        for ty in [ty::<Range<usize>>(), ty::<RangeInclusive<usize>>()] {
            assert_eq!(field_names(&ty), vec!["start", "end"]);
        }

        let value = serde_json::to_value(1usize..5).unwrap();

        assert_eq!(value.as_object().unwrap().len(), 2);
    }
}

mod field_ordering {
    use super::*;

    // The model keeps what the user declared...
    #[test]
    fn preserves_insertion_order() {
        let ty = StructType::new([
            ("zebra".to_string(), Schema::null()),
            ("apple".to_string(), Schema::null()),
            ("mango".to_string(), Schema::null()),
        ]);

        assert_eq!(
            ty.fields.keys().collect::<Vec<_>>(),
            vec!["zebra", "apple", "mango"]
        );
    }

    // ...while renderers read through this, so generated output stays
    // alphabetical no matter how the source type is ordered.
    #[test]
    fn sorted_fields_is_alphabetical() {
        let ty = StructType::new([
            ("zebra".to_string(), Schema::null()),
            ("apple".to_string(), Schema::null()),
            ("mango".to_string(), Schema::null()),
        ]);

        assert_eq!(
            ty.sorted_fields().into_keys().collect::<Vec<_>>(),
            vec!["apple", "mango", "zebra"]
        );

        // The declared order is untouched by reading it
        assert_eq!(
            ty.fields.keys().collect::<Vec<_>>(),
            vec!["zebra", "apple", "mango"]
        );
    }

    #[test]
    fn sorted_fields_sees_every_field() {
        let ty = StructType::new([
            ("b".to_string(), Schema::null()),
            ("a".to_string(), Schema::null()),
        ]);

        assert_eq!(ty.sorted_fields().len(), ty.fields.len());
    }

    #[test]
    fn add_field_appends() {
        let mut schema = Schema::structure(StructType::new([("b".to_string(), Schema::null())]));

        schema.add_field("a", Schema::null());

        let SchemaType::Struct(inner) = &schema.ty else {
            panic!("expected a struct");
        };

        assert_eq!(inner.fields.keys().collect::<Vec<_>>(), vec!["b", "a"]);
    }

    #[test]
    fn survives_a_serde_round_trip() {
        let schema = Schema::structure(StructType::new([
            ("zebra".to_string(), Schema::null()),
            ("apple".to_string(), Schema::null()),
        ]));

        let json = serde_json::to_string(&schema).unwrap();
        let back: Schema = serde_json::from_str(&json).unwrap();

        assert_eq!(back, schema);

        let SchemaType::Struct(inner) = &back.ty else {
            panic!("expected a struct");
        };

        assert_eq!(
            inner.fields.keys().collect::<Vec<_>>(),
            vec!["zebra", "apple"]
        );
    }
}

mod array_contains {
    use super::*;

    #[test]
    fn holds_a_schema_not_a_flag() {
        let ty = ArrayType {
            contains: Some(Box::new(Schema::string(StringType::default()))),
            ..ArrayType::new(Schema::null())
        };

        assert_eq!(
            ty.contains.as_deref().map(|schema| &schema.ty),
            Some(&SchemaType::String(Box::default()))
        );
    }

    #[test]
    fn round_trips_through_serde() {
        let schema = Schema::array(ArrayType {
            contains: Some(Box::new(Schema::string(StringType::default()))),
            ..ArrayType::new(Schema::null())
        });

        let json = serde_json::to_string(&schema).unwrap();

        assert_eq!(serde_json::from_str::<Schema>(&json).unwrap(), schema);
    }
}

mod display {
    use super::*;

    #[test]
    fn renders_scalars() {
        assert_eq!(Schema::null().to_string(), "null");
        assert_eq!(Schema::unknown().to_string(), "unknown");
        assert_eq!(Schema::boolean(BooleanType::default()).to_string(), "bool");
        assert_eq!(Schema::string(StringType::default()).to_string(), "string");
        assert_eq!(
            Schema::integer(IntegerType::new_kind(IntegerKind::U16)).to_string(),
            "u16"
        );
        assert_eq!(
            Schema::float(FloatType::new_kind(FloatKind::F64)).to_string(),
            "f64"
        );
    }

    #[test]
    fn renders_a_formatted_string() {
        let schema = Schema::string(StringType {
            format: Some("path".into()),
            ..StringType::default()
        });

        assert_eq!(schema.to_string(), "string:path");
    }

    #[test]
    fn renders_a_char_as_a_length_one_string() {
        let schema = Schema::string(StringType {
            min_length: Some(1),
            max_length: Some(1),
            ..StringType::default()
        });

        assert_eq!(schema.to_string(), "char");
    }

    #[test]
    fn renders_containers() {
        assert_eq!(
            Schema::array(ArrayType::new(Schema::string(StringType::default()))).to_string(),
            "[string]"
        );
        assert_eq!(
            Schema::object(ObjectType::new(
                Schema::string(StringType::default()),
                Schema::boolean(BooleanType::default()),
            ))
            .to_string(),
            "{string: bool}"
        );
        assert_eq!(
            Schema::tuple(TupleType::new([
                Schema::string(StringType::default()),
                Schema::null(),
            ]))
            .to_string(),
            "(string, null)"
        );
    }

    #[test]
    fn renders_unions_and_enums() {
        assert_eq!(
            SchemaBuilder::build_root::<Option<String>>().to_string(),
            "string | null"
        );
        assert_eq!(
            Schema::enumerable(EnumType::new([
                LiteralValue::String("a".into()),
                LiteralValue::UInt(2),
            ]))
            .to_string(),
            "\"a\" | 2"
        );
    }

    // Only structs and references render as their name; everything else
    // renders structurally even when named.
    #[test]
    fn renders_names_for_structs_and_references_only() {
        let mut schema = Schema::structure(StructType::default());
        schema.set_name("MyStruct");
        assert_eq!(schema.to_string(), "MyStruct");

        assert_eq!(Schema::reference("MyRef").to_string(), "MyRef");

        let mut schema = Schema::string(StringType::default());
        schema.set_name("MyString");
        assert_eq!(schema.to_string(), "string");
    }

    #[test]
    fn renders_literals() {
        assert_eq!(
            Schema::literal_value(LiteralValue::String("a".into())).to_string(),
            "\"a\""
        );
        assert_eq!(
            Schema::literal_value(LiteralValue::Bool(true)).to_string(),
            "true"
        );
        assert_eq!(
            Schema::literal_value(LiteralValue::Int(-3)).to_string(),
            "-3"
        );
    }
}

mod unions {
    use super::*;

    fn variants() -> [Schema; 2] {
        [Schema::string(StringType::default()), Schema::null()]
    }

    #[test]
    fn new_any_defaults_to_any_of() {
        let ty = UnionType::new_any(variants());

        assert_eq!(ty.operator, UnionOperator::AnyOf);
        assert_eq!(ty.variants_types.len(), 2);
        assert!(!ty.partial);
        assert_eq!(ty.default_index, None);
    }

    #[test]
    fn new_one_uses_one_of() {
        assert_eq!(
            UnionType::new_one(variants()).operator,
            UnionOperator::OneOf
        );
    }

    #[test]
    fn from_schemas_carries_the_default_index() {
        let ty = UnionType::from_schemas(variants(), Some(1));

        assert_eq!(ty.default_index, Some(1));
        assert_eq!(ty.operator, UnionOperator::AnyOf);
    }

    #[test]
    fn from_named_schemas_carries_names_by_position() {
        let ty = UnionType::from_named_schemas(
            [
                ("Text".to_owned(), Schema::string(StringType::default())),
                ("Nothing".to_owned(), Schema::null()),
            ],
            Some(1),
        );

        assert_eq!(ty.default_index, Some(1));
        assert_eq!(ty.variants_types.len(), 2);
        assert_eq!(ty.get_variant_name(0), Some(&"Text".to_owned()));
        assert_eq!(ty.get_variant_name(1), Some(&"Nothing".to_owned()));
        assert_eq!(ty.get_variant_name(2), None);
    }

    // A union built by hand has no names, so lookups are simply empty
    #[test]
    fn unnamed_union_has_no_variant_names() {
        let ty = UnionType::new_any(variants());

        assert_eq!(ty.variants_names, None);
        assert_eq!(ty.get_variant_name(0), None);
    }

    #[test]
    fn has_null_detects_a_null_variant() {
        assert!(UnionType::new_any(variants()).has_null());
        assert!(
            !UnionType::new_any([
                Schema::string(StringType::default()),
                Schema::boolean(BooleanType::default()),
            ])
            .has_null()
        );
    }
}

mod enums {
    use super::*;

    #[test]
    fn new_collects_values_without_variants() {
        let ty = EnumType::new([
            LiteralValue::String("a".into()),
            LiteralValue::String("b".into()),
        ]);

        assert_eq!(ty.values.len(), 2);
        assert!(ty.variants.is_none());
        assert_eq!(ty.default_index, None);
    }

    #[test]
    fn from_fields_keeps_declaration_order() {
        let ty = EnumType::from_fields(
            [
                (
                    "zebra".to_string(),
                    SchemaField::new(Schema::literal_value(LiteralValue::String("z".into()))),
                ),
                (
                    "apple".to_string(),
                    SchemaField::new(Schema::literal_value(LiteralValue::String("a".into()))),
                ),
            ],
            Some(1),
        );

        assert_eq!(
            ty.variants.as_ref().unwrap().keys().collect::<Vec<_>>(),
            vec!["zebra", "apple"]
        );
        assert_eq!(ty.values.len(), 2);
        assert_eq!(
            Schema::enumerable(ty).get_default(),
            Some(&LiteralValue::String("a".into()))
        );
    }

    #[test]
    fn from_fields_skips_non_literal_values() {
        let ty = EnumType::from_fields(
            [
                ("null".to_string(), SchemaField::new(Schema::null())),
                (
                    "text".to_string(),
                    SchemaField::new(Schema::literal_value(LiteralValue::String("t".into()))),
                ),
            ],
            None,
        );

        assert_eq!(ty.variants.as_ref().unwrap().len(), 2);
        assert_eq!(ty.values.len(), 1);
    }
}

mod objects {
    use super::*;

    #[test]
    fn new_sets_both_types() {
        let ty = ObjectType::new(
            Schema::string(StringType::default()),
            Schema::boolean(BooleanType::default()),
        );

        assert_eq!(ty.key_type.ty, SchemaType::String(Box::default()));
        assert_eq!(ty.value_type.ty, SchemaType::Boolean(Box::default()));
        assert_eq!(ty.required, None);
    }
}

mod literal_equality {
    use super::*;

    // Derived `PartialEq` would inherit `NaN != NaN`, making a schema
    // unequal to its own clone.
    #[test]
    fn nan_equals_itself() {
        for schema in [
            Schema::literal_value(LiteralValue::F64(f64::NAN)),
            Schema::literal_value(LiteralValue::F32(f32::NAN)),
        ] {
            assert_eq!(schema, schema.clone());
        }
    }

    #[test]
    fn ordinary_values_still_compare_normally() {
        assert_eq!(LiteralValue::F64(1.5), LiteralValue::F64(1.5));
        assert_ne!(LiteralValue::F64(1.5), LiteralValue::F64(2.5));
        assert_ne!(LiteralValue::F64(1.5), LiteralValue::F32(1.5));
        assert_eq!(LiteralValue::F64(0.0), LiteralValue::F64(-0.0));
        assert_ne!(LiteralValue::Int(1), LiteralValue::UInt(1));
    }
}

mod generic_names {
    use super::*;
    use std::collections::HashMap;

    struct Named;

    impl Schematic for Named {
        fn schema_name() -> Option<String> {
            Some("Named".into())
        }
    }

    #[test]
    fn uses_a_types_own_name_when_it_has_one() {
        assert_eq!(schema_name_of::<Named>(), "Named");
    }

    // Primitives and collections have no name of their own, so they fall
    // back to their Rust type name with module paths stripped. Without this
    // every instantiation over an unnamed type would resolve identically.
    #[test]
    fn falls_back_to_the_rust_type_name() {
        assert_eq!(schema_name_of::<bool>(), "Bool");
        assert_eq!(schema_name_of::<usize>(), "Usize");
        assert_eq!(schema_name_of::<String>(), "String");
        assert_eq!(schema_name_of::<str>(), "Str");
    }

    #[test]
    fn flattens_nested_type_arguments() {
        assert_eq!(schema_name_of::<Vec<String>>(), "VecString");
        assert_eq!(schema_name_of::<Option<bool>>(), "OptionBool");
        assert_eq!(
            schema_name_of::<HashMap<String, usize>>(),
            "HashMapStringUsize"
        );
        assert_eq!(schema_name_of::<Vec<Named>>(), "VecNamed");
    }

    #[test]
    fn produces_distinct_names_for_distinct_arguments() {
        assert_ne!(schema_name_of::<bool>(), schema_name_of::<usize>());
        assert_ne!(
            schema_name_of::<Vec<String>>(),
            schema_name_of::<Vec<usize>>()
        );
    }

    #[test]
    fn produces_identifier_safe_names() {
        for name in [
            schema_name_of::<Vec<String>>(),
            schema_name_of::<HashMap<String, Vec<bool>>>(),
            schema_name_of::<(bool, usize)>(),
        ] {
            assert!(
                name.chars().all(|c| c.is_alphanumeric() || c == '_'),
                "`{name}` is not usable as an identifier"
            );
        }
    }
}
