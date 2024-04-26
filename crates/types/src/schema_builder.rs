use crate::*;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

#[derive(Clone, Debug, Default)]
pub struct SchemaBuilder {
    pub description: Option<String>,
    pub name: Option<String>,
    pub type_of: SchemaType,

    existing_names: Rc<RefCell<HashSet<String>>>,
}

impl SchemaBuilder {
    /// Generate a schema from the provided type.
    pub fn build_root<T: Schematic>() -> Schema {
        let mut builder = SchemaBuilder::default();

        if let Some(name) = T::schema_name() {
            builder.set_name(name);
        }

        T::build_schema(builder)
    }

    /// Build the schema from the configured values.
    pub fn build(self) -> Schema {
        Schema {
            description: self.description,
            name: self.name,
            type_of: self.type_of,
        }
    }

    /// Add a description for this schema.
    pub fn set_description(&mut self, value: impl AsRef<str>) {
        self.description = Some(value.as_ref().to_owned());
    }

    /// Add a name for this schema.
    pub fn set_name(&mut self, value: impl AsRef<str>) {
        let name = value.as_ref();

        self.name = Some(name.to_owned());
        self.existing_names.borrow_mut().insert(name.to_owned());
    }

    /// Build an array type.
    pub fn array(&mut self, value: ArrayType) {
        self.custom(SchemaType::Array(Box::new(value)));
    }

    /// Build a boolean type.
    pub fn boolean(&mut self, value: BooleanType) {
        self.custom(SchemaType::Boolean(Box::new(value)));
    }

    /// Build with a custom type.
    pub fn custom(&mut self, value: SchemaType) {
        self.type_of = value;
    }

    /// Build an enum type.
    pub fn enumerable(&mut self, value: EnumType) {
        self.custom(SchemaType::Enum(Box::new(value)));
    }

    /// Build a float type.
    pub fn float(&mut self, value: FloatType) {
        self.custom(SchemaType::Float(Box::new(value)));
    }

    /// Build an integer type.
    pub fn integer(&mut self, value: IntegerType) {
        self.custom(SchemaType::Integer(Box::new(value)));
    }

    /// Build a literal type.
    pub fn literal(&mut self, value: LiteralType) {
        self.custom(SchemaType::Literal(Box::new(value)));
    }

    /// Build an object type.
    pub fn object(&mut self, value: ObjectType) {
        self.custom(SchemaType::Object(Box::new(value)));
    }

    /// Build a string type.
    pub fn string(&mut self, value: StringType) {
        self.custom(SchemaType::String(Box::new(value)));
    }

    /// Build a struct type.
    pub fn structure(&mut self, value: StructType) {
        self.custom(SchemaType::Struct(Box::new(value)));
    }

    /// Build a tuple type.
    pub fn tuple(&mut self, value: TupleType) {
        self.custom(SchemaType::Tuple(Box::new(value)));
    }

    /// Build a union type.
    pub fn union(&mut self, value: UnionType) {
        self.custom(SchemaType::Union(Box::new(value)));
    }

    /// Convert the current schema to a nullable type. If already nullable,
    /// do nothing and return, otherwise convert to a union.
    pub fn nullable(&mut self) {
        if let SchemaType::Union(inner) = &mut self.type_of {
            // If the union has an explicit name, then we can assume it's a distinct
            // type, so we shouldn't add null to it and alter the intended type.
            if self.name.is_none() {
                if !inner.has_null() {
                    inner.variants_types.push(Box::new(Schema::null()));
                }

                return;
            }
        }

        // Convert to a nullable union
        let current_type = std::mem::replace(&mut self.type_of, SchemaType::Unknown);

        self.union(UnionType::new_any([
            Schema::new(current_type),
            Schema::null(),
        ]));
    }

    /// Infer a [`Schema`] from a type that implements [`Schematic`].
    pub fn infer<T: Schematic>(&self) -> Schema {
        let mut builder = SchemaBuilder::default();
        builder.existing_names = Rc::clone(&self.existing_names);

        // No name, so return the schema immediately
        let Some(name) = T::schema_name() else {
            return T::build_schema(builder);
        };

        // If this name has already been used, create a reference
        // so that we avoid recursion!
        if self.existing_names.borrow().contains(&name) {
            builder.custom(SchemaType::Reference(name));

            return builder.build();
        }

        // Otherwise generate a new schema and persist our name cache
        builder.set_name(&name);

        T::build_schema(builder)
    }

    /// Infer a [`SchemaType`] from a type that implements [`Schematic`].
    pub fn infer_type<T: Schematic>(&self) -> SchemaType {
        self.infer::<T>().type_of
    }

    /// Infer a [`SchemaType`] from a type that implements [`Schematic`],
    /// and mark the schema is partial (is marked as `nested`).
    pub fn infer_type_as_partial<T: Schematic>(&self) -> SchemaType {
        let mut schema = self.infer_type::<T>();
        schema.set_partial(true);
        schema
    }

    /// Infer a [`SchemaType`] from a type that implements [`Schematic`],
    /// and also provide a default literal value.
    pub fn infer_type_with_default<T: Schematic>(&self, default: LiteralValue) -> SchemaType {
        let mut schema = self.infer_type::<T>();
        schema.set_default(default);
        schema
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

impl Into<Schema> for SchemaBuilder {
    fn into(self) -> Schema {
        self.build()
    }
}
