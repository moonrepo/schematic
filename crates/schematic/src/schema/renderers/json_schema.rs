use crate::schema::{RenderResult, SchemaRenderer};
use indexmap::IndexMap;
use miette::IntoDiagnostic;
use schemars::generate::{GenTransform, SchemaSettings};
use schematic_types::*;
use serde_json::{Map, Number, Value};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::mem;

/// A schema document, which schemars models as JSON rather than as typed nodes.
type JsonSchema = schemars::Schema;

/// Keys that describe a schema rather than constrain it. A field replaces these
/// wholesale with its own, so they are dropped from what the type contributed.
const METADATA_KEYS: [&str; 6] = [
    "title",
    "description",
    "default",
    "deprecated",
    "readOnly",
    "writeOnly",
];

pub struct JsonSchemaOptions {
    /// Allows newlines in descriptions, otherwise strips them.
    pub allow_newlines_in_description: bool,

    /// Exclude field aliases from being rendered.
    pub exclude_aliases: bool,

    /// Includes a `markdownDescription` field in the JSON file. This is non-standard.
    pub markdown_description: bool,

    /// Marks all non-option struct fields as required.
    pub mark_struct_fields_required: bool,

    /// Sets the field's name as the `title` of each schema entry.
    /// This overrides any `title` manually defined by a type.
    pub set_field_name_as_title: bool,

    /// Prefix that `$ref` values are built from.
    pub definitions_path: String,

    // Inherited from schemars.
    pub meta_schema: Option<String>,
    pub transforms: Vec<Box<dyn GenTransform>>,
    pub inline_subschemas: bool,
}

