use crate::format::Format;
use crate::schema::{RenderResult, SchemaRenderer};
use indexmap::IndexMap;
use miette::miette;
use schematic_types::*;
use std::collections::{HashMap, HashSet, VecDeque};

/// Options to control the rendered template.
pub struct TemplateOptions {
    /// Include field comments in output.
    pub comments: bool,

    /// List of field names to render but comment out. Supports dot notation.
    pub comment_fields: Vec<String>,

    /// Default values for each field within the root struct.
    pub default_values: HashMap<String, SchemaType>,

    /// Content to append to the bottom of the output.
    pub footer: String,

    /// Content to prepend to the top of the output.
    pub header: String,

    /// List of field names to not render. Supports dot notation.
    pub hide_fields: Vec<String>,

    /// Character(s) to use for indentation.
    pub indent_char: String,

    /// Insert an extra newline between fields.
    pub newline_between_fields: bool,
}

impl Default for TemplateOptions {
    fn default() -> Self {
        Self {
            comments: true,
            comment_fields: vec![],
            default_values: HashMap::new(),
            footer: String::new(),
            header: String::new(),
            hide_fields: vec![],
            indent_char: "  ".into(),
            newline_between_fields: true,
        }
    }
}

fn lit_to_string(lit: &LiteralValue) -> String {
    match lit {
        LiteralValue::Bool(inner) => inner.to_string(),
        LiteralValue::F32(inner) => inner.to_string(),
        LiteralValue::F64(inner) => inner.to_string(),
        LiteralValue::Int(inner) => inner.to_string(),
        LiteralValue::UInt(inner) => inner.to_string(),
        LiteralValue::String(inner) => format!("\"{}\"", inner),
    }
}

