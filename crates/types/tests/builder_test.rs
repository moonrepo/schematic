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
        schema.structure(StructType::new([SchemaField::from_schema(
            "field",
            schema.infer::<bool>(),
        )]));
        schema.build()
    }
}

#[test]
fn primitives() {
    assert_eq!(test_builder::<()>().type_of, SchemaType::Null);

    assert_eq!(
        test_builder::<bool>().type_of,
        SchemaType::Boolean(Box::new(BooleanType::default()))
    );

    assert_eq!(
        test_builder::<&bool>().type_of,
        SchemaType::Boolean(Box::new(BooleanType::default()))
    );

    assert_eq!(
        test_builder::<&mut bool>().type_of,
        SchemaType::Boolean(Box::new(BooleanType::default()))
    );

    assert_eq!(
        test_builder::<Box<bool>>().type_of,
        SchemaType::Boolean(Box::new(BooleanType::default()))
    );

    assert_eq!(
        test_builder::<Option<bool>>().type_of,
        SchemaType::Union(Box::new(UnionType::new_any(vec![
            SchemaType::Boolean(Box::new(BooleanType::default())),
            SchemaType::Null
        ])))
    );
}

#[test]
fn arrays() {
    assert_eq!(
        test_builder::<Vec<String>>().type_of,
        SchemaType::Array(Box::new(ArrayType::new(SchemaType::String(Box::new(
            StringType::default()
        )))))
    );

    assert_eq!(
        test_builder::<&[String]>().type_of,
        SchemaType::Array(Box::new(ArrayType::new(SchemaType::String(Box::new(
            StringType::default()
        )))))
    );

    assert_eq!(
        test_builder::<[String; 3]>().type_of,
        SchemaType::Array(Box::new(ArrayType {
            items_type: Box::new(Schema::string(StringType::default())),
            max_length: Some(3),
            min_length: Some(3),
            ..ArrayType::default()
        }))
    );

    assert_eq!(
        test_builder::<HashSet<String>>().type_of,
        SchemaType::Array(Box::new(ArrayType {
            items_type: Box::new(Schema::string(StringType::default())),
            unique: Some(true),
            ..ArrayType::default()
        }))
    );

    assert_eq!(
        test_builder::<BTreeSet<String>>().type_of,
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
        test_builder::<u8>().type_of,
        SchemaType::Integer(Box::new(IntegerType::new_kind(IntegerKind::U8)))
    );

    assert_eq!(
        test_builder::<i32>().type_of,
        SchemaType::Integer(Box::new(IntegerType::new_kind(IntegerKind::I32)))
    );
}

#[test]
fn floats() {
    assert_eq!(
        test_builder::<f32>().type_of,
        SchemaType::Float(Box::new(FloatType::new_kind(FloatKind::F32)))
    );

    assert_eq!(
        test_builder::<f64>().type_of,
        SchemaType::Float(Box::new(FloatType::new_kind(FloatKind::F64)))
    );
}

#[test]
fn objects() {
    assert_eq!(
        test_builder::<HashMap<String, Named>>().type_of,
        SchemaType::Object(Box::new(ObjectType::new(
            Schema::string(StringType::default()),
            test_builder::<Named>(),
        )))
    );

    assert_eq!(
        test_builder::<BTreeMap<u128, Named>>().type_of,
        SchemaType::Object(Box::new(ObjectType::new(
            SchemaType::Integer(Box::new(IntegerType::new_kind(IntegerKind::U128))),
            test_builder::<Named>(),
        )))
    );
}

#[test]
fn strings() {
    assert_eq!(
        test_builder::<char>().type_of,
        SchemaType::String(Box::new(StringType {
            max_length: Some(1),
            min_length: Some(1),
            ..StringType::default()
        }))
    );

    assert_eq!(
        test_builder::<&str>().type_of,
        SchemaType::String(Box::new(StringType::default()))
    );

    assert_eq!(
        test_builder::<String>().type_of,
        SchemaType::String(Box::new(StringType::default()))
    );

    assert_eq!(
        test_builder::<&Path>().type_of,
        SchemaType::String(Box::new(StringType {
            format: Some("path".into()),
            ..StringType::default()
        }))
    );

    assert_eq!(
        test_builder::<PathBuf>().type_of,
        SchemaType::String(Box::new(StringType {
            format: Some("path".into()),
            ..StringType::default()
        }))
    );

    assert_eq!(
        test_builder::<Ipv4Addr>().type_of,
        SchemaType::String(Box::new(StringType {
            format: Some("ipv4".into()),
            ..StringType::default()
        }))
    );

    assert_eq!(
        test_builder::<Duration>().type_of,
        SchemaType::String(Box::new(StringType {
            format: Some("duration".into()),
            ..StringType::default()
        }))
    );
}

#[test]
fn tuples() {
    assert_eq!(
        test_builder::<(bool, i16, f32, String)>().type_of,
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
        schema.structure(StructType::new([SchemaField::from_schema(
            "values",
            schema.infer::<HashMap<String, Cycle>>(),
        )]));
        schema.build()
    }
}

#[test]
fn supports_cycles() {
    assert_eq!(
        test_builder::<Cycle>().type_of,
        SchemaType::Struct(Box::new(StructType {
            fields: vec![SchemaField::from_type(
                "values",
                SchemaType::Object(Box::new(ObjectType::new(
                    Schema::string(StringType::default()),
                    SchemaType::Reference("Cycle".into()),
                ))),
            )],
            partial: false,
            required: None
        }))
    );
}
