#![allow(dead_code)]

use schematic_types::*;
use starbase_sandbox::assert_snapshot;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn test_builder<T: Schematic>() -> Schema {
    SchemaBuilder::build_root::<T>()
}

macro_rules! assert_build {
    ($ty:ty, $expected:expr_2021) => {
        let schema = test_builder::<$ty>();

        assert_eq!(schema.ty, $expected);

        let input = serde_json::to_string_pretty(&schema).unwrap();

        assert_snapshot!(&input);

        let output: Schema = serde_json::from_str(&input).unwrap();

        assert_eq!(schema, output);
    };
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
    assert_build!((), SchemaType::Null);

    assert_build!(bool, SchemaType::Boolean(Box::default()));

    assert_build!(&bool, SchemaType::Boolean(Box::default()));

    assert_build!(&mut bool, SchemaType::Boolean(Box::default()));

    assert_build!(Box<bool>, SchemaType::Boolean(Box::default()));

    assert_build!(
        Option<bool>,
        SchemaType::Union(Box::new(UnionType::new_any(vec![
            SchemaType::Boolean(Box::default()),
            SchemaType::Null
        ])))
    );
}

#[test]
fn arrays() {
    assert_build!(
        Vec<String>,
        SchemaType::Array(Box::new(ArrayType::new(SchemaType::String(Box::default()))))
    );

    assert_build!(
        &[String],
        SchemaType::Array(Box::new(ArrayType::new(SchemaType::String(Box::default()))))
    );

    assert_build!(
        [String; 3],
        SchemaType::Array(Box::new(ArrayType {
            items_type: Box::new(Schema::string(StringType::default())),
            max_length: Some(3),
            min_length: Some(3),
            ..ArrayType::default()
        }))
    );

    assert_build!(
        HashSet<String>,
        SchemaType::Array(Box::new(ArrayType {
            items_type: Box::new(Schema::string(StringType::default())),
            unique: Some(true),
            ..ArrayType::default()
        }))
    );

    assert_build!(
        BTreeSet<String>,
        SchemaType::Array(Box::new(ArrayType {
            items_type: Box::new(Schema::string(StringType::default())),
            unique: Some(true),
            ..ArrayType::default()
        }))
    );
}

#[test]
fn integers() {
    assert_build!(
        u8,
        SchemaType::Integer(Box::new(IntegerType::new_kind(IntegerKind::U8)))
    );

    assert_build!(
        i32,
        SchemaType::Integer(Box::new(IntegerType::new_kind(IntegerKind::I32)))
    );
}

#[test]
fn floats() {
    assert_build!(
        f32,
        SchemaType::Float(Box::new(FloatType::new_kind(FloatKind::F32)))
    );

    assert_build!(
        f64,
        SchemaType::Float(Box::new(FloatType::new_kind(FloatKind::F64)))
    );
}

#[test]
fn objects() {
    assert_build!(
        HashMap<String, Named>,
        SchemaType::Object(Box::new(ObjectType::new(
            Schema::string(StringType::default()),
            test_builder::<Named>(),
        )))
    );

    assert_build!(
        BTreeMap<u128, Named>,
        SchemaType::Object(Box::new(ObjectType::new(
            SchemaType::Integer(Box::new(IntegerType::new_kind(IntegerKind::U128))),
            test_builder::<Named>(),
        )))
    );
}

#[test]
fn strings() {
    assert_build!(
        char,
        SchemaType::String(Box::new(StringType {
            max_length: Some(1),
            min_length: Some(1),
            ..StringType::default()
        }))
    );

    assert_build!(&str, SchemaType::String(Box::default()));

    assert_build!(String, SchemaType::String(Box::default()));

    assert_build!(
        &Path,
        SchemaType::String(Box::new(StringType {
            format: Some("path".into()),
            ..StringType::default()
        }))
    );

    assert_build!(
        PathBuf,
        SchemaType::String(Box::new(StringType {
            format: Some("path".into()),
            ..StringType::default()
        }))
    );

    assert_build!(
        Ipv4Addr,
        SchemaType::String(Box::new(StringType {
            format: Some("ipv4".into()),
            ..StringType::default()
        }))
    );

    assert_build!(
        Duration,
        SchemaType::String(Box::new(StringType {
            format: Some("duration".into()),
            ..StringType::default()
        }))
    );
}

struct TestStruct {
    str: String,
    num: usize,
}

impl Schematic for TestStruct {
    fn schema_name() -> Option<String> {
        Some("TestStruct".into())
    }

    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.structure(StructType::new([
            ("str".into(), schema.infer::<String>()),
            ("num".into(), schema.infer::<usize>()),
        ]))
    }
}

#[test]
fn structs() {
    assert_build!(
        TestStruct,
        SchemaType::Struct(Box::new(StructType::new([
            (
                "str".into(),
                Schema::new(SchemaType::String(Box::default()))
            ),
            (
                "num".into(),
                Schema::new(SchemaType::Integer(Box::new(IntegerType::new_kind(
                    IntegerKind::Usize
                ))))
            ),
        ])))
    );
}

#[test]
fn tuples() {
    assert_build!(
        (bool, i16, f32, String),
        SchemaType::Tuple(Box::new(TupleType::new(vec![
            SchemaType::Boolean(Box::default()),
            SchemaType::Integer(Box::new(IntegerType::new_kind(IntegerKind::I16))),
            SchemaType::Float(Box::new(FloatType::new_kind(FloatKind::F32))),
            SchemaType::String(Box::default())
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
