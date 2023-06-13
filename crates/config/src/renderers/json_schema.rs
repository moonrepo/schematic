use crate::schema::{RenderResult, SchemaRenderer};
use schemars::schema::*;
use schematic_types::*;
use std::collections::{BTreeMap, BTreeSet, HashSet};

#[derive(Default)]
pub struct JsonSchemaOptions {}

/// Renders JsonSchema types from a schema.
#[derive(Default)]
pub struct JsonSchemaRenderer {
    options: JsonSchemaOptions,
    references: HashSet<String>,
}

impl JsonSchemaRenderer {
    pub fn new(options: JsonSchemaOptions) -> Self {
        Self {
            options,
            references: HashSet::new(),
        }
    }

    fn create_metadata(&self, name: Option<&String>) -> Option<Box<Metadata>> {
        if let Some(name) = name {
            Some(Box::new(Metadata {
                title: Some(name.to_owned()),
                ..Default::default()
            }))
        } else {
            None
        }
    }

    fn create_schema_from_field(&self, field: &SchemaField) -> RenderResult<Schema> {
        let mut schema = self.render_schema(&field.type_of)?;

        if let Schema::Object(ref mut inner) = schema {
            inner.metadata = Some(Box::new(Metadata {
                title: field.name.clone(),
                description: field.description.clone(),
                deprecated: field.deprecated,
                read_only: field.read_only,
                write_only: field.write_only,
                ..Default::default()
            }));
        }

        Ok(schema)
    }
}

impl SchemaRenderer<Schema> for JsonSchemaRenderer {
    fn is_reference(&self, name: &str) -> bool {
        self.references.contains(name)
    }

