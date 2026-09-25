use super::pkl::*;
use super::template::*;
use crate::schema::{RenderResult, SchemaRenderer};
use indexmap::IndexMap;
use schematic_types::*;

/// Renders Pkl config templates with comments.
///
/// When the header amends or extends a module, such as one rendered by
/// `PklSchemaRenderer`, that module types every property of the template. A
/// list or a map then amends the value it's declared with, so that objects
/// within it take their type from the declaration. Otherwise lists and maps
/// are created with `new Listing` and `new Mapping`, as an untyped object that
/// holds elements can't be loaded.
pub struct PklTemplateRenderer {
    ctx: TemplateContext,
    schemas: IndexMap<String, Schema>,
    typed: bool,
}

impl PklTemplateRenderer {
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        PklTemplateRenderer::new(TemplateOptions::default())
    }

    pub fn new(mut options: TemplateOptions) -> Self {
        // A doc comment must be followed by the member it documents, which
        // neither a commented out field, nor a property of an object, is
        options.comment_prefix = "// ".into();

        let typed = options.header.lines().any(|line| {
            let line = line.trim_start();

            line.starts_with("amends ") || line.starts_with("extends ")
        });

        PklTemplateRenderer {
            ctx: TemplateContext::new(options),
            schemas: IndexMap::default(),
            typed,
        }
    }

    /// Render a list or a map, with its members on their own lines. Members
    /// are rendered one level deeper than the collection.
    fn render_collection(&self, class: &str, members: Vec<String>) -> String {
        let body = if members.is_empty() {
            "{}".to_owned()
        } else {
            let indent = self.ctx.options.indent_char.repeat(self.ctx.depth + 1);

            format!(
                "{{\n{}\n{}}}",
                members
                    .iter()
                    .map(|member| format!("{indent}{member}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                self.ctx.indent()
            )
        };

        if self.typed {
            body
        } else {
            format!("new {class} {body}")
        }
    }

    /// Render the properties of a struct, at the current depth.
    fn render_properties(
        &mut self,
        structure: &StructType,
        in_module: bool,
    ) -> RenderResult<Vec<String>> {
        let mut out = vec![];

        for (name, field) in structure.sorted_fields() {
            if field.flatten {
                continue;
            }

            self.ctx.push_stack(name);

            if !self.ctx.is_hidden(field) {
                // Every module already has an `output` property, which
                // configures how it renders, so it can't hold a setting
                if in_module && name == "output" {
                    out.push(format!(
                        "{}// The `output` setting cannot be set, as every Pkl module reserves it.",
                        self.ctx.indent()
                    ));
                } else {
                    let value = self.render_schema(self.ctx.validate_schema_variant(
                        self.ctx.get_stack_value().as_ref(),
                        &field.schema,
                    ))?;

                    out.push(self.create_field(field, assign(&quote_identifier(name), &value)));
                }
            }

            self.ctx.pop_stack();
        }

        Ok(out)
    }

    /// Render a property with its comment, commenting out every line of it
    /// when requested, as an object spans several.
    fn create_field(&self, field: &SchemaField, property: String) -> String {
        let key = self.ctx.get_stack_key();

        if !self.ctx.options.comment_fields.contains(&key) {
            return self.ctx.create_field(field, property);
        }

        let indent = self.ctx.indent();
        let prefix = self.ctx.get_comment_prefix();

        let lines = property
            .lines()
            .map(|line| {
                let line = line.strip_prefix(&indent).unwrap_or(line);

                format!("{indent}{prefix}{line}")
            })
            .collect::<Vec<_>>();

        format!(
            "{}{}",
            self.ctx.create_field_comment(field),
            lines.join("\n")
        )
    }
}

/// Return true if the value is an object body, `{ ... }`, which amends a value
/// rather than replacing it.
fn is_body(value: &str) -> bool {
    value.starts_with('{')
}

/// Render a value where an expression is required, such as an element of a
/// list, which creates an object instead of amending one.
fn to_expression(value: String) -> String {
    if is_body(&value) {
        format!("new {value}")
    } else {
        value
    }
}

/// Assign a value to a property or a map key, amending it with a body.
fn assign(key: &str, value: &str) -> String {
    if is_body(value) {
        format!("{key} {value}")
    } else {
        format!("{key} = {value}")
    }
}

/// Render a literal. A float must have a fraction to be read as a `Float`, and
/// a string must be escaped to not end early, or begin an interpolation.
fn render_value(value: &LiteralValue, is_float: bool) -> String {
    match value {
        LiteralValue::Bool(inner) => inner.to_string(),
        LiteralValue::F32(inner) => format_float(inner),
        LiteralValue::F64(inner) => format_float(inner),
        LiteralValue::Int(inner) if is_float => format!("{inner}.0"),
        LiteralValue::Int(inner) => inner.to_string(),
        LiteralValue::UInt(inner) if is_float => format!("{inner}.0"),
        LiteralValue::UInt(inner) => inner.to_string(),
        LiteralValue::String(inner) => quote_string(inner),
    }
}

impl SchemaRenderer<String> for PklTemplateRenderer {
    fn is_reference(&self, _name: &str) -> bool {
        false
    }

    fn render_array(&mut self, array: &ArrayType, _schema: &Schema) -> RenderResult<String> {
        let key = self.ctx.get_stack_key();
        let mut items = vec![];

        if self.ctx.is_expanded(&key) {
            self.ctx.depth += 1;
            items.push(to_expression(self.render_schema(&array.items_type)?));
            self.ctx.depth -= 1;
        }

        Ok(self.render_collection("Listing", items))
    }

    fn render_boolean(&mut self, boolean: &BooleanType, _schema: &Schema) -> RenderResult<String> {
        render_boolean(boolean)
    }

    fn render_enum(&mut self, enu: &EnumType, _schema: &Schema) -> RenderResult<String> {
        // `default_index` indexes the variants, not the values, so resolve it
        // through the enum rather than subscripting `values` directly.
        match enu.get_default().or_else(|| enu.values.first()) {
            Some(value) => Ok(render_value(value, false)),
            None => render_null(),
        }
    }

    fn render_float(&mut self, float: &FloatType, _schema: &Schema) -> RenderResult<String> {
        match &float.default {
            Some(default) => Ok(render_value(default, true)),
            None => Ok("0.0".into()),
        }
    }

    fn render_integer(&mut self, integer: &IntegerType, _schema: &Schema) -> RenderResult<String> {
        render_integer(integer)
    }

    fn render_literal(&mut self, literal: &LiteralType, _schema: &Schema) -> RenderResult<String> {
        Ok(render_value(&literal.value, false))
    }

    fn render_null(&mut self, _schema: &Schema) -> RenderResult<String> {
        render_null()
    }

    fn render_object(&mut self, object: &ObjectType, _schema: &Schema) -> RenderResult<String> {
        let key = self.ctx.get_stack_key();
        let mut entries = vec![];

        if self.ctx.is_expanded(&key) {
            self.ctx.depth += 1;
            let value = self.render_schema(&object.value_type)?;
            self.ctx.depth -= 1;

            let mut key = self.render_schema(&object.key_type)?;

            if key == EMPTY_STRING {
                key = "\"example\"".into();
            }

            entries.push(assign(&format!("[{key}]"), &value));
        }

        Ok(self.render_collection("Mapping", entries))
    }

    fn render_reference(&mut self, reference: &str, _schema: &Schema) -> RenderResult<String> {
        if let Some(schema) = self.schemas.get(reference) {
            return self.render_schema_without_reference(&schema.to_owned());
        }

        render_reference(reference)
    }

    fn render_string(&mut self, string: &StringType, _schema: &Schema) -> RenderResult<String> {
        match &string.default {
            Some(default) => Ok(render_value(default, false)),
            None => Ok(EMPTY_STRING.into()),
        }
    }

    fn render_struct(&mut self, structure: &StructType, _schema: &Schema) -> RenderResult<String> {
        // Serde encodes a duration as seconds and nanoseconds, which is what
        // a Pkl `Duration` is decoded into
        if is_duration(structure) {
            return Ok("0.s".into());
        }

        self.ctx.depth += 1;

        let out = self.render_properties(structure, false)?;

        self.ctx.depth -= 1;

        if out.is_empty() {
            return Ok("{}".into());
        }

        Ok(format!(
            "{{\n{}\n{}}}",
            out.join(self.ctx.gap()),
            self.ctx.indent()
        ))
    }

    fn render_tuple(&mut self, tuple: &TupleType, _schema: &Schema) -> RenderResult<String> {
        self.ctx.depth += 1;

        let mut items = vec![];

        for item in &tuple.items_types {
            items.push(to_expression(self.render_schema(item)?));
        }

        self.ctx.depth -= 1;

        // A pair is decoded into a sequence of its two values, the same as a
        // tuple, while a longer tuple can only be a list
        if let [first, second] = items.as_slice() {
            return Ok(format!("Pair({first}, {second})"));
        }

        Ok(self.render_collection("Listing", items))
    }

    fn render_union(&mut self, uni: &UnionType, _schema: &Schema) -> RenderResult<String> {
        render_union(uni, |schema| self.render_schema(schema))
    }

    fn render_unknown(&mut self, _schema: &Schema) -> RenderResult<String> {
        render_unknown()
    }

    fn render(&mut self, schemas: IndexMap<String, Schema>) -> RenderResult {
        self.schemas = schemas;

        let root = validate_root(&self.schemas)?;

        let SchemaType::Struct(structure) = &root.ty else {
            unreachable!("The root is validated to be a struct.");
        };

        // A module's properties are its body, without braces around them
        let template = self
            .render_properties(structure, true)?
            .join(self.ctx.gap());

        // The header may amend a module, which isn't a comment, so it's always
        // included
        Ok(format!(
            "{}{template}{}",
            self.ctx.options.header, self.ctx.options.footer
        ))
    }
}
