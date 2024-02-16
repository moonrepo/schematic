use crate::schema::{RenderResult, SchemaRenderer};
use indexmap::IndexMap;
use miette::IntoDiagnostic;
use schemars::gen::{GenVisitor, SchemaSettings};
use schemars::schema::*;
use schematic_types::*;
use serde_json::{Number, Value};
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub struct JsonSchemaOptions {
    /// Allows newlines in descriptions, otherwise strips them.
    pub allow_newlines_in_description: bool,
    /// Includes a `markdownDescription` field in the JSON file. This is non-standard.
    pub markdown_description: bool,
    /// Marks all non-option struct fields as required.
    pub mark_struct_fields_required: bool,
    /// Sets the field's name as the `title` of each schema entry.
    /// This overrides any `title` manually defined by a type.
    pub set_field_name_as_title: bool,

    // Inherited from schemars.
    pub option_nullable: bool,
    pub option_add_null_type: bool,
    pub definitions_path: String,
    pub meta_schema: Option<String>,
    pub visitors: Vec<Box<dyn GenVisitor>>,
    pub inline_subschemas: bool,
}

impl Default for JsonSchemaOptions {
    fn default() -> Self {
        let settings = SchemaSettings::draft07();

        Self {
            allow_newlines_in_description: false,
            markdown_description: false,
            mark_struct_fields_required: true,
            set_field_name_as_title: false,
            option_nullable: settings.option_nullable,
            option_add_null_type: settings.option_add_null_type,
            definitions_path: settings.definitions_path,
            meta_schema: settings.meta_schema,
            visitors: settings.visitors,
            inline_subschemas: settings.inline_subschemas,
        }
    }
}

/// Renders JSON schema documents from a schema.
#[derive(Default)]
pub struct JsonSchemaRenderer {
    options: JsonSchemaOptions,
    references: HashSet<String>,
}

fn clean_comment(comment: String, allow_newlines: bool) -> String {
    let comment = comment.trim();

    if allow_newlines {
        comment.to_owned()
    } else {
        comment.replace('\n', " ")
    }
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
                title: if self.options.set_field_name_as_title {
                    Some(field.name.clone())
                } else {
                    None
                },
                description: field
                    .description
                    .clone()
                    .map(|desc| clean_comment(desc, self.options.allow_newlines_in_description)),
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

    fn inject_markdown_descriptions(&self, json: &mut Value) -> RenderResult<()> {
        match json {
            Value::Array(array) => {
                for item in array.iter_mut() {
                    self.inject_markdown_descriptions(item)?;
                }
            }
            Value::Object(object) => {
                let mut markdown = None;

                for (key, value) in object.iter_mut() {
                    if key != "description" {
                        self.inject_markdown_descriptions(value)?;
                        continue;
                    }

                    // Only add field if we actually detect markdown
                    if let Value::String(inner) = value {
                        if inner.contains('`')
                            || inner.contains('*')
                            || inner.contains('_')
                            || (inner.contains('[') && inner.contains('('))
                        {
                            markdown = Some(inner.to_owned());
                        }
                    }
                }

                if let Some(markdown) = markdown {
                    object.insert("markdownDescription".into(), Value::String(markdown));
                }
            }
            _ => {
                // Do nothing
            }
        };

        Ok(())
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
            title: if self.options.set_field_name_as_title {
                None
            } else {
                enu.name.clone()
            },
            description: enu
                .description
                .clone()
                .map(|desc| clean_comment(desc, self.options.allow_newlines_in_description)),
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

            if !field.optional && self.options.mark_struct_fields_required {
                required.insert(field.name.clone());
            }

            properties.insert(field.name.clone(), self.create_schema_from_field(field)?);
        }

        let data = SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Object))),
            metadata: Some(Box::new(Metadata {
                title: if self.options.set_field_name_as_title {
                    None
                } else {
                    structure.name.clone()
                },
                description: structure
                    .description
                    .clone()
                    .map(|desc| clean_comment(desc, self.options.allow_newlines_in_description)),
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
            title: if self.options.set_field_name_as_title {
                None
            } else {
                uni.name.clone()
            },
            description: uni
                .description
                .clone()
                .map(|desc| clean_comment(desc, self.options.allow_newlines_in_description)),
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

        let mut json = serde_json::to_value(&root_schema).into_diagnostic()?;

        if self.options.markdown_description {
            self.inject_markdown_descriptions(&mut json)?;
        }

        serde_json::to_string_pretty(&json).into_diagnostic()
    }
}
