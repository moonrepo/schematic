use crate::Schematic;
use crate::arrays::*;
use crate::bools::*;
use crate::enums::*;
use crate::literals::*;
use crate::numbers::*;
use crate::objects::*;
use crate::schema::*;
use crate::strings::*;
use crate::structs::*;
use crate::tuples::*;
use crate::unions::*;

/// All possible types within a schema.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum SchemaType {
    Null,
    #[default]
    Unknown,
    Array(Box<ArrayType>),
    Boolean(Box<BooleanType>),
    Enum(Box<EnumType>),
    Float(Box<FloatType>),
    Integer(Box<IntegerType>),
    Literal(Box<LiteralType>),
    Object(Box<ObjectType>),
    Reference(String),
    Struct(Box<StructType>),
    String(Box<StringType>),
    Tuple(Box<TupleType>),
    Union(Box<UnionType>),
}

impl SchemaType {
    /// Return a `default` value from the inner schema type.
    pub fn get_default(&self) -> Option<&LiteralValue> {
        match self {
            SchemaType::Boolean(inner) => inner.default.as_ref(),
            SchemaType::Enum(inner) => {
                if let Some(index) = &inner.default_index {
                    if let Some(value) = inner.values.get(*index) {
                        return Some(value);
                    }
                }

                None
            }
            SchemaType::Float(inner) => inner.default.as_ref(),
            SchemaType::Integer(inner) => inner.default.as_ref(),
            SchemaType::String(inner) => inner.default.as_ref(),
            SchemaType::Union(inner) => {
                if let Some(index) = &inner.default_index {
                    if let Some(value) = inner.variants_types.get(*index) {
                        return value.get_default();
                    }
                }

                for variant in &inner.variants_types {
                    if let Some(value) = variant.get_default() {
                        return Some(value);
                    }
                }

                None
            }
            _ => None,
        }
    }

    /// Return true if the schema is an explicit null.
    pub fn is_null(&self) -> bool {
        matches!(self, SchemaType::Null)
    }

    /// Return true if the schema is nullable (a union with a null).
    pub fn is_nullable(&self) -> bool {
        if let SchemaType::Union(uni) = self {
            return uni.has_null();
        }

        false
    }

    /// Return true if the schema is a reference.
    pub fn is_reference(&self) -> bool {
        matches!(self, SchemaType::Reference(_))
    }

    /// Return true if the schema is a struct.
    pub fn is_struct(&self) -> bool {
        matches!(self, SchemaType::Struct(_))
    }

    /// Set the `default` of the inner schema type.
    pub fn set_default(&mut self, default: LiteralValue) {
        match self {
            SchemaType::Boolean(inner) => {
                inner.default = Some(default);
            }
            SchemaType::Float(inner) => {
                inner.default = Some(default);
            }
            SchemaType::Integer(inner) => {
                inner.default = Some(default);
            }
            SchemaType::String(inner) => {
                inner.default = Some(default);
            }
            _ => {}
        };
    }

    /// Add a field to the type if it's a struct.
    pub fn add_field(&mut self, key: &str, value: impl Into<SchemaField>) {
        if let SchemaType::Struct(map) = self {
            map.fields.insert(key.to_owned(), Box::new(value.into()));
        }
    }
}

impl From<SchemaType> for Schema {
    fn from(val: SchemaType) -> Self {
        Schema::new(val)
    }
}

impl Schematic for SchemaType {}
