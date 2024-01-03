use crate::format::Format;
use crate::schema::{RenderResult, SchemaRenderer};
use indexmap::IndexMap;
use miette::miette;
use schematic_types::*;
use std::collections::{HashSet, VecDeque};

/// Options to control the rendered template.
#[derive(Default)]
pub struct TemplateOptions {
    /// File format to render.
    pub format: Format,

    /// Character(s) to use for indentation.
    pub indent_char: String,
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

/// Renders template files from a schema.
pub struct TemplateRenderer {
    depth: usize,
    options: TemplateOptions,
    stack: VecDeque<String>,
}

impl TemplateRenderer {
    pub fn with_format(format: Format) -> Self {
        Self::new(TemplateOptions {
            format,
            ..TemplateOptions::default()
        })
    }

    pub fn new(options: TemplateOptions) -> Self {
        Self {
            depth: 0,
            options,
            stack: VecDeque::new(),
        }
    }

    fn indent(&self) -> String {
        let chars = if self.options.indent_char.is_empty() {
            "  "
        } else {
            &self.options.indent_char
        };

        if self.depth == 0 {
            String::new()
        } else {
            chars.repeat(self.depth)
        }
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
        if let Some(value) = enu.values.first() {
            return Ok(lit_to_string(value));
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
            if self.options.format.is_json() {
                let mut out = vec![];

                self.depth += 1;

                for field in &structure.fields {
                    if field.hidden {
                        continue;
                    }

                    out.push(format!(
                        "{}\"{}\": {},",
                        self.indent(),
                        field.name,
                        self.render_schema(&field.type_of)?
                    ));
                }

                self.depth -= 1;

                return Ok(format!("{{\n{}\n{}}}", out.join("\n"), self.indent()));
            }
        }

        #[cfg(feature = "toml")]
        {
            if self.options.format.is_toml() {
                let mut out = vec![];
                let mut structs = vec![];

                for field in &structure.fields {
                    if field.hidden {
                        continue;
                    }

                    // Nested structs have weird syntax, so render them
                    // at the bottom after other fields
                    if matches!(field.type_of, SchemaType::Struct(_)) {
                        structs.push(field);
                        continue;
                    }

                    out.push(format!(
                        "{} = {}",
                        field.name,
                        self.render_schema(&field.type_of)?
                    ));
                }

                for field in structs {
                    self.stack.push_back(field.name.clone());

                    out.push(format!(
                        "\n[{}]\n{}",
                        self.stack.iter().cloned().collect::<Vec<_>>().join("."),
                        self.render_schema(&field.type_of)?
                    ));

                    self.stack.pop_back();
                }

                return Ok(out.join("\n"));
            }
        }

        #[cfg(feature = "yaml")]
        {
            if self.options.format.is_yaml() {
                let mut out = vec![];

                for field in &structure.fields {
                    if field.hidden {
                        continue;
                    }

                    let is_nested = matches!(field.type_of, SchemaType::Struct(_));

                    if is_nested {
                        self.depth += 1;
                    }

                    let value = self.render_schema(&field.type_of)?;

                    if is_nested {
                        self.depth -= 1;
                    }

                    out.push(format!(
                        "{}{}:{}{}",
                        self.indent(),
                        field.name,
                        if is_nested { "\n" } else { " " },
                        value
                    ));
                }

                return Ok(out.join("\n"));
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

    fn render_union(&mut self, _uni: &UnionType) -> RenderResult {
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

        let mut output = self.render_struct(schema)?;

        if self.options.format.is_toml() || self.options.format.is_yaml() {
            output.push_str("\n");
        }

        Ok(output)
    }
}
