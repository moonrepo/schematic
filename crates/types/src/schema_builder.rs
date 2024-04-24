use crate::*;

#[derive(Clone, Debug, Default)]
pub struct SchemaBuilder {
    pub description: Option<String>,
    pub name: Option<String>,
    pub type_of: SchemaType,
}

impl SchemaBuilder {
    /// Add a description for this schema.
    pub fn description(&mut self, value: impl AsRef<str>) -> &mut Self {
        self.description = Some(value.as_ref().to_owned());
        self
    }

    /// Add a name for this schema.
    pub fn name(&mut self, value: impl AsRef<str>) -> &mut Self {
        self.name = Some(value.as_ref().to_owned());
        self
    }

    /// Build an array type.
    pub fn array(&mut self, value: ArrayType) -> &mut Self {
        self.type_of = SchemaType::Array(Box::new(value));
        self
    }

    /// Build a boolean type.
    pub fn boolean(&mut self, value: BooleanType) -> &mut Self {
        self.type_of = SchemaType::Boolean(Box::new(value));
        self
    }

    /// Build an enum type.
    pub fn enumerable(&mut self, value: EnumType) -> &mut Self {
        self.type_of = SchemaType::Enum(Box::new(value));
        self
    }

    /// Build a float type.
    pub fn float(&mut self, value: FloatType) -> &mut Self {
        self.type_of = SchemaType::Float(Box::new(value));
        self
    }

    /// Build an integer type.
    pub fn integer(&mut self, value: IntegerType) -> &mut Self {
        self.type_of = SchemaType::Integer(Box::new(value));
        self
    }

    /// Build a literal type.
    pub fn literal(&mut self, value: LiteralType) -> &mut Self {
        self.type_of = SchemaType::Literal(Box::new(value));
        self
    }

    /// Build an object type.
    pub fn object(&mut self, value: ObjectType) -> &mut Self {
        self.type_of = SchemaType::Object(Box::new(value));
        self
    }

    /// Build a string type.
    pub fn string(&mut self, value: StringType) -> &mut Self {
        self.type_of = SchemaType::String(Box::new(value));
        self
    }

    /// Build a struct type.
    pub fn structure(&mut self, value: StructType) -> &mut Self {
        self.type_of = SchemaType::Struct(Box::new(value));
        self
    }

    /// Build a tuple type.
    pub fn tuple(&mut self, value: TupleType) -> &mut Self {
        self.type_of = SchemaType::Tuple(Box::new(value));
        self
    }

    /// Build a union type.
    pub fn union(&mut self, value: UnionType) -> &mut Self {
        self.type_of = SchemaType::Union(Box::new(value));
        self
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
}
