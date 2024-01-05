use crate::format::Format;
use crate::schema::{RenderResult, SchemaRenderer};
use indexmap::IndexMap;
use miette::miette;
use schematic_types::*;
use std::collections::{HashMap, VecDeque};

/// Options to control the rendered template.
pub struct TemplateOptions {
    /// Include field comments in output.
    pub comments: bool,

    /// List of field names to render but comment out.
    pub comment_fields: Vec<String>,

    /// Default values for each field within the root struct.
    pub default_values: HashMap<String, SchemaType>,

    /// List of array and object field names to expand and render a fake item.
    pub expand_fields: Vec<String>,

    /// Content to append to the bottom of the output.
    pub footer: String,

    /// Content to prepend to the top of the output.
    pub header: String,

    /// List of field names to not render.
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
            expand_fields: vec![],
            footer: String::new(),
            header: String::new(),
            hide_fields: vec![],
            indent_char: "  ".into(),
            newline_between_fields: true,
        }
    }
}

pub fn lit_to_string(lit: &LiteralValue) -> String {
    match lit {
        LiteralValue::Bool(inner) => inner.to_string(),
        LiteralValue::F32(inner) => inner.to_string(),
        LiteralValue::F64(inner) => inner.to_string(),
        LiteralValue::Int(inner) => inner.to_string(),
        LiteralValue::UInt(inner) => inner.to_string(),
        LiteralValue::String(inner) => format!("\"{}\"", inner),
    }
}

pub fn is_nested_type(schema: &SchemaType) -> bool {
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
#[deprecated = "Use the format specific renderers instead!"]
pub struct TemplateRenderer;

#[allow(deprecated)]
impl TemplateRenderer {
    pub fn new_format(format: Format) -> Box<dyn SchemaRenderer<String>> {
        Self::new(format, TemplateOptions::default())
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new(format: Format, options: TemplateOptions) -> Box<dyn SchemaRenderer<String>> {
        match format {
            Format::None => unreachable!(),

            #[cfg(feature = "json")]
            Format::Json => Box::new(super::jsonc_template::JsoncTemplateRenderer::new(options)),

            #[cfg(feature = "toml")]
            Format::Toml => Box::new(super::toml_template::TomlTemplateRenderer::new(options)),

            #[cfg(feature = "yaml")]
            Format::Yaml => Box::new(super::yaml_template::YamlTemplateRenderer::new(options)),
        }
    }
}

pub struct TemplateContext {
    pub depth: usize,
    pub options: TemplateOptions,

    format: Format,
    stack: VecDeque<String>,
}

impl TemplateContext {
    pub fn new(format: Format, options: TemplateOptions) -> Self {
        Self {
            depth: 0,
            format,
            options,
            stack: VecDeque::new(),
        }
    }

    pub fn indent(&self) -> String {
        if self.depth == 0 {
            String::new()
        } else {
            self.options.indent_char.repeat(self.depth)
        }
    }

    pub fn gap(&self) -> &str {
        if self.options.newline_between_fields {
            "\n\n"
        } else {
            "\n"
        }
    }

    pub fn create_comment(&self, field: &SchemaField) -> String {
        if !self.options.comments {
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

        if lines.is_empty() {
            return String::new();
        }

        let mut out = lines.join("\n");
        out.push('\n');
        out
    }

    pub fn create_field(&self, field: &SchemaField, property: String) -> String {
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

    pub fn get_comment_prefix(&self) -> &str {
        if self.format.is_json() {
            "// "
        } else {
            "# "
        }
    }

    pub fn get_stack_key(&self) -> String {
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

    pub fn is_expanded(&self, key: &String) -> bool {
        self.options.expand_fields.contains(key)
    }

    pub fn is_hidden(&self, field: &SchemaField) -> bool {
        let key = self.get_stack_key();

        field.hidden || self.options.hide_fields.contains(&key)
    }

    pub fn push_stack(&mut self, name: &str) {
        self.stack.push_back(name.to_owned());
    }

    pub fn pop_stack(&mut self) {
        self.stack.pop_back();
    }
}

pub fn render_array(_array: &ArrayType) -> RenderResult {
    Ok("[]".into())
}

pub fn render_boolean(boolean: &BooleanType) -> RenderResult {
    if let Some(default) = &boolean.default {
        return Ok(lit_to_string(default));
    }

    Ok("false".into())
}

pub fn render_enum(enu: &EnumType) -> RenderResult {
    if let Some(index) = &enu.default_index {
        if let Some(value) = enu.values.get(*index) {
            return Ok(lit_to_string(value));
        }
    }

    render_null()
}

pub fn render_float(float: &FloatType) -> RenderResult {
    if let Some(default) = &float.default {
        return Ok(lit_to_string(default));
    }

    Ok("0.0".into())
}

pub fn render_integer(integer: &IntegerType) -> RenderResult {
    if let Some(default) = &integer.default {
        return Ok(lit_to_string(default));
    }

    Ok("0".into())
}

pub fn render_literal(literal: &LiteralType) -> RenderResult {
    if let Some(value) = &literal.value {
        return Ok(lit_to_string(value));
    }

    render_null()
}

pub fn render_null() -> RenderResult {
    Ok("null".into())
}

pub fn render_object(_object: &ObjectType) -> RenderResult {
    Ok("{}".into())
}

pub fn render_reference(reference: &str) -> RenderResult {
    Ok(reference.into())
}

pub const EMPTY_STRING: &str = "\"\"";

pub fn render_string(string: &StringType) -> RenderResult {
    if let Some(default) = &string.default {
        return Ok(lit_to_string(default));
    }

    Ok(EMPTY_STRING.into())
}

pub fn render_struct(_structure: &StructType) -> RenderResult {
    Ok("{}".into())
}

pub fn render_tuple(
    tuple: &TupleType,
    mut render: impl FnMut(&SchemaType) -> RenderResult,
) -> RenderResult {
    let mut items = vec![];

    for item in &tuple.items_types {
        items.push(render(item)?);
    }

    Ok(format!("[{}]", items.join(", ")))
}

pub fn render_union(
    uni: &UnionType,
    mut render: impl FnMut(&SchemaType) -> RenderResult,
) -> RenderResult {
    if let Some(index) = &uni.default_index {
        if let Some(variant) = uni.variants_types.get(*index) {
            return render(variant);
        }
    }

    // We have a nullable type, so render the non-null value
    if uni.is_nullable() {
        if let Some(variant) = uni.variants_types.iter().find(|v| !v.is_null()) {
            return render(variant);
        }
    }

    render_null()
}

pub fn render_unknown() -> RenderResult {
    render_null()
}

pub fn validate_root(schemas: &IndexMap<String, SchemaType>) -> miette::Result<StructType> {
    let Some(schema) = schemas.values().last() else {
        return Err(miette!(
            "At least 1 schema is required to generate a template."
        ));
    };

    let SchemaType::Struct(root) = schema else {
        return Err(miette!("The last registered schema must be a struct type."));
    };

    Ok(root.to_owned())
}