    fn render_array(&self, array: &ArrayType) -> RenderResult<Schema> {
        let data = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Array))),
            metadata: self.create_metadata(array.name.as_ref()),
            array: Some(Box::new(ArrayValidation {
                items: Some(SingleOrVec::Single(Box::new(
                    self.render_schema(&array.items_type)?,
                ))),
                max_items: array.max_length.map(|i| i as u32),
                min_items: array.min_length.map(|i| i as u32),
                unique_items: Some(array.unique),
                ..Default::default()
            })),
            ..Default::default()
        };

        Ok(Schema::Object(data))
    }

    fn render_boolean(&self) -> RenderResult<Schema> {
        Ok(Schema::Object(SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Boolean))),
            ..Default::default()
        }))
    }

    fn render_float(&self, float: &FloatType) -> RenderResult<Schema> {
        let data = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Number))),
            metadata: self.create_metadata(float.name.as_ref()),
            format: float.format.clone(),
            number: Some(Box::new(NumberValidation {
                exclusive_maximum: float.max_exclusive,
                exclusive_minimum: float.min_exclusive,
                maximum: float.max,
                minimum: float.min,
                multiple_of: float.multiple_of,
            })),
            ..Default::default()
        };

        Ok(Schema::Object(data))
    }

    fn render_integer(&self, integer: &IntegerType) -> RenderResult<Schema> {
        let data = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Number))),
            metadata: self.create_metadata(integer.name.as_ref()),
            format: integer.format.clone(),
            number: Some(Box::new(NumberValidation {
                exclusive_maximum: integer.max_exclusive.map(|i| i as f64),
                exclusive_minimum: integer.min_exclusive.map(|i| i as f64),
                maximum: integer.max.map(|i| i as f64),
                minimum: integer.min.map(|i| i as f64),
                multiple_of: integer.multiple_of.map(|i| i as f64),
            })),
            ..Default::default()
        };

        Ok(Schema::Object(data))
    }

    fn render_literal(&self, literal: &LiteralType) -> RenderResult<Schema> {
        // TODO?
        // if let Some(value) = &literal.value {
        //     return Ok(match value {
        //         LiteralValue::Bool(inner) => inner.to_string(),
        //         LiteralValue::Float(inner) => inner.to_owned(),
        //         LiteralValue::Int(inner) => inner.to_string(),
        //         LiteralValue::UInt(inner) => inner.to_string(),
        //         LiteralValue::String(inner) => format!("'{inner}'"),
        //     });
        // }

        self.render_unknown()
    }

    fn render_null(&self) -> RenderResult<Schema> {
        Ok(Schema::Object(SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Null))),
            ..Default::default()
        }))
    }

    fn render_object(&self, object: &ObjectType) -> RenderResult<Schema> {
        let data = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Object))),
            metadata: self.create_metadata(object.name.as_ref()),
            object: Some(Box::new(ObjectValidation {
                max_properties: object.max_fields.map(|i| i as u32),
                min_properties: object.min_fields.map(|i| i as u32),
                required: BTreeSet::from_iter(object.required.clone()),
                additional_properties: Some(Box::new(self.render_schema(&object.value_type)?)),
                property_names: Some(Box::new(self.render_schema(&object.key_type)?)),
                ..Default::default()
            })),
            ..Default::default()
        };

        Ok(Schema::Object(data))
    }

    fn render_reference(&self, reference: &str) -> RenderResult<Schema> {
        Ok(Schema::Object(SchemaObject {
            reference: Some(reference.into()),
            ..Default::default()
        }))
    }

    fn render_string(&self, string: &StringType) -> RenderResult<Schema> {
        let data = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
            metadata: self.create_metadata(string.name.as_ref()),
            format: string.format.clone(),
            string: Some(Box::new(StringValidation {
                max_length: string.max_length.map(|i| i as u32),
                min_length: string.min_length.map(|i| i as u32),
                pattern: string.pattern.clone(),
            })),
            ..Default::default()
        };

        Ok(Schema::Object(data))
    }

    fn render_struct(&self, structure: &StructType) -> RenderResult<Schema> {
        let mut properties = BTreeMap::new();
        let mut required = BTreeSet::from_iter(structure.required.clone());

        for field in &structure.fields {
            let name = field.name.clone().unwrap();

            if !field.optional {
                required.insert(name.clone());
            }

            properties.insert(name, self.create_schema_from_field(field)?);
        }

        let data = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Object))),
            metadata: self.create_metadata(structure.name.as_ref()),
            object: Some(Box::new(ObjectValidation {
                required,
                properties,
                ..Default::default()
            })),
            ..Default::default()
        };

        Ok(Schema::Object(data))
    }

    fn render_tuple(&self, tuple: &TupleType) -> RenderResult<Schema> {
        let mut items = vec![];

        for item in &tuple.items_types {
            items.push(self.render_schema(item)?);
        }

        let data = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Array))),
            metadata: self.create_metadata(tuple.name.as_ref()),
            array: Some(Box::new(ArrayValidation {
                items: Some(SingleOrVec::Vec(items)),
                max_items: Some(tuple.items_types.len() as u32),
                min_items: Some(tuple.items_types.len() as u32),
                ..Default::default()
            })),
            ..Default::default()
        };

        Ok(Schema::Object(data))
    }

    fn render_union(&self, uni: &UnionType) -> RenderResult<Schema> {
        // let mut items = vec![];

        // for item in &uni.variants_types {
        //     items.push(self.render_schema(item)?);
        // }

        // Ok(items.join(" | "))

        self.render_unknown()
    }

    fn render_unknown(&self) -> RenderResult<Schema> {
        Ok(Schema::Object(SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Null))),
            ..Default::default()
        }))
    }

    fn render(&mut self, schemas: &[SchemaType], references: &HashSet<String>) -> RenderResult {
        self.references.extend(references.to_owned());

        let mut outputs = vec![
            "// Automatically generated by schematic. DO NOT MODIFY!".to_string(),
            "/* eslint-disable */".to_string(),
        ];

        for schema in schemas {
            // if let Some(name) = schema.get_name() {
            //     outputs.push(match schema {
            //         SchemaType::Struct(inner) => self.export_object_type(name, inner)?,
            //         SchemaType::Union(inner) => self.export_enum_type(name, inner)?,
            //         _ => self.export_type_alias(name, self.render_schema(schema)?)?,
            //     });
            // }
        }

        Ok(outputs.join("\n\n"))
    }
}