fn is_nested_type(schema: &SchemaType) -> bool {
    match schema {
        SchemaType::Struct(_) => true,
        SchemaType::Union(uni) => {
            if uni.is_nullable() && uni.variants_types.len() == 2 {
                uni.variants_types
                    .iter()
                    .find(|v| !v.is_null())
                    .is_some_and(|v| is_nested_type(v))
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Renders template files from a schema.
pub struct TemplateRenderer {
    depth: usize,
    format: Format,
    options: TemplateOptions,
    stack: VecDeque<String>,
}

impl TemplateRenderer {
    pub fn new_format(format: Format) -> Self {
        Self::new(format, TemplateOptions::default())
    }

    pub fn new(format: Format, options: TemplateOptions) -> Self {
        Self {
            depth: 0,
            format,
            options,
            stack: VecDeque::new(),
        }
    }

    fn indent(&self) -> String {
        if self.depth == 0 {
            String::new()
        } else {
            self.options.indent_char.repeat(self.depth)
        }
    }

    fn gap(&self) -> &str {
        if self.options.newline_between_fields {
            "\n\n"
        } else {
            "\n"
        }
    }

    fn create_comment(&self, field: &SchemaField) -> String {
        if !self.options.comments
            || field.description.is_none()
            || field
                .description
                .as_ref()
                .is_some_and(|desc| desc.is_empty())
        {
            return String::new();
        }

        let mut lines = vec![];
        let indent = self.indent();
        let prefix = self.get_comment_prefix();

        let mut push = |line: String| {
            lines.push(format!("{indent}{prefix}{}", line));
        };

        if let Some(comment) = &field.description {
            comment
                .trim()
                .split('\n')
                .for_each(|c| push(c.trim().to_owned()));
        }

        if let Some(deprecated) = &field.deprecated {
            push(if deprecated.is_empty() {
                "@deprecated".into()
            } else {
                format!("@deprecated {}", deprecated)
            });
        }

        if let Some(env_var) = &field.env_var {
            push(format!("@envvar {}", env_var));
        }

        let mut out = lines.join("\n");
        out.push('\n');
        out
    }

    fn create_field(&self, field: &SchemaField, property: String) -> String {
        let key = self.get_stack_key();

        format!(
            "{}{}{}{}",
            self.create_comment(field),
            self.indent(),
            if self.options.comment_fields.contains(&key) {
                self.get_comment_prefix()
            } else {
                ""
            },
            property
        )
    }

    fn get_comment_prefix(&self) -> &str {
        if self.format.is_json() {
            "// "
        } else {
            "# "
        }
    }

    fn get_field_value(&mut self, schema: &SchemaType) -> miette::Result<String> {
        let key = self.get_stack_key();

        if let Some(default) = self.options.default_values.remove(&key) {
            return self.render_schema(&default);
        }

        self.render_schema(schema)
    }

    fn get_stack_key(&self) -> String {
        let mut key = String::new();
        let last_index = self.stack.len() - 1;

        for (index, item) in self.stack.iter().enumerate() {
            key.push_str(item);

            if index != last_index {
                key.push('.');
            }
        }

        key
    }

    fn is_hidden(&self, field: &SchemaField) -> bool {
        let key = self.get_stack_key();

        field.hidden || self.options.hide_fields.contains(&key)
    }
}

impl SchemaRenderer<String> for TemplateRenderer {
    fn is_reference(&self, _name: &str) -> bool {
        false
    }

    fn render_array(&mut self, _array: &ArrayType) -> RenderResult {
        Ok("[]".into())
    }

    fn render_boolean(&mut self, boolean: &BooleanType) -> RenderResult {
        if let Some(default) = &boolean.default {
            return Ok(lit_to_string(default));
        }

        Ok("false".into())
    }

    fn render_enum(&mut self, enu: &EnumType) -> RenderResult {
        if let Some(index) = &enu.default_index {
            if let Some(value) = enu.values.get(*index) {
                return Ok(lit_to_string(value));
            }
        }

        self.render_null()
    }

    fn render_float(&mut self, float: &FloatType) -> RenderResult {
        if let Some(default) = &float.default {
            return Ok(lit_to_string(default));
        }

        Ok("0.0".into())
    }

    fn render_integer(&mut self, integer: &IntegerType) -> RenderResult {
        if let Some(default) = &integer.default {
            return Ok(lit_to_string(default));
        }

        Ok("0".into())
    }

    fn render_literal(&mut self, literal: &LiteralType) -> RenderResult {
        if let Some(value) = &literal.value {
            return Ok(lit_to_string(value));
        }

        self.render_null()
    }

    fn render_null(&mut self) -> RenderResult {
        Ok("null".into())
    }

    fn render_object(&mut self, _object: &ObjectType) -> RenderResult {
        Ok("{}".into())
    }

    fn render_reference(&mut self, reference: &str) -> RenderResult {
        Ok(reference.into())
    }

    fn render_string(&mut self, string: &StringType) -> RenderResult {
        if let Some(default) = &string.default {
            return Ok(lit_to_string(default));
        }

        Ok("\"\"".into())
    }

    fn render_struct(&mut self, structure: &StructType) -> RenderResult {
        #[cfg(feature = "json")]
        {
            if self.format.is_json() {
                let mut out = vec![];

                self.depth += 1;

                for field in &structure.fields {
                    self.stack.push_back(field.name.clone());

                    if !self.is_hidden(field) {
                        let prop = format!(
                            "\"{}\": {},",
                            field.name,
                            self.get_field_value(&field.type_of)?,
                        );

                        out.push(self.create_field(field, prop));
                    }

                    self.stack.pop_back();
                }

                self.depth -= 1;

                return Ok(format!("{{\n{}\n{}}}", out.join(self.gap()), self.indent()));
            }
        }

        #[cfg(feature = "toml")]
        {
            if self.format.is_toml() {
                let mut out = vec![];
                let mut structs = vec![];

                for field in &structure.fields {
                    // Nested structs have weird syntax, so render them
                    // at the bottom after other fields
                    if is_nested_type(&field.type_of) {
                        structs.push(field);
                        continue;
                    }

                    self.stack.push_back(field.name.clone());

                    if !self.is_hidden(field) {
                        let prop =
                            format!("{} = {}", field.name, self.get_field_value(&field.type_of)?,);

                        out.push(self.create_field(field, prop));
                    }

                    self.stack.pop_back();
                }

                for field in structs {
                    self.stack.push_back(field.name.clone());

                    if !self.is_hidden(field) {
                        out.push(format!(
                            "{}{}[{}]\n{}",
                            if self.options.newline_between_fields && self.stack.len() == 1 {
                                ""
                            } else {
                                "\n"
                            },
                            self.create_comment(field),
                            self.stack.iter().cloned().collect::<Vec<_>>().join("."),
                            self.get_field_value(&field.type_of)?,
                        ));
                    }

                    self.stack.pop_back();
                }

                return Ok(out.join(self.gap()));
            }
        }

        #[cfg(feature = "yaml")]
        {
            if self.format.is_yaml() {
                let mut out = vec![];

                for field in &structure.fields {
                    self.stack.push_back(field.name.clone());

                    if self.is_hidden(field) {
                        self.stack.pop_back();
                        continue;
                    }

                    let is_nested = is_nested_type(&field.type_of);

                    if is_nested {
                        self.depth += 1;
                    }

                    let value = self.get_field_value(&field.type_of)?;

                    if is_nested {
                        self.depth -= 1;
                    }

                    let prop = format!(
                        "{}:{}{}",
                        field.name,
                        if is_nested { "\n" } else { " " },
                        value
                    );

                    out.push(self.create_field(field, prop));

                    self.stack.pop_back();
                }

                return Ok(out.join(self.gap()));
            }
        }

        Ok("".into())
    }

    fn render_tuple(&mut self, tuple: &TupleType) -> RenderResult {
        let mut items = vec![];

        for item in &tuple.items_types {
            items.push(self.render_schema(item)?);
        }

        Ok(format!("[{}]", items.join(", ")))
    }

    fn render_union(&mut self, uni: &UnionType) -> RenderResult {
        if let Some(index) = &uni.default_index {
            if let Some(variant) = uni.variants_types.get(*index) {
                return self.render_schema(variant);
            }
        }

        // We have a nullable type, so render the non-null value
        if uni.is_nullable() {
            if let Some(variant) = uni.variants_types.iter().find(|v| !v.is_null()) {
                return self.render_schema(variant);
            }
        }

        self.render_null()
    }

    fn render_unknown(&mut self) -> RenderResult {
        self.render_null()
    }

    fn render(
        &mut self,
        schemas: &IndexMap<String, SchemaType>,
        _references: &HashSet<String>,
    ) -> RenderResult {
        let Some(schema) = schemas.values().last() else {
            return Err(miette!(
                "At least 1 schema is required to generate a template."
            ));
        };

        let SchemaType::Struct(schema) = schema else {
            return Err(miette!("The last registered schema must be a struct type."));
        };

        let template = self.render_struct(schema)?;

        let mut output = format!("{}{}{}", self.options.header, template, self.options.footer);

        if self.format.is_toml() || self.format.is_yaml() {
            output.push('\n');
        }

        Ok(output)
    }
}
