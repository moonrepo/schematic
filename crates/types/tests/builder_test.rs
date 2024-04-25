use schematic_types::*;
use std::collections::{BTreeSet, HashMap, HashSet};

fn test_builder<T: Schematic>() -> Schema {
    SchemaBuilder::generate::<T>()
}

pub struct Named {
    pub field: bool,
}

impl Schematic for Named {
    fn schema_name() -> Option<String> {
        Some("Named".into())
    }

    fn generate_schema(mut schema: SchemaBuilder) -> Schema {
        schema.structure(StructType::new([SchemaField::from_type(
            "field",
            schema.infer_type::<bool>(),
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
            items_type: Box::new(SchemaType::String(Box::new(StringType::default()))),
            max_length: Some(3),
            min_length: Some(3),
            ..ArrayType::default()
        }))
    );

    assert_eq!(
        test_builder::<HashSet<String>>().type_of,
        SchemaType::Array(Box::new(ArrayType {
            items_type: Box::new(SchemaType::String(Box::new(StringType::default()))),
            unique: Some(true),
            ..ArrayType::default()
        }))
    );

    assert_eq!(
        test_builder::<BTreeSet<String>>().type_of,
        SchemaType::Array(Box::new(ArrayType {
            items_type: Box::new(SchemaType::String(Box::new(StringType::default()))),
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

pub struct Cycle {
    pub values: HashMap<String, Cycle>,
}

impl Schematic for Cycle {
    fn schema_name() -> Option<String> {
        Some("Cycle".into())
    }

    fn generate_schema(mut schema: SchemaBuilder) -> Schema {
        schema.structure(StructType::new([SchemaField::from_type(
            "values",
            schema.infer_type::<HashMap<String, Cycle>>(),
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
                    SchemaType::String(Box::new(StringType::default())),
                    SchemaType::Reference("Cycle".into()),
                ))),
            )],
            partial: false,
            required: None
        }))
    );
}
