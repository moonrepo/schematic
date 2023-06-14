use crate::schema::{RenderResult, SchemaRenderer};
use miette::IntoDiagnostic;
use schemars::gen::SchemaSettings;
use schemars::schema::*;
use schematic_types::*;
use serde_json::{Number, Value};
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub type JsonSchemaOptions = SchemaSettings;

/// Renders JSON schema documents from a schema.
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

    fn create_schema_from_field(
        &mut self,
        field: &SchemaField,
        partial: bool,
    ) -> RenderResult<Schema> {
        let mut schema = if partial && !matches!(&field.type_of, SchemaType::Union(_)) {
            self.render_union(&UnionType {
                name: field.name.clone(),
                operator: UnionOperator::OneOf,
                variants_types: vec![Box::new(field.type_of.clone()), Box::new(SchemaType::Null)],
                ..Default::default()
            })?
        } else {
            self.render_schema(&field.type_of)?
        };

        if let Schema::Object(ref mut inner) = schema {
            inner.metadata = Some(Box::new(Metadata {
                // title: field.name.clone(),
                description: field
                    .description
                    .clone()
                    .map(|d| d.trim().replace("* ", "").replace(" * ", "")),
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

    fn render_array(&mut self, array: &ArrayType) -> RenderResult<Schema> {
        let data = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Array))),
            array: Some(Box::new(ArrayValidation {
                items: Some(SingleOrVec::Single(Box::new(
                    self.render_schema(&array.items_type)?,
                ))),
                max_items: array.max_length.map(|i| i as u32),
                min_items: array.min_length.map(|i| i as u32),
                unique_items: array.unique,
                ..Default::default()
            })),
            ..Default::default()
        };

        Ok(Schema::Object(data))
    }

    fn render_boolean(&mut self) -> RenderResult<Schema> {
        Ok(Schema::Object(SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Boolean))),
            ..Default::default()
        }))
    }

    fn render_enum(&mut self, enu: &EnumType) -> RenderResult<Schema> {
        let mut instance_type = InstanceType::String;
        let mut enum_values = vec![];

        for value in &enu.values {
            match value.value.clone().unwrap() {
                LiteralValue::Bool(v) => {
                    instance_type = InstanceType::Boolean;
                    enum_values.push(Value::Bool(v));
                }
                LiteralValue::Float(v) => {
                    instance_type = InstanceType::Number;
                    enum_values.push(Value::Number(Number::from_f64(v.parse().unwrap()).unwrap()));
                }
                LiteralValue::Int(v) => {
                    instance_type = InstanceType::Number;
                    enum_values.push(Value::Number(Number::from(v)));
                }
                LiteralValue::UInt(v) => {
                    instance_type = InstanceType::Number;
                    enum_values.push(Value::Number(Number::from(v)));
                }
                LiteralValue::String(v) => {
                    instance_type = InstanceType::String;
                    enum_values.push(Value::String(v));
                }
            };
        }

        Ok(Schema::Object(SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(instance_type))),
            enum_values: Some(enum_values),
            ..Default::default()
        }))
    }

    fn render_float(&mut self, float: &FloatType) -> RenderResult<Schema> {
        let data = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Number))),
            enum_values: float.enum_values.clone().map(|values| {
                values
                    .into_iter()
                    .map(|v| Value::Number(Number::from_f64(v).unwrap()))
                    .collect()
            }),
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

    fn render_integer(&mut self, integer: &IntegerType) -> RenderResult<Schema> {
        let data = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Number))),
            enum_values: integer.enum_values.clone().map(|values| {
                values
                    .into_iter()
                    .map(|v| Value::Number(Number::from(v)))
                    .collect()
            }),
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

    // Note: This isn't used...
    fn render_literal(&mut self, _: &LiteralType) -> RenderResult<Schema> {
        self.render_unknown()
    }

    fn render_null(&mut self) -> RenderResult<Schema> {
        Ok(Schema::Object(SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Null))),
            ..Default::default()
        }))
    }

    fn render_object(&mut self, object: &ObjectType) -> RenderResult<Schema> {
        let data = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Object))),
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

    fn render_reference(&mut self, reference: &str) -> RenderResult<Schema> {
        Ok(Schema::Object(SchemaObject {
            reference: Some(format!("{}{}", self.options.definitions_path, reference)),
            ..Default::default()
        }))
    }

    fn render_string(&mut self, string: &StringType) -> RenderResult<Schema> {
        let data = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
            enum_values: string
                .enum_values
                .clone()
                .map(|values| values.into_iter().map(Value::String).collect()),
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

    fn render_struct(&mut self, structure: &StructType) -> RenderResult<Schema> {
        let mut properties = BTreeMap::new();
        let mut required = BTreeSet::from_iter(structure.required.clone());

        for field in &structure.fields {
            if field.hidden {
                continue;
            }

            let name = field.name.clone().unwrap();

            if !field.optional {
                required.insert(name.clone());
            }

            properties.insert(
                name,
                self.create_schema_from_field(field, structure.partial)?,
            );
        }

        let data = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Object))),
            metadata: structure.name.as_ref().map(|n| {
                Box::new(Metadata {
                    title: Some(n.to_owned()),
                    ..Default::default()
                })
            }),
            object: Some(Box::new(ObjectValidation {
                required,
                properties,
                ..Default::default()
            })),
            ..Default::default()
        };

        Ok(Schema::Object(data))
    }

    fn render_tuple(&mut self, tuple: &TupleType) -> RenderResult<Schema> {
        let mut items = vec![];

        for item in &tuple.items_types {
            items.push(self.render_schema(item)?);
        }

        let data = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Array))),
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

    fn render_union(&mut self, uni: &UnionType) -> RenderResult<Schema> {
        let mut items = vec![];

        for item in &uni.variants_types {
            items.push(self.render_schema(item)?);
        }

        let subschema = match uni.operator {
            UnionOperator::AnyOf => SubschemaValidation {
                any_of: Some(items),
                ..Default::default()
            },
            UnionOperator::OneOf => SubschemaValidation {
                one_of: Some(items),
                ..Default::default()
            },
        };

        Ok(Schema::Object(SchemaObject {
            subschemas: Some(Box::new(subschema)),
            ..Default::default()
        }))
    }

    fn render_unknown(&mut self) -> RenderResult<Schema> {
        Ok(Schema::Object(SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Null))),
            ..Default::default()
        }))
    }

    fn render(&mut self, schemas: &[SchemaType], references: &HashSet<String>) -> RenderResult {
        self.references.extend(references.to_owned());

        let mut root_schema = RootSchema {
            meta_schema: self.options.meta_schema.clone(),
            ..RootSchema::default()
        };

        for (i, schema) in schemas.iter().enumerate() {
            let name = schema.get_name().unwrap().to_owned();

            // The last schema in the generator is the root schema
            if i == schemas.len() - 1 {
                root_schema.schema = self.render_schema_without_reference(schema)?.into_object();

            // Otherwise the others are all ref definitions
            } else {
                root_schema
                    .definitions
                    .insert(name, self.render_schema_without_reference(schema)?);
            }
        }

        for visitor in &mut self.options.visitors {
            visitor.visit_root_schema(&mut root_schema)
        }

        serde_json::to_string_pretty(&root_schema).into_diagnostic()
    }
}