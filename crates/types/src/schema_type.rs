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
            SchemaType::Boolean(ref mut inner) => {
                inner.default = Some(default);
            }
            SchemaType::Float(ref mut inner) => {
                inner.default = Some(default);
            }
            SchemaType::Integer(ref mut inner) => {
                inner.default = Some(default);
            }
            SchemaType::String(ref mut inner) => {
                inner.default = Some(default);
            }
            _ => {}
        };
    }

    /// Mark the inner schema type as partial. Only structs and unions can be marked partial,
    /// but arrays and objects will also be recursively set to update the inner type.
    pub fn set_partial(&mut self, state: bool) {
        match self {
            SchemaType::Array(ref mut inner) => inner.items_type.set_partial(state),
            SchemaType::Object(ref mut inner) => inner.value_type.set_partial(state),
            SchemaType::Struct(ref mut inner) => {
                inner.partial = state;
            }
            SchemaType::Union(ref mut inner) => {
                inner.partial = state;

                // This is to handle things wrapped in `Option`, is it correct?
                // Not sure of a better way to do this at the moment...
                let is_nullable = inner.variants_types.iter().any(|t| t.ty.is_nullable());

                if is_nullable {
                    for item in inner.variants_types.iter_mut() {
                        item.set_partial(state);
                    }
                }
            }
            _ => {}
        };
    }

    /// Add the field to the inner schema type. This is only applicable to enums, structs,
    /// and unions, otherwise this is a no-op.
    pub fn add_field(&mut self, field: Schema) {
        match self {
            SchemaType::Enum(ref mut inner) => {
                inner.variants.get_or_insert(vec![]).push(Box::new(field));
            }
            SchemaType::Struct(ref mut inner) => {
                inner.fields.push(Box::new(field));
            }
            SchemaType::Union(ref mut inner) => {
                inner.variants.get_or_insert(vec![]).push(Box::new(field));
            }
            _ => {}
        };
    }
}

impl Into<Schema> for SchemaType {
    fn into(self) -> Schema {
        Schema::new(self)
    }
}
