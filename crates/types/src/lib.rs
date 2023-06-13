mod arrays;
mod enums;
mod literals;
mod numbers;
mod objects;
mod strings;
mod structs;
mod tuples;
mod unions;

pub use arrays::*;
pub use enums::*;
pub use literals::*;
pub use numbers::*;
pub use objects::*;
pub use strings::*;
pub use structs::*;
pub use tuples::*;
pub use unions::*;

/// All possible types within a schema.
#[derive(Clone, Debug, Default)]
pub enum SchemaType {
    Boolean,
    Null,
    #[default]
    Unknown,
    Array(ArrayType),
    Enum(EnumType),
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
    /// Infer a schema from a type that implements [`Schematic`].
    pub fn infer<T: Schematic>() -> SchemaType {
        T::generate_schema()
    }

    /// Create an array schema with the provided item types.
    pub fn array(items_type: SchemaType) -> SchemaType {
        SchemaType::Array(ArrayType {
            items_type: Box::new(items_type),
            ..ArrayType::default()
        })
    }

    /// Create a float schema with the provided kind.
    pub fn float(kind: FloatKind) -> SchemaType {
        SchemaType::Float(FloatType {
            kind,
            ..FloatType::default()
        })
    }

    /// Create an integer schema with the provided kind.
    pub fn integer(kind: IntegerKind) -> SchemaType {
        SchemaType::Integer(IntegerType {
            kind,
            ..IntegerType::default()
        })
    }

    /// Create a literal schema with the provided value.
    pub fn literal(value: LiteralValue) -> SchemaType {
        SchemaType::Literal(LiteralType {
            value: Some(value),
            ..LiteralType::default()
        })
    }

    /// Create an indexed/mapable object schema with the provided key and value types.
    pub fn object(key_type: SchemaType, value_type: SchemaType) -> SchemaType {
        SchemaType::Object(ObjectType {
            key_type: Box::new(key_type),
            value_type: Box::new(value_type),
            ..ObjectType::default()
        })
    }

    /// Create a string schema.
    pub fn string() -> SchemaType {
        SchemaType::String(StringType::default())
    }

    /// Create a struct/shape schema with the provided fields.
    pub fn structure<I>(fields: I) -> SchemaType
    where
        I: IntoIterator<Item = SchemaField>,
    {
        SchemaType::Struct(StructType {
            fields: fields.into_iter().collect(),
            ..StructType::default()
        })
    }

    /// Create a tuple schema with the provided item types.
    pub fn tuple<I>(items_types: I) -> SchemaType
    where
        I: IntoIterator<Item = SchemaType>,
    {
        SchemaType::Tuple(TupleType {
            items_types: items_types.into_iter().map(Box::new).collect(),
            ..TupleType::default()
        })
    }

    /// Create an "any of" union.
    pub fn union<I>(variants_types: I) -> SchemaType
    where
        I: IntoIterator<Item = SchemaType>,
    {
        SchemaType::Union(UnionType {
            variants_types: variants_types.into_iter().map(Box::new).collect(),
            ..UnionType::default()
        })
    }

    /// Create a "one of" union.
    pub fn union_one<I>(variants_types: I) -> SchemaType
    where
        I: IntoIterator<Item = SchemaType>,
    {
        SchemaType::Union(UnionType {
            operator: UnionOperator::OneOf,
            variants_types: variants_types.into_iter().map(Box::new).collect(),
            ..UnionType::default()
        })
    }

    /// Return a `name` from the inner schema type.
    pub fn get_name(&self) -> Option<&String> {
        match self {
            SchemaType::Boolean => None,
            SchemaType::Null => None,
            SchemaType::Unknown => None,
            SchemaType::Array(ArrayType { name, .. }) => name.as_ref(),
            SchemaType::Enum(EnumType { name, .. }) => name.as_ref(),
            SchemaType::Float(FloatType { name, .. }) => name.as_ref(),
            SchemaType::Integer(IntegerType { name, .. }) => name.as_ref(),
            SchemaType::Literal(LiteralType { name, .. }) => name.as_ref(),
            SchemaType::Object(ObjectType { name, .. }) => name.as_ref(),
            SchemaType::Struct(StructType { name, .. }) => name.as_ref(),
            SchemaType::String(StringType { name, .. }) => name.as_ref(),
            SchemaType::Tuple(TupleType { name, .. }) => name.as_ref(),
            SchemaType::Union(UnionType { name, .. }) => name.as_ref(),
        }
    }

    /// Set the `name` of the inner schema type. If the inner type does not support
    /// names, this is a no-op.
    pub fn set_name<S: AsRef<str>>(&mut self, name: S) {
        let name = Some(name.as_ref().to_owned());

        match self {
            SchemaType::Array(ref mut inner) => {
                inner.name = name;
            }
            SchemaType::Enum(ref mut inner) => {
                inner.name = name;
            }
            SchemaType::Float(ref mut inner) => {
                inner.name = name;
            }
            SchemaType::Integer(ref mut inner) => {
                inner.name = name;
            }
            SchemaType::Literal(ref mut inner) => {
                inner.name = name;
            }
            SchemaType::Object(ref mut inner) => {
                inner.name = name;
            }
            SchemaType::Struct(ref mut inner) => {
                inner.name = name;
            }
            SchemaType::String(ref mut inner) => {
                inner.name = name;
            }
            SchemaType::Tuple(ref mut inner) => {
                inner.name = name;
            }
            SchemaType::Union(ref mut inner) => {
                inner.name = name;
            }
            _ => {}
        };
    }
}

#[derive(Clone, Debug, Default)]
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
    /// Create and return a schema that models the structure of the implementing type.
    /// The schema can be used to generate code, documentation, or other artifacts.
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
        SchemaType::union_one([T::generate_schema(), SchemaType::Null])
    }
}
