use crate::*;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

#[derive(Clone, Debug, Default)]
pub struct SchemaBuilder {
    description: Option<String>,
    name: Option<String>,
    name_stack: Rc<RefCell<Vec<String>>>,
    ty: SchemaType,
    nullable: bool,
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
            nullable: self.nullable,
            ty: self.ty,
            ..Default::default()
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
        self.name_stack.borrow_mut().push(name.to_owned());
    }

    /// Build an array type.
    pub fn array(&mut self, value: ArrayType) {
        self.custom(SchemaType::Array(Box::new(value)));
    }

    /// Build a boolean type.
    pub fn boolean(&mut self, value: BooleanType) {
        self.custom(SchemaType::Boolean(Box::new(value)));
    }

    /// Build a boolean type with default settings.
    pub fn boolean_default(&mut self) {
        self.boolean(BooleanType::default())
    }

    /// Build with a custom type.
    pub fn custom(&mut self, value: SchemaType) {
        self.ty = value;
    }

    /// Build an enum type.
    pub fn enumerable(&mut self, value: EnumType) {
        self.custom(SchemaType::Enum(Box::new(value)));
    }

    /// Build a float type.
    pub fn float(&mut self, value: FloatType) {
        self.custom(SchemaType::Float(Box::new(value)));
    }

    /// Build a 32bit float type with default settings.
    pub fn float32_default(&mut self) {
        self.float(FloatType::new_kind(FloatKind::F32))
    }

    /// Build a 64bit float type with default settings.
    pub fn float64_default(&mut self) {
        self.float(FloatType::new_kind(FloatKind::F64))
    }

    /// Build an integer type.
    pub fn integer(&mut self, value: IntegerType) {
        self.custom(SchemaType::Integer(Box::new(value)));
    }

    /// Build a literal type.
    pub fn literal(&mut self, value: LiteralType) {
        self.custom(SchemaType::Literal(Box::new(value)));
    }

    /// Build a literal type with a value.
    pub fn literal_value(&mut self, value: LiteralValue) {
        self.literal(LiteralType::new(value))
    }

    /// Build a nested [`Schema`] by cloning another builder.
    pub fn nest(&self) -> SchemaBuilder {
        SchemaBuilder {
            description: None,
            name: None,
            name_stack: Rc::clone(&self.name_stack),
            ty: SchemaType::Unknown,
            nullable: false,
        }
    }

    /// Build a schema that is also nullable (uses a union).
    pub fn nullable(&mut self, value: impl Into<Schema>) {
        self.union(UnionType::new_any([value.into(), Schema::null()]))
    }

    /// Build an object type.
    pub fn object(&mut self, value: ObjectType) {
        self.custom(SchemaType::Object(Box::new(value)));
    }

    /// Build a string type.
    pub fn string(&mut self, value: StringType) {
        self.custom(SchemaType::String(Box::new(value)));
    }

    /// Build a string type with default settings.
    pub fn string_default(&mut self) {
        self.string(StringType::default())
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

    /// Infer a [`Schema`] from a type that implements [`Schematic`].
    pub fn infer<T: Schematic>(&self) -> Schema {
        let mut builder = self.nest();

        // No name, so return the schema immediately
        let Some(name) = T::schema_name() else {
            return T::build_schema(builder);
        };

        // If this name has already been used, create a reference
        // so that we avoid recursion!
        if self.name_stack.borrow().contains(&name) {
            builder.custom(SchemaType::Reference(name));

            return builder.build();
        }

        // Otherwise generate a new schema and persist our name cache
        builder.set_name(&name);

        let schema = T::build_schema(builder);

        self.name_stack.borrow_mut().pop();

        schema
    }

    /// Infer a [`Schema`] from a type that implements [`Schematic`],
    /// and mark the schema is partial (is marked as `nested`).
    pub fn infer_as_nested<T: Schematic>(&self) -> Schema {
        let mut schema = self.infer::<T>();
        schema.partialize();
        schema
    }

    /// Infer a [`Schema`] from a type that implements [`Schematic`],
    /// and also provide a default literal value.
    pub fn infer_with_default<T: Schematic>(&self, default: LiteralValue) -> Schema {
        let mut schema = self.infer::<T>();
        schema.set_default(default);
        schema
    }
}

impl Deref for SchemaBuilder {
    type Target = SchemaType;

    fn deref(&self) -> &Self::Target {
        &self.ty
    }
}

impl DerefMut for SchemaBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ty
    }
}

impl From<SchemaBuilder> for Schema {
    fn from(builder: SchemaBuilder) -> Self {
        builder.build()
    }
}
