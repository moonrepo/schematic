use crate::*;
use std::ops::{Deref, DerefMut};

/// Represents an individual type, a container, field within a schema struct,
/// or a variant within a schema enum/union.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Schema {
    pub description: Option<String>,
    pub name: Option<String>,
    pub ty: SchemaType,

    // States
    pub deprecated: Option<String>,
    pub env_var: Option<String>,
    pub hidden: bool,
    pub nullable: bool,
    pub optional: bool,
    pub read_only: bool,
    pub write_only: bool,
}

impl Schema {
    /// Create a schema with the provided type.
    pub fn new(ty: SchemaType) -> Self {
        Self {
            ty,
            ..Default::default()
        }
    }

    /// Create a schema field with the provided type.
    pub fn new_field(name: impl AsRef<str>, ty: impl Into<SchemaType>) -> Self {
        Self {
            name: Some(name.as_ref().to_owned()),
            ty: ty.into(),
            ..Default::default()
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

    /// Create a literal schema with the provided value.
    pub fn literal_value(value: LiteralValue) -> Self {
        Self::new(SchemaType::Literal(Box::new(LiteralType::new(value))))
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

    // pub fn make_nullable(&mut self) {
    //     if let SchemaType::Union(inner) = &mut self.ty {
    //         // If the union has an explicit name, then we can assume it's a distinct
    //         // type, so we shouldn't add null to it and alter the intended type.
    //         if self.name.is_none() {
    //             if !inner.has_null() {
    //                 inner.variants_types.push(Box::new(Schema::null()));
    //             }

    //             return;
    //         }
    //     }

    //     // Convert to a nullable union
    //     let current_type = std::mem::replace(&mut self.ty, SchemaType::Unknown);

    //     self.ty = SchemaType::Union(Box::new(UnionType::new_any([
    //         Schema::new(current_type),
    //         Schema::null(),
    //     ])));
    // }
}

impl Deref for Schema {
    type Target = SchemaType;

    fn deref(&self) -> &Self::Target {
        &self.ty
    }
}

impl DerefMut for Schema {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ty
    }
}

impl Into<SchemaType> for Schema {
    fn into(self) -> SchemaType {
        self.ty
    }
}
