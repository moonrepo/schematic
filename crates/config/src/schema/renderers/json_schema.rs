use crate::schema::{RenderResult, SchemaRenderer};
use indexmap::IndexMap;
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

fn clean_comment(comment: String) -> String {
    comment.trim().replace('\n', " ")
}

fn lit_to_value(lit: &LiteralValue) -> Value {
    match lit {
        LiteralValue::Bool(inner) => Value::Bool(*inner),
        LiteralValue::F32(inner) => Value::Number(Number::from_f64(*inner as f64).unwrap()),
        LiteralValue::F64(inner) => Value::Number(Number::from_f64(*inner).unwrap()),
        LiteralValue::Int(inner) => Value::Number(Number::from(*inner)),
        LiteralValue::UInt(inner) => Value::Number(Number::from(*inner)),
        LiteralValue::String(inner) => Value::String(inner.to_owned()),
    }
}

impl JsonSchemaRenderer {
    pub fn new(options: JsonSchemaOptions) -> Self {
        Self {
            options,
            references: HashSet::new(),
        }
    }

    fn create_schema_from_field(&mut self, field: &SchemaField) -> RenderResult<Schema> {
        let mut schema = self.render_schema(&field.type_of)?;

        if let Schema::Object(ref mut inner) = schema {
            let mut metadata = Metadata {
                // title: field.name.clone(),
                description: field.description.clone().map(clean_comment),
                deprecated: field.deprecated.is_some(),
                read_only: field.read_only,
                write_only: field.write_only,
                ..Default::default()
            };

            if let Some(default) = field.type_of.get_default() {
                metadata.default = Some(lit_to_value(default));
            }

            inner.metadata = Some(Box::new(metadata));
        }

        Ok(schema)
    }
}

impl SchemaRenderer<Schema> for JsonSchemaRenderer {
    fn is_reference(&self, name: &str) -> bool {
        self.references.contains(name)
    }

    fn render_array(&mut self, array: &ArrayType) -> RenderResult<Schema> {
        let use_contains = array.contains.is_some_and(|v| v);

        let data = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Array))),
            array: Some(Box::new(ArrayValidation {
                contains: if use_contains {
                    Some(Box::new(self.render_schema(&array.items_type)?))
                } else {
                    None
                },
                items: if use_contains {
                    None
                } else {
                    Some(SingleOrVec::Single(Box::new(
                        self.render_schema(&array.items_type)?,
                    )))
                },
                max_items: array.max_length.map(|i| i as u32),
                min_items: array.min_length.map(|i| i as u32),
                unique_items: array.unique,
                ..Default::default()
            })),
            ..Default::default()
        };

        Ok(Schema::Object(data))
    }

    fn render_boolean(&mut self, _boolean: &BooleanType) -> RenderResult<Schema> {
        Ok(Schema::Object(SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Boolean))),
            ..Default::default()
        }))
    }

    fn render_enum(&mut self, enu: &EnumType) -> RenderResult<Schema> {
        let mut instance_type = InstanceType::String;
        let mut enum_values = vec![];

        for value in &enu.values {
            match value {
                LiteralValue::Bool(v) => {
                    instance_type = InstanceType::Boolean;
                    enum_values.push(Value::Bool(*v));
                }
                LiteralValue::F32(v) => {
                    instance_type = InstanceType::Number;
                    enum_values.push(Value::Number(Number::from_f64(*v as f64).unwrap()));
                }
                LiteralValue::F64(v) => {
                    instance_type = InstanceType::Number;
                    enum_values.push(Value::Number(Number::from_f64(*v).unwrap()));
                }
                LiteralValue::Int(v) => {
                    instance_type = InstanceType::Number;
                    enum_values.push(Value::Number(Number::from(*v)));
                }
                LiteralValue::UInt(v) => {
                    instance_type = InstanceType::Number;
                    enum_values.push(Value::Number(Number::from(*v)));
                }
                LiteralValue::String(v) => {
                    instance_type = InstanceType::String;
                    enum_values.push(Value::String(v.to_owned()));
                }
            };
        }

        let metadata = Metadata {
            title: enu.name.clone(),
            description: enu.description.clone().map(clean_comment),
            ..Default::default()
        };

        Ok(Schema::Object(SchemaObject {
            metadata: Some(Box::new(metadata)),
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

    fn render_literal(&mut self, literal: &LiteralType) -> RenderResult<Schema> {
        if let Some(value) = &literal.value {
            return Ok(Schema::Object(SchemaObject {
                const_value: Some(lit_to_value(value)),
                ..Default::default()
            }));
        }

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
                max_properties: object.max_length.map(|i| i as u32),
                min_properties: object.min_length.map(|i| i as u32),
                required: BTreeSet::from_iter(object.required.clone().unwrap_or_default()),
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
        let mut required = BTreeSet::from_iter(structure.required.clone().unwrap_or_default());

        for field in &structure.fields {
            if field.hidden {
                continue;
            }

            if !field.optional {
                required.insert(field.name.clone());
            }

            properties.insert(field.name.clone(), self.create_schema_from_field(field)?);
        }

        let data = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Object))),
            metadata: Some(Box::new(Metadata {
                title: structure.name.clone(),
                description: structure.description.clone().map(clean_comment),
                ..Default::default()
            })),
            object: Some(Box::new(ObjectValidation {
                additional_properties: Some(Box::new(Schema::Bool(false))),
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

        let mut metadata = Metadata {
            title: uni.name.clone(),
            description: uni.description.clone().map(clean_comment),
            ..Default::default()
        };

        for item in &uni.variants_types {
            items.push(self.render_schema(item)?);

            if metadata.default.is_none() {
                if let Some(def) = item.get_default() {
                    metadata.default = Some(lit_to_value(def));
                }
            }
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
            metadata: Some(Box::new(metadata)),
            subschemas: Some(Box::new(subschema)),
            ..Default::default()
        }))
    }

    fn render_unknown(&mut self) -> RenderResult<Schema> {
        Ok(Schema::Object(SchemaObject {
            instance_type: Some(SingleOrVec::Vec(vec![
                InstanceType::Boolean,
                InstanceType::Object,
                InstanceType::Array,
                InstanceType::Number,
                InstanceType::String,
                InstanceType::Integer,
            ])),
            ..Default::default()
        }))
    }

    fn render(
        &mut self,
        schemas: &IndexMap<String, SchemaType>,
        references: &HashSet<String>,
    ) -> RenderResult {
        self.references.extend(references.to_owned());

        let mut root_schema = RootSchema {
            meta_schema: self.options.meta_schema.clone(),
            ..RootSchema::default()
        };

        for (i, (name, schema)) in schemas.iter().enumerate() {
            // The last schema in the generator is the root schema
            if i == schemas.len() - 1 {
                root_schema.schema = self.render_schema_without_reference(schema)?.into_object();

            // Otherwise the others are all ref definitions
            } else {
                root_schema.definitions.insert(
                    name.to_owned(),
                    self.render_schema_without_reference(schema)?,
                );
            }
        }

        for visitor in &mut self.options.visitors {
            visitor.visit_root_schema(&mut root_schema)
        }

        serde_json::to_string_pretty(&root_schema).into_diagnostic()
    }
}
