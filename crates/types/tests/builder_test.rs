use schematic_types::*;
use starbase_sandbox::assert_snapshot;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn test_builder<T: Schematic>() -> Schema {
    SchemaBuilder::build_root::<T>()
}

fn assert_serde<T: Schematic>() {
    let schema = test_builder::<T>();
    let input = serde_json::to_string_pretty(&schema).unwrap();

    assert_snapshot!(&input);

    let output: Schema = serde_json::from_str(&input).unwrap();

    assert_eq!(schema, output);
}

pub struct Named {
    pub field: bool,
}

impl Schematic for Named {
    fn schema_name() -> Option<String> {
        Some("Named".into())
    }

    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.structure(StructType::new([("field".into(), schema.infer::<bool>())]))
    }
}

#[test]
fn primitives() {
    assert_eq!(test_builder::<()>().ty, SchemaType::Null);
    assert_serde::<()>();

    assert_eq!(
        test_builder::<bool>().ty,
        SchemaType::Boolean(Box::default())
    );
    assert_serde::<bool>();

    assert_eq!(
        test_builder::<&bool>().ty,
        SchemaType::Boolean(Box::default())
    );

    assert_eq!(
        test_builder::<&mut bool>().ty,
        SchemaType::Boolean(Box::default())
    );

    assert_eq!(
        test_builder::<Box<bool>>().ty,
        SchemaType::Boolean(Box::default())
    );
    assert_serde::<Box<bool>>();

    assert_eq!(
        test_builder::<Option<bool>>().ty,
        SchemaType::Union(Box::new(UnionType::new_any(vec![
            SchemaType::Boolean(Box::default()),
            SchemaType::Null
        ])))
    );
    assert_serde::<Option<bool>>();
}

#[test]
fn arrays() {
    assert_eq!(
        test_builder::<Vec<String>>().ty,
        SchemaType::Array(Box::new(ArrayType::new(SchemaType::String(Box::default()))))
    );
    assert_serde::<Vec<String>>();

    assert_eq!(
        test_builder::<&[String]>().ty,
        SchemaType::Array(Box::new(ArrayType::new(SchemaType::String(Box::default()))))
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
    assert_serde::<HashSet<String>>();

    assert_eq!(
        test_builder::<BTreeSet<String>>().ty,
        SchemaType::Array(Box::new(ArrayType {
            items_type: Box::new(Schema::string(StringType::default())),
            unique: Some(true),
            ..ArrayType::default()
        }))
    );
    assert_serde::<BTreeSet<String>>();
}

#[test]
fn integers() {
    assert_eq!(
        test_builder::<u8>().ty,
        SchemaType::Integer(Box::new(IntegerType::new_kind(IntegerKind::U8)))
    );
    assert_serde::<u8>();

    assert_eq!(
        test_builder::<i32>().ty,
        SchemaType::Integer(Box::new(IntegerType::new_kind(IntegerKind::I32)))
    );
    assert_serde::<i32>();
}

#[test]
fn floats() {
    assert_eq!(
        test_builder::<f32>().ty,
        SchemaType::Float(Box::new(FloatType::new_kind(FloatKind::F32)))
    );
    assert_serde::<f32>();

    assert_eq!(
        test_builder::<f64>().ty,
        SchemaType::Float(Box::new(FloatType::new_kind(FloatKind::F64)))
    );
    assert_serde::<f64>();
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
    assert_serde::<HashMap<String, Named>>();

    assert_eq!(
        test_builder::<BTreeMap<u128, Named>>().ty,
        SchemaType::Object(Box::new(ObjectType::new(
            SchemaType::Integer(Box::new(IntegerType::new_kind(IntegerKind::U128))),
            test_builder::<Named>(),
        )))
    );
    assert_serde::<BTreeMap<u128, Named>>();
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
    assert_serde::<char>();

    assert_eq!(
        test_builder::<&str>().ty,
        SchemaType::String(Box::default())
    );
    assert_serde::<&str>();

    assert_eq!(
        test_builder::<String>().ty,
        SchemaType::String(Box::default())
    );
    assert_serde::<String>();

    assert_eq!(
        test_builder::<&Path>().ty,
        SchemaType::String(Box::new(StringType {
            format: Some("path".into()),
            ..StringType::default()
        }))
    );
    assert_serde::<&Path>();

    assert_eq!(
        test_builder::<PathBuf>().ty,
        SchemaType::String(Box::new(StringType {
            format: Some("path".into()),
            ..StringType::default()
        }))
    );
    assert_serde::<PathBuf>();

    assert_eq!(
        test_builder::<Ipv4Addr>().ty,
        SchemaType::String(Box::new(StringType {
            format: Some("ipv4".into()),
            ..StringType::default()
        }))
    );
    assert_serde::<Ipv4Addr>();

    assert_eq!(
        test_builder::<Duration>().ty,
        SchemaType::String(Box::new(StringType {
            format: Some("duration".into()),
            ..StringType::default()
        }))
    );
    assert_serde::<Duration>();
}

#[test]
fn tuples() {
    assert_eq!(
        test_builder::<(bool, i16, f32, String)>().ty,
        SchemaType::Tuple(Box::new(TupleType::new(vec![
            SchemaType::Boolean(Box::default()),
            SchemaType::Integer(Box::new(IntegerType::new_kind(IntegerKind::I16))),
            SchemaType::Float(Box::new(FloatType::new_kind(FloatKind::F32))),
            SchemaType::String(Box::default())
        ])))
    );
    assert_serde::<(bool, i16, f32, String)>();
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
        )]))
    }
}

#[test]
fn supports_cycles() {
    assert_eq!(
        test_builder::<Cycle>().ty,
        SchemaType::Struct(Box::new(StructType {
            fields: BTreeMap::from_iter([(
                "values".into(),
                Box::new(SchemaField {
                    schema: Schema::object(ObjectType::new(
                        Schema::string(StringType::default()),
                        SchemaType::Reference("Cycle".into()),
                    )),
                    ..Default::default()
                })
            ),]),
            partial: false,
            required: None
        }))
    );
}
