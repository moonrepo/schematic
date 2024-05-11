use super::template::*;
use crate::format::Format;
use crate::schema::{RenderResult, SchemaRenderer};
use indexmap::IndexMap;
use schematic_types::*;
use std::collections::HashSet;

/// Renders JSON config templates with comments.
pub struct JsoncTemplateRenderer<'gen> {
    ctx: TemplateContext,
    schemas: Option<&'gen IndexMap<String, Schema>>,
}

impl<'gen> JsoncTemplateRenderer<'gen> {
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        JsoncTemplateRenderer::new(TemplateOptions::default())
    }

    pub fn new(options: TemplateOptions) -> Self {
        JsoncTemplateRenderer {
            ctx: TemplateContext::new(Format::Json, options),
            schemas: None,
        }
    }
}

impl<'gen> SchemaRenderer<'gen, String> for JsoncTemplateRenderer<'gen> {
    fn is_reference(&self, _name: &str) -> bool {
        false
    }

    fn render_array(&mut self, array: &ArrayType, _schema: &Schema) -> RenderResult<String> {
        let key = self.ctx.get_stack_key();

        if !self.ctx.is_expanded(&key) {
            return render_array(array);
        }

        self.ctx.depth += 1;

        let item_indent = self.ctx.indent();
        let item = self.render_schema(&array.items_type)?;

        self.ctx.depth -= 1;

        Ok(format!("[\n{}{item}\n{}]", item_indent, self.ctx.indent()))
    }

    fn render_boolean(&mut self, boolean: &BooleanType, _schema: &Schema) -> RenderResult<String> {
        render_boolean(boolean)
    }

    fn render_enum(&mut self, enu: &EnumType, _schema: &Schema) -> RenderResult<String> {
        render_enum(enu)
    }

    fn render_float(&mut self, float: &FloatType, _schema: &Schema) -> RenderResult<String> {
        render_float(float)
    }

    fn render_integer(&mut self, integer: &IntegerType, _schema: &Schema) -> RenderResult<String> {
        render_integer(integer)
    }

    fn render_literal(&mut self, literal: &LiteralType, _schema: &Schema) -> RenderResult<String> {
        render_literal(literal)
    }

    fn render_null(&mut self, _schema: &Schema) -> RenderResult<String> {
        render_null()
    }

    fn render_object(&mut self, object: &ObjectType, _schema: &Schema) -> RenderResult<String> {
        let key = self.ctx.get_stack_key();

        if !self.ctx.is_expanded(&key) {
            return render_object(object);
        }

        self.ctx.depth += 1;

        let item_indent = self.ctx.indent();
        let value = self.render_schema(&object.value_type)?;

        self.ctx.depth -= 1;

        let mut key = self.render_schema(&object.key_type)?;

        if key == EMPTY_STRING {
            key = "\"example\"".into();
        }

        Ok(format!(
            "{{\n{}{key}: {value}\n{}}}",
            item_indent,
            self.ctx.indent()
        ))
    }

    fn render_reference(&mut self, reference: &str, _schema: &Schema) -> RenderResult<String> {
        if let Some(schemas) = &self.schemas {
            if let Some(schema) = schemas.get(reference) {
                return self.render_schema_without_reference(schema);
            }
        }

        render_reference(reference)
    }

    fn render_string(&mut self, string: &StringType, _schema: &Schema) -> RenderResult<String> {
        render_string(string)
    }

    fn render_struct(&mut self, structure: &StructType, _schema: &Schema) -> RenderResult<String> {
        let mut out = vec![];
        let last_index = structure.fields.len() - 1;

        self.ctx.depth += 1;

        for (index, (name, field)) in structure.fields.iter().enumerate() {
            self.ctx.push_stack(name);

            if !self.ctx.is_hidden(field) {
                let prop = format!(
                    "\"{}\": {}{}",
                    name,
                    self.render_schema(field)?,
                    if index == last_index { "" } else { "," }
                );

                out.push(self.ctx.create_field(field, prop));
            }

            self.ctx.pop_stack();
        }

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
        render_tuple(tuple, |schema| self.render_schema(schema))
    }

    fn render_union(&mut self, uni: &UnionType, _schema: &Schema) -> RenderResult<String> {
        render_union(uni, |schema| self.render_schema(schema))
    }

    fn render_unknown(&mut self, _schema: &Schema) -> RenderResult<String> {
        render_unknown()
    }

    fn render(
        &mut self,
        schemas: &'gen IndexMap<String, Schema>,
        _references: &'gen HashSet<String>,
    ) -> RenderResult {
        self.schemas = Some(schemas);

        let root = validate_root(schemas)?;
        let mut template = self.render_schema_without_reference(&root)?;

        // Inject the header and footer
        if self.ctx.options.comments {
            template = format!(
                "{}{template}{}",
                self.ctx.options.header, self.ctx.options.footer
            );
        }

        Ok(template)
    }
}
