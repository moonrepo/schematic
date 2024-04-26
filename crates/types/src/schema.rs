use std::ops::{Deref, DerefMut};

use crate::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Schema {
    pub description: Option<String>,
    pub name: Option<String>,
    pub type_of: SchemaType,
}

impl Schema {
    /// Create a schema with the provided type.
    pub fn new(type_of: SchemaType) -> Self {
        Self {
            description: None,
            name: None,
            type_of,
        }
    }

    /// Create an array schema.
    pub fn array(value: ArrayType) -> Self {
        Self::new(SchemaType::Array(Box::new(value)))
    }

    /// Create a boolean schema.
    pub fn boolean(value: BooleanType) -> Self {
        Self::new(SchemaType::Boolean(Box::new(value)))
    }

    /// Create an enum schema.
    pub fn enumerable(value: EnumType) -> Self {
        Self::new(SchemaType::Enum(Box::new(value)))
    }

    /// Create a float schema.
    pub fn float(value: FloatType) -> Self {
        Self::new(SchemaType::Float(Box::new(value)))
    }

    /// Create an integer schema.
    pub fn integer(value: IntegerType) -> Self {
        Self::new(SchemaType::Integer(Box::new(value)))
    }

    /// Create a literal schema.
    pub fn literal(value: LiteralType) -> Self {
        Self::new(SchemaType::Literal(Box::new(value)))
    }

    /// Create an object schema.
    pub fn object(value: ObjectType) -> Self {
        Self::new(SchemaType::Object(Box::new(value)))
    }

    /// Create a string schema.
    pub fn string(value: StringType) -> Self {
        Self::new(SchemaType::String(Box::new(value)))
    }

    /// Create a struct schema.
    pub fn structure(value: StructType) -> Self {
        Self::new(SchemaType::Struct(Box::new(value)))
    }

    /// Create a tuple schema.
    pub fn tuple(value: TupleType) -> Self {
        Self::new(SchemaType::Tuple(Box::new(value)))
    }

    /// Create a union schema.
    pub fn union(value: UnionType) -> Self {
        Self::new(SchemaType::Union(Box::new(value)))
    }

    /// Create a null schema.
    pub fn null() -> Self {
        Self::new(SchemaType::Null)
    }

    /// Create an unknown schema.
    pub fn unknown() -> Self {
        Self::new(SchemaType::Unknown)
    }
}

impl Deref for Schema {
    type Target = SchemaType;

    fn deref(&self) -> &Self::Target {
        &self.type_of
    }
}

impl DerefMut for Schema {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.type_of
    }
}

/// Represents a field within a schema struct, or a variant within a schema enum/union.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SchemaField {
    pub name: String,
    pub description: Option<String>,
    pub type_of: Box<SchemaType>,
    pub type_name: Option<String>,
    pub deprecated: Option<String>,
    pub env_var: Option<String>,
    pub hidden: bool,
    pub nullable: bool,
    pub optional: bool,
    pub read_only: bool,
    pub write_only: bool,
}

impl SchemaField {
    /// Create a new field from a [`SchemaType`].
    pub fn from_type(name: &str, type_of: SchemaType) -> SchemaField {
        SchemaField {
            name: name.to_owned(),
            type_of: Box::new(type_of),
            ..SchemaField::default()
        }
    }

    /// Create a new field from a [`Schema`].
    pub fn from_schema(name: &str, schema: Schema) -> SchemaField {
        SchemaField {
            name: name.to_owned(),
            description: schema.description,
            type_of: Box::new(schema.type_of),
            type_name: schema.name,
            ..SchemaField::default()
        }
    }
}
