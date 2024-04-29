use schematic_types::*;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn test_builder<T: Schematic>() -> Schema {
    SchemaBuilder::build_root::<T>()
}

pub struct Named {
    pub field: bool,
}

impl Schematic for Named {
    fn schema_name() -> Option<String> {
        Some("Named".into())
    }

    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.structure(StructType::new([("field".into(), schema.infer::<bool>())]));
        schema.build()
    }
}

#[test]
fn primitives() {
    assert_eq!(test_builder::<()>().ty, SchemaType::Null);

    assert_eq!(
        test_builder::<bool>().ty,
        SchemaType::Boolean(Box::new(BooleanType::default()))
    );

    assert_eq!(
        test_builder::<&bool>().ty,
        SchemaType::Boolean(Box::new(BooleanType::default()))
    );

    assert_eq!(
        test_builder::<&mut bool>().ty,
        SchemaType::Boolean(Box::new(BooleanType::default()))
    );

    assert_eq!(
        test_builder::<Box<bool>>().ty,
        SchemaType::Boolean(Box::new(BooleanType::default()))
    );

    assert_eq!(
        test_builder::<Option<bool>>().ty,
        SchemaType::Union(Box::new(UnionType::new_any(vec![
            SchemaType::Boolean(Box::new(BooleanType::default())),
            SchemaType::Null
        ])))
    );
}

#[test]
fn arrays() {
    assert_eq!(
        test_builder::<Vec<String>>().ty,
        SchemaType::Array(Box::new(ArrayType::new(SchemaType::String(Box::new(
            StringType::default()
        )))))
    );

    assert_eq!(
        test_builder::<&[String]>().ty,
        SchemaType::Array(Box::new(ArrayType::new(SchemaType::String(Box::new(
            StringType::default()
        )))))
    );

    assert_eq!(
        test_builder::<[String; 3]>().ty,
        SchemaType::Array(Box::new(ArrayType {
            items_type: Box::new(Schema::string(StringType::default())),
            max_length: Some(3),
            min_length: Some(3),
            ..ArrayType::default()
        }))
    );

    assert_eq!(
        test_builder::<HashSet<String>>().ty,
        SchemaType::Array(Box::new(ArrayType {
            items_type: Box::new(Schema::string(StringType::default())),
            unique: Some(true),
            ..ArrayType::default()
        }))
    );

    assert_eq!(
        test_builder::<BTreeSet<String>>().ty,
        SchemaType::Array(Box::new(ArrayType {
            items_type: Box::new(Schema::string(StringType::default())),
            unique: Some(true),
            ..ArrayType::default()
        }))
    );
}

#[test]
fn integers() {
    assert_eq!(
        test_builder::<u8>().ty,
        SchemaType::Integer(Box::new(IntegerType::new_kind(IntegerKind::U8)))
    );

    assert_eq!(
        test_builder::<i32>().ty,
        SchemaType::Integer(Box::new(IntegerType::new_kind(IntegerKind::I32)))
    );
}

#[test]
fn floats() {
    assert_eq!(
        test_builder::<f32>().ty,
        SchemaType::Float(Box::new(FloatType::new_kind(FloatKind::F32)))
    );

    assert_eq!(
        test_builder::<f64>().ty,
        SchemaType::Float(Box::new(FloatType::new_kind(FloatKind::F64)))
    );
}

#[test]
fn objects() {
    assert_eq!(
        test_builder::<HashMap<String, Named>>().ty,
        SchemaType::Object(Box::new(ObjectType::new(
            Schema::string(StringType::default()),
            test_builder::<Named>(),
        )))
    );

    assert_eq!(
        test_builder::<BTreeMap<u128, Named>>().ty,
        SchemaType::Object(Box::new(ObjectType::new(
            SchemaType::Integer(Box::new(IntegerType::new_kind(IntegerKind::U128))),
            test_builder::<Named>(),
        )))
    );
}

#[test]
fn strings() {
    assert_eq!(
        test_builder::<char>().ty,
        SchemaType::String(Box::new(StringType {
            max_length: Some(1),
            min_length: Some(1),
            ..StringType::default()
        }))
    );

    assert_eq!(
        test_builder::<&str>().ty,
        SchemaType::String(Box::new(StringType::default()))
    );

    assert_eq!(
        test_builder::<String>().ty,
        SchemaType::String(Box::new(StringType::default()))
    );

    assert_eq!(
        test_builder::<&Path>().ty,
        SchemaType::String(Box::new(StringType {
            format: Some("path".into()),
            ..StringType::default()
        }))
    );

    assert_eq!(
        test_builder::<PathBuf>().ty,
        SchemaType::String(Box::new(StringType {
            format: Some("path".into()),
            ..StringType::default()
        }))
    );

    assert_eq!(
        test_builder::<Ipv4Addr>().ty,
        SchemaType::String(Box::new(StringType {
            format: Some("ipv4".into()),
            ..StringType::default()
        }))
    );

    assert_eq!(
        test_builder::<Duration>().ty,
        SchemaType::String(Box::new(StringType {
            format: Some("duration".into()),
            ..StringType::default()
        }))
    );
}

#[test]
fn tuples() {
    assert_eq!(
        test_builder::<(bool, i16, f32, String)>().ty,
        SchemaType::Tuple(Box::new(TupleType::new(vec![
            SchemaType::Boolean(Box::new(BooleanType::default())),
            SchemaType::Integer(Box::new(IntegerType::new_kind(IntegerKind::I16))),
            SchemaType::Float(Box::new(FloatType::new_kind(FloatKind::F32))),
            SchemaType::String(Box::new(StringType::default()))
        ])))
    );
}

pub struct Cycle {
    pub values: HashMap<String, Cycle>,
}

impl Schematic for Cycle {
    fn schema_name() -> Option<String> {
        Some("Cycle".into())
    }

    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.structure(StructType::new([(
            "values".into(),
            schema.infer::<HashMap<String, Cycle>>(),
        )]));
        schema.build()
    }
}

#[test]
fn supports_cycles() {
    assert_eq!(
        test_builder::<Cycle>().ty,
        SchemaType::Struct(Box::new(StructType {
            fields: BTreeMap::from_iter([(
                "values".into(),
                Box::new(Schema::object(ObjectType::new(
                    Schema::string(StringType::default()),
                    SchemaType::Reference("Cycle".into()),
                )))
            ),]),
            partial: false,
            required: None
        }))
    );
}
