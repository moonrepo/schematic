mod arrays;
mod literals;
mod numbers;
mod objects;
mod strings;
mod structs;
mod tuples;
mod unions;

pub use arrays::*;
pub use literals::*;
pub use numbers::*;
pub use objects::*;
pub use strings::*;
pub use structs::*;
pub use tuples::*;
pub use unions::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum SchemaType {
    Boolean,
    Null,
    #[default]
    Unknown,
    Array(ArrayType),
    Float(FloatType),
    Integer(IntegerType),
    Literal(LiteralType),
    Object(ObjectType),
    Struct(StructType),
    String(StringType),
    Tuple(TupleType),
    Union(UnionType),
}

impl SchemaType {
    pub fn infer<T: Schematic>() -> SchemaType {
        T::generate_schema()
    }

    pub fn array(items_type: SchemaType) -> SchemaType {
        SchemaType::Array(ArrayType {
            items_type: Box::new(items_type),
            ..ArrayType::default()
        })
    }

    pub fn float(kind: FloatKind) -> SchemaType {
        SchemaType::Float(FloatType {
            kind,
            ..FloatType::default()
        })
    }

    pub fn integer(kind: IntegerKind) -> SchemaType {
        SchemaType::Integer(IntegerType {
            kind,
            ..IntegerType::default()
        })
    }

    pub fn literal(value: LiteralValue) -> SchemaType {
        SchemaType::Literal(LiteralType {
            format: None,
            value: Some(value),
        })
    }

    pub fn object(key_type: SchemaType, value_type: SchemaType) -> SchemaType {
        SchemaType::Object(ObjectType {
            key_type: Box::new(key_type),
            value_type: Box::new(value_type),
            ..ObjectType::default()
        })
    }

    pub fn string() -> SchemaType {
        SchemaType::String(StringType::default())
    }

    pub fn structure<I>(name: &str, fields: I) -> SchemaType
    where
        I: IntoIterator<Item = SchemaField>,
    {
        SchemaType::Struct(StructType {
            name: name.to_owned(),
            fields: fields.into_iter().collect(),
            ..StructType::default()
        })
    }

    pub fn tuple<I>(items_types: I) -> SchemaType
    where
        I: IntoIterator<Item = SchemaType>,
    {
        SchemaType::Tuple(TupleType {
            items_types: items_types.into_iter().map(Box::new).collect(),
        })
    }

    pub fn union<I>(variants_types: I) -> SchemaType
    where
        I: IntoIterator<Item = SchemaType>,
    {
        SchemaType::Union(UnionType {
            variants_types: variants_types.into_iter().map(Box::new).collect(),
            ..UnionType::default()
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SchemaField {
    pub name: Option<String>,
    pub description: Option<String>,
    pub type_of: SchemaType,
    pub deprecated: bool,
    pub hidden: bool,
    pub nullable: bool,
    pub optional: bool,
    pub read_only: bool,
    pub write_only: bool,
}

pub trait Schematic {
    fn generate_schema() -> SchemaType {
        SchemaType::Unknown
    }
}

// CORE

impl<T: Schematic> Schematic for &T {
    fn generate_schema() -> SchemaType {
        T::generate_schema()
    }
}

impl<T: Schematic> Schematic for &mut T {
    fn generate_schema() -> SchemaType {
        T::generate_schema()
    }
}

impl<T: Schematic> Schematic for Box<T> {
    fn generate_schema() -> SchemaType {
        T::generate_schema()
    }
}

impl<T: Schematic> Schematic for Option<T> {
    fn generate_schema() -> SchemaType {
        SchemaType::union([T::generate_schema(), SchemaType::Null])
    }
}
