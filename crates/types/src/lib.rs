mod arrays;
mod bools;
mod enums;
mod externals;
mod literals;
mod numbers;
mod objects;
mod schema;
mod strings;
mod structs;
mod tuples;
mod unions;

pub use arrays::*;
pub use bools::*;
pub use enums::*;
pub use literals::*;
pub use numbers::*;
pub use objects::*;
pub use schema::*;
pub use strings::*;
pub use structs::*;
pub use tuples::*;
pub use unions::*;

/// All possible types within a schema.
#[derive(Clone, Debug, Default)]
pub enum SchemaType {
    Null,
    #[default]
    Unknown,
    Array(ArrayType),
    Boolean(BooleanType),
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

    /// Infer a schema from a type that implements [`Schematic`],
    /// and also provide a default literal value.
    pub fn infer_with_default<T: Schematic>(default: LiteralValue) -> SchemaType {
        let mut schema = T::generate_schema();
        schema.set_default(default);
        schema
    }

    /// Infer a schema from a type that implements [`Schematic`],
    /// and mark the schema is partial (is marked as `nested`).
    pub fn infer_partial<T: Schematic>() -> SchemaType {
        let mut schema = T::generate_schema();
        schema.set_partial(true);
        schema
    }

    /// Create an array schema with the provided item types.
    pub fn array(items_type: SchemaType) -> SchemaType {
        SchemaType::Array(ArrayType {
            items_type: Box::new(items_type),
            ..ArrayType::default()
        })
    }

    /// Create a boolean type.
    pub fn boolean() -> SchemaType {
        SchemaType::Boolean(BooleanType::default())
    }

    /// Create an enumerable type with the provided literal values.
    pub fn enumerable<I>(values: I) -> SchemaType
    where
        I: IntoIterator<Item = LiteralValue>,
    {
        SchemaType::Enum(EnumType {
            values: values.into_iter().collect(),
            ..EnumType::default()
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

    /// Convert the provided schema to a nullable type. If already nullable,
    /// do nothing and return, otherwise convert to a union.
    pub fn nullable(mut schema: SchemaType) -> SchemaType {
        if let SchemaType::Union(inner) = &mut schema {
            // If the union has an explicit name, then we can assume it's a distinct
            // type, so we shouldn't add null to it and alter the intended type.
            // if inner.name.is_none() {
            //     if !inner
            //         .variants_types
            //         .iter()
            //         .any(|t| matches!(**t, SchemaType::Null))
            //     {
            //         inner.variants_types.push(Box::new(SchemaType::Null));
            //     }

            //     return schema;
            // }
        }

        // Convert to a nullable union
        SchemaType::union([schema, SchemaType::Null])
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

    /// Return a `default` value from the inner schema type.
    pub fn get_default(&self) -> Option<&LiteralValue> {
        match self {
            SchemaType::Boolean(BooleanType { default, .. }) => default.as_ref(),
            SchemaType::Enum(EnumType {
                default_index,
                values,
                ..
            }) => {
                if let Some(index) = default_index {
                    if let Some(value) = values.get(*index) {
                        return Some(value);
                    }
                }

                None
            }
            SchemaType::Float(FloatType { default, .. }) => default.as_ref(),
            SchemaType::Integer(IntegerType { default, .. }) => default.as_ref(),
            SchemaType::String(StringType { default, .. }) => default.as_ref(),
            SchemaType::Union(UnionType {
                default_index,
                variants_types,
                ..
            }) => {
                if let Some(index) = default_index {
                    if let Some(value) = variants_types.get(*index) {
                        return value.get_default();
                    }
                }

                for variant in variants_types {
                    if let Some(value) = variant.get_default() {
                        return Some(value);
                    }
                }

                None
            }
            _ => None,
        }
    }

    /// Return a `name` from the inner schema type.
    pub fn get_name(&self) -> Option<&String> {
        // match self {
        //     SchemaType::Null => None,
        //     SchemaType::Unknown => None,
        //     SchemaType::Array(ArrayType { name, .. }) => name.as_ref(),
        //     SchemaType::Boolean(BooleanType { name, .. }) => name.as_ref(),
        //     SchemaType::Enum(EnumType { name, .. }) => name.as_ref(),
        //     SchemaType::Float(FloatType { name, .. }) => name.as_ref(),
        //     SchemaType::Integer(IntegerType { name, .. }) => name.as_ref(),
        //     SchemaType::Literal(LiteralType { name, .. }) => name.as_ref(),
        //     SchemaType::Object(ObjectType { name, .. }) => name.as_ref(),
        //     SchemaType::Struct(StructType { name, .. }) => name.as_ref(),
        //     SchemaType::String(StringType { name, .. }) => name.as_ref(),
        //     SchemaType::Tuple(TupleType { name, .. }) => name.as_ref(),
        //     SchemaType::Union(UnionType { name, .. }) => name.as_ref(),
        // }
        None
    }

    /// Return true if the schema is an explicit null.
    pub fn is_null(&self) -> bool {
        matches!(self, SchemaType::Null)
    }

    /// Return true if the schema is nullable (a union with a null).
    pub fn is_nullable(&self) -> bool {
        if let SchemaType::Union(uni) = self {
            return uni.is_nullable();
        }

        false
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

    /// Set the `name` of the inner schema type. If the inner type does not support
    /// names, this is a no-op.
    pub fn set_name<S: AsRef<str>>(&mut self, _name: S) {
        // let name = Some(name.as_ref().to_owned());

        // match self {
        //     SchemaType::Array(ref mut inner) => {
        //         inner.name = name;
        //     }
        //     SchemaType::Enum(ref mut inner) => {
        //         inner.name = name;
        //     }
        //     SchemaType::Float(ref mut inner) => {
        //         inner.name = name;
        //     }
        //     SchemaType::Integer(ref mut inner) => {
        //         inner.name = name;
        //     }
        //     SchemaType::Literal(ref mut inner) => {
        //         inner.name = name;
        //     }
        //     SchemaType::Object(ref mut inner) => {
        //         inner.name = name;
        //     }
        //     SchemaType::Struct(ref mut inner) => {
        //         inner.name = name;
        //     }
        //     SchemaType::String(ref mut inner) => {
        //         inner.name = name;
        //     }
        //     SchemaType::Tuple(ref mut inner) => {
        //         inner.name = name;
        //     }
        //     SchemaType::Union(ref mut inner) => {
        //         inner.name = name;
        //     }
        //     _ => {}
        // };
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
                let is_nullable = inner
                    .variants_types
                    .iter()
                    .any(|t| matches!(**t, SchemaType::Null));

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
    pub fn add_field(&mut self, field: SchemaField) {
        match self {
            SchemaType::Enum(ref mut inner) => {
                inner.variants.get_or_insert(vec![]).push(field);
            }
            SchemaType::Struct(ref mut inner) => {
                inner.fields.push(field);
            }
            SchemaType::Union(ref mut inner) => {
                inner.variants.get_or_insert(vec![]).push(field);
            }
            _ => {}
        };
    }
}

/// Defines a schema that represents the shape of the implementing type.
pub trait Schematic {
    /// Create and return a schema that models the structure of the implementing type.
    /// The schema can be used to generate code, documentation, or other artifacts.
    fn generate_schema() -> SchemaType {
        SchemaType::Unknown
    }
}

// CORE

impl Schematic for () {
    fn generate_schema() -> SchemaType {
        SchemaType::Null
    }
}

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
        SchemaType::nullable(T::generate_schema())
    }
}
