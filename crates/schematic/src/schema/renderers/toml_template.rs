use super::template::*;
use crate::format::Format;
use crate::schema::{RenderResult, SchemaRenderer};
use indexmap::IndexMap;
use schematic_types::*;
use std::collections::{BTreeMap, HashSet};
use std::mem;

struct Section {
    comment: String,
    table: StructType,
}

/// Renders TOML file templates.
pub struct TomlTemplateRenderer {
    ctx: TemplateContext,

    arrays: BTreeMap<String, Section>,
    tables: BTreeMap<String, Section>,
}

impl TomlTemplateRenderer {
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        TomlTemplateRenderer::new(TemplateOptions::default())
    }

    pub fn new(options: TemplateOptions) -> Self {
        TomlTemplateRenderer {
            ctx: TemplateContext::new(Format::Toml, options),
            arrays: BTreeMap::new(),
            tables: BTreeMap::new(),
        }
    }

    fn extract_sections(&mut self, doc: &mut StructType) {
        for field in &mut doc.fields {
            self.ctx.push_stack(&field.name);

            let comment = self.ctx.create_comment(field);

            match &mut field.type_of {
                SchemaType::Array(array) if array.items_type.is_struct() => {
                    if let SchemaType::Struct(table) = &mut *array.items_type {
                        let key = self.ctx.get_stack_key();

                        field.hidden = true;

                        self.extract_sections(table);

                        if !table.is_hidden() {
                            self.arrays.insert(
                                key,
                                Section {
                                    comment,
                                    table: table.to_owned(),
                                },
                            );
                        }
                    }
                }
                SchemaType::Struct(table) => {
                    let key = self.ctx.get_stack_key();

                    field.hidden = true;

                    self.extract_sections(table);

                    if !table.is_hidden() {
                        self.tables.insert(
                            key,
                            Section {
                                comment,
                                table: table.to_owned(),
                            },
                        );
                    }
                }
                _ => {}
            };

            self.ctx.pop_stack();
        }
    }
}

impl SchemaRenderer<String> for TomlTemplateRenderer {
    fn is_reference(&self, _name: &str) -> bool {
        false
    }

    fn render_array(&mut self, array: &ArrayType) -> RenderResult<String> {
        let key = self.ctx.get_stack_key();

        if !self.ctx.is_expanded(&key) {
            return render_array(array);
        }

        Ok(format!("[{}]", self.render_schema(&array.items_type)?))
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
        let key = self.ctx.get_stack_key();

        if !self.ctx.is_expanded(&key) || object.value_type.is_struct() {
            return render_object(object);
        }

        let comments = self.ctx.options.comments;

        // Objects are inline, so we can't show comments
        self.ctx.options.comments = false;

        let value = self.render_schema(&object.value_type)?;
        let mut key = self.render_schema(&object.key_type)?;

        if key == EMPTY_STRING {
            key = "example".into();
        }

        self.ctx.options.comments = comments;

        Ok(format!("{{ {key} = {value} }}"))
    }

    fn render_reference(&mut self, reference: &str) -> RenderResult<String> {
        render_reference(reference)
    }

    fn render_string(&mut self, string: &StringType) -> RenderResult<String> {
        render_string(string)
    }

    fn render_struct(&mut self, structure: &StructType) -> RenderResult<String> {
        let mut out = vec![];

        for field in &structure.fields {
            self.ctx.push_stack(&field.name);

            if !self.ctx.is_hidden(field) {
                let prop = format!("{} = {}", field.name, self.render_schema(&field.type_of)?,);

                out.push(self.ctx.create_field(field, prop));
            }

            self.ctx.pop_stack();
        }

        if out.is_empty() {
            return Ok("{}".into());
        }

        Ok(out.join(self.ctx.gap()))
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
        let mut root = validate_root(schemas)?;

        // Recursively extract all sections (arrays, objects)
        self.extract_sections(&mut root);

        // Then render each section accordingly
        let mut sections = vec![self.render_struct(&root)?];

        for (key, value) in mem::take(&mut self.arrays) {
            sections.push(format!(
                "{}[[{key}]]\n{}",
                value.comment,
                self.render_struct(&value.table)?
            ));
        }

        for (key, value) in mem::take(&mut self.tables) {
            sections.push(format!(
                "{}[{key}]\n{}",
                value.comment,
                self.render_struct(&value.table)?
            ));
        }

        let mut template = sections.join("\n\n");

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
