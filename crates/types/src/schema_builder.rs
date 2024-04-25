use crate::*;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Default)]
pub struct SchemaBuilder {
    pub description: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub type_of: SchemaType,
}

impl SchemaBuilder {
    /// Build the schema from the configured values.
    pub fn build(self) -> Schema {
        Schema {
            description: self.description,
            name: self.name,
            type_of: self.type_of,
        }
    }

    /// Add a description for this schema.
    pub fn set_description(&mut self, value: impl AsRef<str>) -> &mut Self {
        self.description = Some(value.as_ref().to_owned());
        self
    }

    /// Add a name for this schema.
    pub fn set_name(&mut self, value: impl AsRef<str>) -> &mut Self {
        self.name = Some(value.as_ref().to_owned());
        self
    }

    /// Build an array type.
    pub fn array(&mut self, value: ArrayType) -> &mut Self {
        self.custom(SchemaType::Array(Box::new(value)))
    }

    /// Build a boolean type.
    pub fn boolean(&mut self, value: BooleanType) -> &mut Self {
        self.custom(SchemaType::Boolean(Box::new(value)))
    }

    /// Build with a custom type.
    pub fn custom(&mut self, value: SchemaType) -> &mut Self {
        self.type_of = value;
        self
    }

    /// Build an enum type.
    pub fn enumerable(&mut self, value: EnumType) -> &mut Self {
        self.custom(SchemaType::Enum(Box::new(value)))
    }

    /// Build a float type.
    pub fn float(&mut self, value: FloatType) -> &mut Self {
        self.custom(SchemaType::Float(Box::new(value)))
    }

    /// Build an integer type.
    pub fn integer(&mut self, value: IntegerType) -> &mut Self {
        self.custom(SchemaType::Integer(Box::new(value)))
    }

    /// Build a literal type.
    pub fn literal(&mut self, value: LiteralType) -> &mut Self {
        self.custom(SchemaType::Literal(Box::new(value)))
    }

    /// Build an object type.
    pub fn object(&mut self, value: ObjectType) -> &mut Self {
        self.custom(SchemaType::Object(Box::new(value)))
    }

    /// Build a string type.
    pub fn string(&mut self, value: StringType) -> &mut Self {
        self.custom(SchemaType::String(Box::new(value)))
    }

    /// Build a struct type.
    pub fn structure(&mut self, value: StructType) -> &mut Self {
        self.custom(SchemaType::Struct(Box::new(value)))
    }

    /// Build a tuple type.
    pub fn tuple(&mut self, value: TupleType) -> &mut Self {
        self.custom(SchemaType::Tuple(Box::new(value)))
    }

    /// Build a union type.
    pub fn union(&mut self, value: UnionType) -> &mut Self {
        self.custom(SchemaType::Union(Box::new(value)))
    }

    /// Convert the current schema to a nullable type. If already nullable,
    /// do nothing and return, otherwise convert to a union.
    pub fn nullable(&mut self) -> &mut Self {
        if let SchemaType::Union(inner) = &mut self.type_of {
            // If the union has an explicit name, then we can assume it's a distinct
            // type, so we shouldn't add null to it and alter the intended type.
            if self.name.is_none() {
                if !inner.has_null() {
                    inner.variants_types.push(Box::new(SchemaType::Null));
                }

                return self;
            }
        }

        // Convert to a nullable union
        let current_type = std::mem::replace(&mut self.type_of, SchemaType::Unknown);

        self.union(UnionType::new_any([current_type, SchemaType::Null]))
    }

    /// Infer a schema from a type that implements [`Schematic`].
    pub fn infer<T: Schematic>(&self) -> SchemaType {
        self.internal_infer::<T>().type_of
    }

    /// Infer a schema from a type that implements [`Schematic`],
    /// and mark the schema is partial (is marked as `nested`).
    pub fn infer_as_partial<T: Schematic>(&self) -> SchemaType {
        let mut schema = self.infer::<T>();
        schema.set_partial(true);
        schema
    }

    /// Infer a schema from a type that implements [`Schematic`],
    /// and also provide a default literal value.
    pub fn infer_with_default<T: Schematic>(&self, default: LiteralValue) -> SchemaType {
        let mut schema = self.infer::<T>();
        schema.set_default(default);
        schema
    }

    fn internal_infer<T: Schematic>(&self) -> Schema {
        let id = T::schema_id();
        let mut builder = SchemaBuilder::default();

        if id == self.id {
            assert!(
                self.name.is_some(),
                "Self-referencing types require a schema name."
            );

            builder.custom(SchemaType::Reference(self.name.clone().unwrap()));
            builder.build()
        } else {
            T::generate_schema(builder)
        }
    }
}

impl Deref for SchemaBuilder {
    type Target = SchemaType;

    fn deref(&self) -> &Self::Target {
        &self.type_of
    }
}

impl DerefMut for SchemaBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.type_of
    }
}
