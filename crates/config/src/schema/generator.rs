use schematic_types::*;
use std::collections::HashSet;

pub struct SchemaGenerator {
    references: HashSet<String>,
    schemas: Vec<SchemaType>,
}

impl SchemaGenerator {
    pub fn add(&mut self, schema: SchemaType) {
        let maybe_name = match &schema {
            SchemaType::Boolean => None,
            SchemaType::Null => None,
            SchemaType::Unknown => None,
            SchemaType::Array(ArrayType { name, .. }) => name.as_ref(),
            SchemaType::Float(FloatType { name, .. }) => name.as_ref(),
            SchemaType::Integer(IntegerType { name, .. }) => name.as_ref(),
            SchemaType::Literal(LiteralType { name, .. }) => name.as_ref(),
            SchemaType::Object(ObjectType { name, .. }) => name.as_ref(),
            SchemaType::Struct(StructType { name, .. }) => name.as_ref(),
            SchemaType::String(StringType { name, .. }) => name.as_ref(),
            SchemaType::Tuple(TupleType { name, .. }) => name.as_ref(),
            SchemaType::Union(UnionType { name, .. }) => name.as_ref(),
        };

        // Store the name so that we can use it as a reference for other types
        if let Some(name) = maybe_name {
            // Type has already been added
            if self.references.contains(name) {
                return;
            }

            self.references.insert(name.clone());

        // Types without a name cannot be rendered at the root
        } else {
            return;
        }

        // Recursively add any nested schema types
        match &schema {
            SchemaType::Array(ArrayType { items_type, .. }) => {
                self.add(*(*items_type).clone());
            }
            SchemaType::Object(ObjectType {
                key_type,
                value_type,
                ..
            }) => {
                self.add(*(*key_type).clone());
                self.add(*(*value_type).clone());
            }
            SchemaType::Struct(StructType { fields, .. }) => {
                for field in fields {
                    self.add(field.type_of.clone());
                }
            }
            SchemaType::Tuple(TupleType { items_types, .. }) => {
                for item in items_types {
                    self.add(*(*item).clone());
                }
            }
            SchemaType::Union(UnionType { variants_types, .. }) => {
                for variant in variants_types {
                    self.add(*(*variant).clone());
                }
            }
            _ => {}
        };

        self.schemas.push(schema);
    }
}
