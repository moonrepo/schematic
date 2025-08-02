use crate::*;
use std::fmt;
use std::ops::{Deref, DerefMut};

/// Describes the metadata and shape of a type.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Schema {
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub deprecated: Option<String>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub description: Option<String>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub name: Option<String>,

    #[cfg_attr(feature = "serde", serde(default, skip_serializing))]
    pub nullable: bool,

    pub ty: SchemaType,
}

impl Schema {
    /// Create a schema with the provided type.
    pub fn new(ty: impl Into<SchemaType>) -> Self {
        Self {
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

    /// Convert the current schema to a nullable type. If already nullable,
    /// do nothing, otherwise convert to a union.
    pub fn nullify(&mut self) {
        if self.nullable {
            // May already be a null union through inference
            return;
        }

        self.nullable = true;

        if let SchemaType::Union(inner) = &mut self.ty {
            // If the union has an explicit name, then we can assume it's a distinct
            // type, so we shouldn't add null to it and alter the intended type.
            if self.name.is_none() {
                if !inner.variants_types.iter().any(|t| t.is_null()) {
                    inner.variants_types.push(Box::new(Schema::null()));
                }

                return;
            }
        }

        // Convert to a nullable union
        let mut new_schema = Schema::new(std::mem::replace(&mut self.ty, SchemaType::Unknown));
        new_schema.name = self.name.take();
        new_schema.description.clone_from(&self.description);
        new_schema.deprecated.clone_from(&self.deprecated);

        self.ty = SchemaType::Union(Box::new(UnionType::new_any([new_schema, Schema::null()])));
    }

    /// Mark the inner schema type as partial. Only structs and unions can be marked partial,
    /// but arrays and objects will also be recursively set to update the inner type.
    pub fn partialize(&mut self) {
        match &mut self.ty {
            SchemaType::Array(inner) => inner.items_type.partialize(),
            SchemaType::Object(inner) => inner.value_type.partialize(),
            SchemaType::Struct(inner) => {
                inner.partial = true;
            }
            SchemaType::Union(inner) => {
                inner.partial = true;

                // This is to handle things wrapped in `Option`, is it correct?
                // Not sure of a better way to do this at the moment...
                let is_nullable = inner.variants_types.iter().any(|t| t.ty.is_null());

                if is_nullable {
                    for item in inner.variants_types.iter_mut() {
                        if !item.is_null() {
                            item.partialize();
                        }
                    }
                }
            }
            _ => {}
        };
    }

    /// Mark this schema as deprecated.
    pub fn set_deprecated(&mut self, value: impl AsRef<str>) {
        self.deprecated = Some(value.as_ref().to_owned());
    }

    /// Add a description for this schema.
    pub fn set_description(&mut self, value: impl AsRef<str>) {
        self.description = Some(value.as_ref().to_owned());
    }

    /// Add a name for this schema.
    pub fn set_name(&mut self, value: impl AsRef<str>) {
        let name = value.as_ref();

        self.name = Some(name.to_owned());
    }

    /// Set the type of schema.
    pub fn set_type(&mut self, value: SchemaType) {
        self.ty = value;
    }
}

impl fmt::Display for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.name.is_some() && (self.ty.is_struct() || self.ty.is_reference()) {
            write!(f, "{}", self.name.as_ref().unwrap())
        } else {
            write!(f, "{}", self.ty)
        }
    }
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

impl From<Schema> for SchemaType {
    fn from(val: Schema) -> Self {
        val.ty
    }
}

impl Schematic for Schema {}

#[cfg(feature = "serde")]
fn is_false(value: &bool) -> bool {
    !value
}

/// Describes the metadata and shape of a field within a struct or enum.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SchemaField {
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub aliases: Vec<String>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub comment: Option<String>,

    pub schema: Schema,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub deprecated: Option<String>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub env_var: Option<String>,

    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_false"))]
    pub hidden: bool,

    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_false"))]
    pub nullable: bool,

    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_false"))]
    pub optional: bool,

    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_false"))]
    pub read_only: bool,

    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "is_false"))]
    pub write_only: bool,
}

impl SchemaField {
    pub fn new(schema: impl Into<Schema>) -> Self {
        Self {
            schema: schema.into(),
            ..Default::default()
        }
    }
}

impl From<SchemaField> for Schema {
    fn from(val: SchemaField) -> Self {
        val.schema
    }
}

impl From<Schema> for SchemaField {
    fn from(mut schema: Schema) -> Self {
        SchemaField {
            comment: schema.description.take(),
            schema,
            ..Default::default()
        }
    }
}

impl Schematic for SchemaField {}
