use super::template::*;
use crate::format::Format;
use crate::schema::{RenderResult, SchemaRenderer};
use indexmap::IndexMap;
use schematic_types::*;

/// Renders YAML config templates.
pub struct YamlTemplateRenderer {
    ctx: TemplateContext,
    schemas: IndexMap<String, Schema>,
}

impl YamlTemplateRenderer {
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        YamlTemplateRenderer::new(TemplateOptions::default())
    }

    pub fn new(options: TemplateOptions) -> Self {
        YamlTemplateRenderer {
            ctx: TemplateContext::new(Format::Yaml, options),
            schemas: IndexMap::default(),
        }
    }
}

impl SchemaRenderer<String> for YamlTemplateRenderer {
    fn is_reference(&self, _name: &str) -> bool {
        false
    }

    fn render_array(&mut self, array: &ArrayType, _schema: &Schema) -> RenderResult<String> {
        let key = self.ctx.get_stack_key();

        if !self.ctx.is_expanded(&key) {
            return render_array(array);
        }

        let items_type = self.ctx.resolve_schema(&array.items_type, &self.schemas);

        if !items_type.is_struct() {
            return Ok(format!("[{}]", self.render_schema(&items_type)?));
        }

        self.ctx.depth += 2;

        let mut item = self.render_schema(&items_type)?;

        self.ctx.depth -= 2;

        item.replace_range(2..3, "-");

        Ok(item)
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
        let value_type = self.ctx.resolve_schema(&object.value_type, &self.schemas);

        if !self.ctx.is_expanded(&key) || !value_type.is_struct() {
            return render_object(object);
        }

        self.ctx.depth += 2;

        let value = self.render_schema(&value_type)?;

        self.ctx.depth -= 1;

        let item_indent = self.ctx.indent();

        self.ctx.depth -= 1;

        let mut key = self.render_schema(&object.key_type)?;

        if key == EMPTY_STRING {
            key = "example".into();
        }

        Ok(format!("{item_indent}{key}:\n{value}"))
    }

    fn render_reference(&mut self, reference: &str, _schema: &Schema) -> RenderResult<String> {
        if let Some(schema) = self.schemas.get(reference) {
            return self.render_schema_without_reference(&schema.to_owned());
        }

        render_reference(reference)
    }

    fn render_string(&mut self, string: &StringType, _schema: &Schema) -> RenderResult<String> {
        render_string(string)
    }

    fn render_struct(&mut self, structure: &StructType, _schema: &Schema) -> RenderResult<String> {
        let mut out = vec![];

        for (name, field) in &structure.fields {
            self.ctx.push_stack(name);

            if self.ctx.is_hidden(field) {
                self.ctx.pop_stack();
                continue;
            }

            let is_nested = is_nested_type(&field.schema.ty);

            if is_nested {
                self.ctx.depth += 1;
            }

            let value = self.render_schema(&field.schema)?;
            let prop = format!(
                "{}:{}{}",
                name,
                if value.contains('\n') || is_nested {
                    "\n"
                } else {
                    " "
                },
                value
            );

            if is_nested {
                self.ctx.depth -= 1;
            }

            out.push(self.ctx.create_field(field, prop));

            self.ctx.pop_stack();
        }

        if out.is_empty() {
            return Ok("{}".into());
        }

        Ok(out.join(self.ctx.gap()))
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

    fn render(&mut self, schemas: IndexMap<String, Schema>) -> RenderResult {
        self.schemas = schemas;

        let root = validate_root(&self.schemas)?;
        let mut template = self.render_schema_without_reference(&root)?;

        // Inject the header and footer
        template = format!(
            "{}{template}{}",
            self.ctx.options.header, self.ctx.options.footer
        );

        // And always add a trailing newline
        template.push('\n');

        Ok(template)
    }
}