impl Default for JsonSchemaOptions {
    fn default() -> Self {
        let settings = SchemaSettings::draft07();

        Self {
            allow_newlines_in_description: false,
            exclude_aliases: false,
            markdown_description: false,
            mark_struct_fields_required: true,
            set_field_name_as_title: false,
            // Schemars models this as a JSON pointer, while we concatenate it
            // onto a name to form a `$ref`, so it is not inherited
            definitions_path: "#/definitions/".into(),
            meta_schema: settings.meta_schema.map(|schema| schema.to_string()),
            transforms: settings.transforms,
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

fn strip_markdown(description: &str) -> String {
    use markdown::{ParseOptions, to_mdast};

    to_mdast(description, &ParseOptions::gfm())
        .unwrap()
        .to_string()
}

fn inject_markdown_descriptions(json: &mut Value) -> RenderResult<()> {
    match json {
        Value::Array(array) => {
            for item in array.iter_mut() {
                inject_markdown_descriptions(item)?;
            }
        }
        Value::Object(object) => {
            let mut markdown = None;

            for (key, value) in object.iter_mut() {
                if key != "description" {
                    inject_markdown_descriptions(value)?;
                    continue;
                }

                // Only add field if we actually detect markdown
                if let Value::String(inner) = value {
                    if inner.contains('`')
                        || inner.contains('*')
                        || inner.contains('_')
                        || inner.contains('-')
                        || (inner.contains('[') && inner.contains('('))
                    {
                        markdown = Some(mem::take(inner));
                    }
                }
            }

            if let Some(markdown) = markdown {
                object.insert(
                    "description".into(),
                    Value::String(strip_markdown(&markdown)),
                );

                object.insert("markdownDescription".into(), Value::String(markdown));
            }
        }
        _ => {
            // Do nothing
        }
    };

    Ok(())
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

/// Insert a key only when there is a value for it, so that an absent setting
/// leaves no trace in the document.
fn insert_some(map: &mut Map<String, Value>, key: &str, value: Option<Value>) {
    if let Some(value) = value {
        map.insert(key.to_owned(), value);
    }
}

fn number(value: f64) -> Option<Value> {
    Number::from_f64(value).map(Value::Number)
}

fn count(value: usize) -> Value {
    Value::Number(Number::from(value))
}

impl JsonSchemaRenderer {
    pub fn new(options: JsonSchemaOptions) -> Self {
        Self {
            options,
            references: HashSet::default(),
        }
    }

    /// Start a schema document with the metadata a type carries. Keys are
    /// inserted in the order schemars used to serialize them, so that upgrading
    /// does not reshuffle every generated file.
    fn create_metadata_from_schema(&self, schema: &Schema) -> Map<String, Value> {
        let mut map = Map::new();

        if !self.options.set_field_name_as_title
            && let Some(name) = &schema.name
        {
            map.insert("title".into(), Value::String(name.to_owned()));
        }

        if let Some(description) = &schema.description {
            map.insert(
                "description".into(),
                Value::String(clean_comment(
                    description.to_owned(),
                    self.options.allow_newlines_in_description,
                )),
            );
        }

        if schema.deprecated.is_some() {
            map.insert("deprecated".into(), Value::Bool(true));
        }

        map
    }

    fn create_field_from_schema(
        &mut self,
        name: &str,
        field: &SchemaField,
    ) -> RenderResult<JsonSchema> {
        let rendered = self.render_schema(&field.schema)?;

        let Some(object) = rendered.as_object() else {
            return Ok(rendered);
        };

        // The field describes itself, so its metadata replaces whatever the
        // type contributed rather than merging with it
        let mut map = Map::new();

        if self.options.set_field_name_as_title && !name.is_empty() {
            map.insert("title".into(), Value::String(name.to_owned()));
        }

        if let Some(comment) = &field.comment {
            map.insert(
                "description".into(),
                Value::String(clean_comment(
                    comment.to_owned(),
                    self.options.allow_newlines_in_description,
                )),
            );
        }

        if let Some(default) = field.schema.get_default() {
            map.insert("default".into(), lit_to_value(default));
        }

        if field.deprecated.is_some() {
            map.insert("deprecated".into(), Value::Bool(true));
        }

        if field.read_only {
            map.insert("readOnly".into(), Value::Bool(true));
        }

        if field.write_only {
            map.insert("writeOnly".into(), Value::Bool(true));
        }

        for (key, value) in object {
            if !METADATA_KEYS.contains(&key.as_str()) {
                map.insert(key.to_owned(), value.to_owned());
            }
        }

        Ok(JsonSchema::from(map))
    }
}

impl SchemaRenderer<JsonSchema> for JsonSchemaRenderer {
    fn is_reference(&self, name: &str) -> bool {
        self.references.contains(name)
    }

    fn render_array(&mut self, array: &ArrayType, schema: &Schema) -> RenderResult<JsonSchema> {
        let mut map = self.create_metadata_from_schema(schema);

        map.insert("type".into(), Value::String("array".into()));
        map.insert(
            "items".into(),
            self.render_schema(&array.items_type)?.to_value(),
        );

        insert_some(&mut map, "maxItems", array.max_length.map(count));
        insert_some(&mut map, "minItems", array.min_length.map(count));
        insert_some(&mut map, "uniqueItems", array.unique.map(Value::Bool));

        // `contains` constrains the array as a whole, and `items` every entry,
        // so the two are independent and both are rendered when present.
        if let Some(inner) = &array.contains {
            map.insert("contains".into(), self.render_schema(inner)?.to_value());
        }

        Ok(JsonSchema::from(map))
    }

    fn render_boolean(
        &mut self,
        _boolean: &BooleanType,
        schema: &Schema,
    ) -> RenderResult<JsonSchema> {
        let mut map = self.create_metadata_from_schema(schema);

        map.insert("type".into(), Value::String("boolean".into()));

        Ok(JsonSchema::from(map))
    }

    fn render_enum(&mut self, enu: &EnumType, schema: &Schema) -> RenderResult<JsonSchema> {
        let mut map = self.create_metadata_from_schema(schema);

        // Unit enum with a fallback variant
        if enu
            .variants
            .as_ref()
            .is_some_and(|v| v.len() != enu.values.len())
        {
            let mut any_of = vec![];

            for (name, field) in enu.variants.as_ref().unwrap() {
                if !field.hidden {
                    any_of.push(self.create_field_from_schema(name, field)?.to_value());
                }
            }

            map.insert("anyOf".into(), Value::Array(any_of));

            return Ok(JsonSchema::from(map));
        }

        // Unit enum with no fallback variant
        let mut instance_type = "string";
        let mut enum_values = vec![];

        for value in &enu.values {
            match value {
                LiteralValue::Bool(v) => {
                    instance_type = "boolean";
                    enum_values.push(Value::Bool(*v));
                }
                LiteralValue::F32(v) => {
                    instance_type = "number";
                    enum_values.push(Value::Number(Number::from_f64(*v as f64).unwrap()));
                }
                LiteralValue::F64(v) => {
                    instance_type = "number";
                    enum_values.push(Value::Number(Number::from_f64(*v).unwrap()));
                }
                LiteralValue::Int(v) => {
                    instance_type = "number";
                    enum_values.push(Value::Number(Number::from(*v)));
                }
                LiteralValue::UInt(v) => {
                    instance_type = "number";
                    enum_values.push(Value::Number(Number::from(*v)));
                }
                LiteralValue::String(v) => {
                    instance_type = "string";
                    enum_values.push(Value::String(v.to_owned()));
                }
            };
        }

        map.insert("type".into(), Value::String(instance_type.into()));
        map.insert("enum".into(), Value::Array(enum_values));

        Ok(JsonSchema::from(map))
    }

    fn render_float(&mut self, float: &FloatType, schema: &Schema) -> RenderResult<JsonSchema> {
        let mut map = self.create_metadata_from_schema(schema);

        map.insert("type".into(), Value::String("number".into()));

        insert_some(&mut map, "format", float.format.clone().map(Value::String));

        if let Some(values) = &float.enum_values {
            map.insert(
                "enum".into(),
                Value::Array(values.iter().filter_map(|v| number(*v)).collect()),
            );
        }

        insert_some(&mut map, "multipleOf", float.multiple_of.and_then(number));
        insert_some(&mut map, "maximum", float.max.and_then(number));
        insert_some(
            &mut map,
            "exclusiveMaximum",
            float.max_exclusive.and_then(number),
        );
        insert_some(&mut map, "minimum", float.min.and_then(number));
        insert_some(
            &mut map,
            "exclusiveMinimum",
            float.min_exclusive.and_then(number),
        );

        Ok(JsonSchema::from(map))
    }

    fn render_integer(
        &mut self,
        integer: &IntegerType,
        schema: &Schema,
    ) -> RenderResult<JsonSchema> {
        let mut map = self.create_metadata_from_schema(schema);

        map.insert("type".into(), Value::String("number".into()));

        insert_some(
            &mut map,
            "format",
            integer.format.clone().map(Value::String),
        );

        if let Some(values) = &integer.enum_values {
            map.insert(
                "enum".into(),
                Value::Array(
                    values
                        .iter()
                        .map(|v| Value::Number(Number::from(*v)))
                        .collect(),
                ),
            );
        }

        insert_some(
            &mut map,
            "multipleOf",
            integer.multiple_of.and_then(|i| number(i as f64)),
        );
        insert_some(
            &mut map,
            "maximum",
            integer.max.and_then(|i| number(i as f64)),
        );
        insert_some(
            &mut map,
            "exclusiveMaximum",
            integer.max_exclusive.and_then(|i| number(i as f64)),
        );
        insert_some(
            &mut map,
            "minimum",
            integer.min.and_then(|i| number(i as f64)),
        );
        insert_some(
            &mut map,
            "exclusiveMinimum",
            integer.min_exclusive.and_then(|i| number(i as f64)),
        );

        Ok(JsonSchema::from(map))
    }

    fn render_literal(
        &mut self,
        literal: &LiteralType,
        schema: &Schema,
    ) -> RenderResult<JsonSchema> {
        let mut map = self.create_metadata_from_schema(schema);

        map.insert("const".into(), lit_to_value(&literal.value));

        Ok(JsonSchema::from(map))
    }

    fn render_null(&mut self, schema: &Schema) -> RenderResult<JsonSchema> {
        let mut map = self.create_metadata_from_schema(schema);

        map.insert("type".into(), Value::String("null".into()));

        Ok(JsonSchema::from(map))
    }

    fn render_object(&mut self, object: &ObjectType, schema: &Schema) -> RenderResult<JsonSchema> {
        let mut map = self.create_metadata_from_schema(schema);

        map.insert("type".into(), Value::String("object".into()));

        insert_some(&mut map, "maxProperties", object.max_length.map(count));
        insert_some(&mut map, "minProperties", object.min_length.map(count));

        let required = BTreeSet::from_iter(object.required.clone().unwrap_or_default());

        if !required.is_empty() {
            map.insert(
                "required".into(),
                Value::Array(required.into_iter().map(Value::String).collect()),
            );
        }

        map.insert(
            "additionalProperties".into(),
            self.render_schema(&object.value_type)?.to_value(),
        );
        map.insert(
            "propertyNames".into(),
            self.render_schema(&object.key_type)?.to_value(),
        );

        Ok(JsonSchema::from(map))
    }

    fn render_reference(&mut self, reference: &str, _schema: &Schema) -> RenderResult<JsonSchema> {
        // Note: Don't add metadata as it causes nested schema references!
        Ok(JsonSchema::new_ref(format!(
            "{}{}",
            self.options.definitions_path, reference
        )))
    }

    fn render_string(&mut self, string: &StringType, schema: &Schema) -> RenderResult<JsonSchema> {
        let mut map = self.create_metadata_from_schema(schema);

        map.insert("type".into(), Value::String("string".into()));

        insert_some(&mut map, "format", string.format.clone().map(Value::String));

        if let Some(values) = &string.enum_values {
            map.insert(
                "enum".into(),
                Value::Array(values.iter().cloned().map(Value::String).collect()),
            );
        }

        insert_some(&mut map, "maxLength", string.max_length.map(count));
        insert_some(&mut map, "minLength", string.min_length.map(count));
        insert_some(
            &mut map,
            "pattern",
            string.pattern.clone().map(Value::String),
        );

        Ok(JsonSchema::from(map))
    }

    fn render_struct(
        &mut self,
        structure: &StructType,
        schema: &Schema,
    ) -> RenderResult<JsonSchema> {
        let mut properties = BTreeMap::new();
        let mut required = BTreeSet::from_iter(structure.required.clone().unwrap_or_default());
        let mut additional_properties = Some(Value::Bool(false));
        let exclude_aliases = self.options.exclude_aliases;

        for (name, field) in structure.sorted_fields() {
            if field.hidden {
                continue;
            }

            if field.flatten {
                if let Some(schema) = field.schema.get_nonnull_schema() {
                    let flattened = self.render_schema_without_reference(schema)?;

                    if matches!(schema.ty, SchemaType::Object(_))
                        && let Some(inner) = flattened.as_object()
                        && let Some(value) = inner.get("additionalProperties")
                    {
                        additional_properties = Some(value.to_owned());
                    } else {
                        additional_properties = Some(flattened.to_value());
                    }
                }

                continue;
            }

            if !field.optional && self.options.mark_struct_fields_required {
                required.insert(name.to_owned());
            }

            if !exclude_aliases {
                for alias in &field.aliases {
                    properties.insert(
                        alias.to_owned(),
                        self.create_field_from_schema(alias, field)?.to_value(),
                    );
                }
            }

            properties.insert(
                name.to_owned(),
                self.create_field_from_schema(name, field)?.to_value(),
            );
        }

        let mut map = self.create_metadata_from_schema(schema);

        map.insert("type".into(), Value::String("object".into()));

        if !required.is_empty() {
            map.insert(
                "required".into(),
                Value::Array(required.into_iter().map(Value::String).collect()),
            );
        }

        map.insert(
            "properties".into(),
            Value::Object(properties.into_iter().collect()),
        );

        insert_some(&mut map, "additionalProperties", additional_properties);

        Ok(JsonSchema::from(map))
    }

    fn render_tuple(&mut self, tuple: &TupleType, schema: &Schema) -> RenderResult<JsonSchema> {
        let mut items = vec![];

        for item in &tuple.items_types {
            items.push(self.render_schema(item)?.to_value());
        }

        let mut map = self.create_metadata_from_schema(schema);

        map.insert("type".into(), Value::String("array".into()));
        map.insert("items".into(), Value::Array(items));
        map.insert("maxItems".into(), count(tuple.items_types.len()));
        map.insert("minItems".into(), count(tuple.items_types.len()));

        Ok(JsonSchema::from(map))
    }

    fn render_union(&mut self, uni: &UnionType, schema: &Schema) -> RenderResult<JsonSchema> {
        let mut items = vec![];
        let mut default = None;

        for item in &uni.variants_types {
            items.push(self.render_schema(item)?.to_value());

            if default.is_none()
                && let Some(def) = item.get_default()
            {
                default = Some(lit_to_value(def));
            }
        }

        let mut map = self.create_metadata_from_schema(schema);

        // `default` sits with the other metadata keys, ahead of the subschemas
        if let Some(default) = default {
            let rest = mem::take(&mut map);

            for (key, value) in rest {
                map.insert(key.clone(), value);

                if key == "description" || (key == "title" && !map.contains_key("description")) {
                    map.insert("default".into(), default.clone());
                }
            }

            if !map.contains_key("default") {
                map.insert("default".into(), default);
            }
        }

        map.insert(
            match uni.operator {
                UnionOperator::AnyOf => "anyOf".into(),
                UnionOperator::OneOf => "oneOf".into(),
            },
            Value::Array(items),
        );

        Ok(JsonSchema::from(map))
    }

    fn render_unknown(&mut self, schema: &Schema) -> RenderResult<JsonSchema> {
        let mut map = self.create_metadata_from_schema(schema);

        map.insert(
            "type".into(),
            Value::Array(
                ["boolean", "object", "array", "number", "string", "integer"]
                    .into_iter()
                    .map(|ty| Value::String(ty.into()))
                    .collect(),
            ),
        );

        Ok(JsonSchema::from(map))
    }

    fn render(&mut self, schemas: IndexMap<String, Schema>) -> RenderResult {
        self.references = HashSet::from_iter(schemas.keys().cloned());

        let mut root = Map::new();

        if let Some(meta_schema) = &self.options.meta_schema {
            root.insert("$schema".into(), Value::String(meta_schema.to_owned()));
        }

        let mut definitions = BTreeMap::new();

        for (i, (name, schema)) in schemas.iter().enumerate() {
            let rendered = self.render_schema_without_reference(schema)?;

            // The last schema in the generator is the root schema
            if i == schemas.len() - 1 {
                if let Some(object) = rendered.as_object() {
                    for (key, value) in object {
                        root.insert(key.to_owned(), value.to_owned());
                    }
                }

            // Otherwise the others are all ref definitions
            } else {
                definitions.insert(name.to_owned(), rendered.to_value());
            }
        }

        if !definitions.is_empty() {
            root.insert(
                "definitions".into(),
                Value::Object(definitions.into_iter().collect()),
            );
        }

        let mut root_schema = JsonSchema::from(root);

        for transform in &mut self.options.transforms {
            transform.transform(&mut root_schema);
        }

        let mut json = root_schema.to_value();

        if self.options.markdown_description {
            inject_markdown_descriptions(&mut json)?;
        }

        serde_json::to_string_pretty(&json).into_diagnostic()
    }
}
