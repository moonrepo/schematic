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
