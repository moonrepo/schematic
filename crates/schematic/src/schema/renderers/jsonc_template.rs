use super::template::*;
use crate::format::Format;
use crate::schema::{RenderResult, SchemaRenderer};
use indexmap::IndexMap;
use schematic_types::*;
use std::collections::HashSet;

/// Renders JSON file templates with comments.
pub struct JsoncTemplateRenderer {
    ctx: TemplateContext,
}

impl JsoncTemplateRenderer {
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        JsoncTemplateRenderer::new(TemplateOptions::default())
    }

    pub fn new(options: TemplateOptions) -> Self {
        JsoncTemplateRenderer {
            ctx: TemplateContext::new(Format::Json, options),
        }
    }
}

impl SchemaRenderer<String> for JsoncTemplateRenderer {
    fn is_reference(&self, _name: &str) -> bool {
        false
    }

    fn render_array(&mut self, array: &ArrayType) -> RenderResult<String> {
        let key = self.ctx.get_stack_key();

        if !self.ctx.is_expanded(&key) {
            return render_array(array);
        }

        self.ctx.depth += 1;

        let item_indent = self.ctx.indent();
        let item = self.render_schema(&array.items_type)?;

        self.ctx.depth -= 1;

        Ok(format!(
            "[\n{}{}\n{}]",
            item_indent,
            item,
            self.ctx.indent()
        ))
    }

    fn render_boolean(&mut self, boolean: &BooleanType) -> RenderResult<String> {
        render_boolean(boolean)
    }

    fn render_enum(&mut self, enu: &EnumType) -> RenderResult<String> {
        render_enum(enu)
    }

    fn render_float(&mut self, float: &FloatType) -> RenderResult<String> {
        render_float(float)
    }

    fn render_integer(&mut self, integer: &IntegerType) -> RenderResult<String> {
        render_integer(integer)
    }

    fn render_literal(&mut self, literal: &LiteralType) -> RenderResult<String> {
        render_literal(literal)
    }

    fn render_null(&mut self) -> RenderResult<String> {
        render_null()
    }

    fn render_object(&mut self, object: &ObjectType) -> RenderResult<String> {
        render_object(object)
    }

    fn render_reference(&mut self, reference: &str) -> RenderResult<String> {
        render_reference(reference)
    }

    fn render_string(&mut self, string: &StringType) -> RenderResult<String> {
        render_string(string)
    }

    fn render_struct(&mut self, structure: &StructType) -> RenderResult<String> {
        let mut out = vec![];
        let last_index = structure.fields.len() - 1;

        self.ctx.depth += 1;

        for (index, field) in structure.fields.iter().enumerate() {
            self.ctx.push_stack(&field.name);

            if !self.ctx.is_hidden(field) {
                let prop = format!(
                    "\"{}\": {}{}",
                    field.name,
                    self.render_schema(&field.type_of)?,
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

    fn render_tuple(&mut self, tuple: &TupleType) -> RenderResult<String> {
        render_tuple(tuple, |schema| self.render_schema(schema))
    }

    fn render_union(&mut self, uni: &UnionType) -> RenderResult<String> {
        render_union(uni, |schema| self.render_schema(schema))
    }

    fn render_unknown(&mut self) -> RenderResult<String> {
        render_unknown()
    }

    fn render(
        &mut self,
        schemas: &IndexMap<String, SchemaType>,
        _references: &HashSet<String>,
    ) -> RenderResult {
        validat_schemas(schemas)?;

        let mut template = self.render_schema(schemas.values().last().unwrap())?;

        // Inject the header and footer
        if self.ctx.options.comments {
            template = format!(
                "{}{}{}",
                self.ctx.options.header, template, self.ctx.options.footer
            );
        }

        Ok(template)
    }
}
