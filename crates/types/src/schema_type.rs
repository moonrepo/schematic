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
use std::fmt;

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
    // A struct variant, and not a newtype, as serde is unable to
    // internally tag a newtype whose value is not a map.
    Reference {
        name: String,

        // The name refers to a partial config type, like `StructType.partial`.
        // Set by `Schema::partialize`, consumed when the schema is partialized
        // at runtime and the name gains its `Partial` prefix.
        #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_false"))]
        partial: bool,
    },
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
            SchemaType::Enum(inner) => inner.get_default(),
            SchemaType::Float(inner) => inner.default.as_ref(),
            SchemaType::Integer(inner) => inner.default.as_ref(),
            SchemaType::String(inner) => inner.default.as_ref(),
            SchemaType::Union(inner) => {
                if let Some(index) = &inner.default_index
                    && let Some(value) = inner.variants_types.get(*index)
                {
                    return value.get_default();
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
        matches!(self, SchemaType::Reference { .. })
    }

    /// Return true if the schema is a struct.
    pub fn is_struct(&self) -> bool {
        matches!(self, SchemaType::Struct(_))
    }

    /// Set the `default` of the inner schema type. Mirrors [`Self::get_default`],
    /// so unions delegate to the variant that would be read back, and enums
    /// record the position of the matching value.
    ///
    /// Returns false when the type holds no default, as most don't — a struct
    /// or an array has nowhere to put one.
    pub fn set_default(&mut self, default: LiteralValue) -> bool {
        match self {
            SchemaType::Boolean(inner) => {
                inner.default = Some(default);
            }
            SchemaType::Enum(inner) => {
                return inner.set_default(default);
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
            // A union itself holds no value, so push the default down into the
            // variant it represents. This is how `Option<T>` keeps its default.
            SchemaType::Union(inner) => {
                let index = inner.default_index.unwrap_or_else(|| {
                    inner
                        .variants_types
                        .iter()
                        .position(|variant| !variant.is_null())
                        .unwrap_or(0)
                });

                return inner
                    .variants_types
                    .get_mut(index)
                    .is_some_and(|variant| variant.set_default(default));
            }
            _ => return false,
        };

        true
    }

    /// Add a field to the type if it's a struct. Returns false when it isn't
    /// one, as there is nowhere to put the field.
    pub fn add_field(&mut self, key: &str, value: impl Into<SchemaField>) -> bool {
        let SchemaType::Struct(map) = self else {
            return false;
        };

        map.fields.insert(key.to_owned(), Box::new(value.into()));

        true
    }
}

impl From<SchemaType> for Schema {
    fn from(val: SchemaType) -> Self {
        Schema::new(val)
    }
}

impl Schematic for SchemaType {}

impl fmt::Display for SchemaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Null => "null".to_string(),
                Self::Unknown => "unknown".to_string(),
                Self::Array(inner) => inner.to_string(),
                Self::Boolean(inner) => inner.to_string(),
                Self::Enum(inner) => inner.to_string(),
                Self::Float(inner) => inner.to_string(),
                Self::Integer(inner) => inner.to_string(),
                Self::Literal(inner) => inner.to_string(),
                Self::Object(inner) => inner.to_string(),
                Self::Reference { name, .. } => name.to_owned(),
                Self::Struct(inner) => inner.to_string(),
                Self::String(inner) => inner.to_string(),
                Self::Tuple(inner) => inner.to_string(),
                Self::Union(inner) => inner.to_string(),
            }
        )
    }
}
