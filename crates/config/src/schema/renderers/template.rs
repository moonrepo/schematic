use crate::format::Format;
use crate::schema::{RenderResult, SchemaRenderer};
use indexmap::IndexMap;
use miette::{miette, IntoDiagnostic};
use schematic_types::*;
use std::collections::HashSet;

/// Options to control the rendered template.
#[derive(Default)]
pub struct TemplateOptions {
    /// File format to render.
    pub format: Format,

    /// Character(s) to use for indentation.
    pub indent_char: String,
}

#[cfg(feature = "json")]
fn lit_to_json_string(lit: &LiteralValue) -> miette::Result<String> {
    use serde_json::{to_string_pretty, Number, Value};

    let value = match lit {
        LiteralValue::Bool(inner) => Value::Bool(*inner),
        LiteralValue::F32(inner) => Value::Number(Number::from_f64(*inner as f64).unwrap()),
        LiteralValue::F64(inner) => Value::Number(Number::from_f64(*inner).unwrap()),
        LiteralValue::Int(inner) => Value::Number(Number::from(*inner)),
        LiteralValue::UInt(inner) => Value::Number(Number::from(*inner)),
        LiteralValue::String(inner) => Value::String(inner.to_owned()),
    };

    to_string_pretty(&value).into_diagnostic()
}

#[cfg(feature = "toml")]
fn lit_to_toml_string(lit: &LiteralValue) -> miette::Result<String> {
    use toml::{to_string_pretty, Value};

    let value = match lit {
        LiteralValue::Bool(inner) => Value::Boolean(*inner),
        LiteralValue::F32(inner) => Value::Float(*inner as f64),
        LiteralValue::F64(inner) => Value::Float(*inner),
        LiteralValue::Int(inner) => Value::Integer(*inner as i64),
        LiteralValue::UInt(inner) => Value::Integer(*inner as i64),
        LiteralValue::String(inner) => Value::String(inner.to_owned()),
    };

    to_string_pretty(&value).into_diagnostic()
}

#[cfg(feature = "yaml")]
fn lit_to_yaml_string(lit: &LiteralValue) -> miette::Result<String> {
    use serde_yaml::{to_string, Number, Value};

    let value = match lit {
        LiteralValue::Bool(inner) => Value::Bool(*inner),
        LiteralValue::F32(inner) => Value::Number(Number::from(*inner)),
        LiteralValue::F64(inner) => Value::Number(Number::from(*inner)),
        LiteralValue::Int(inner) => Value::Number(Number::from(*inner)),
        LiteralValue::UInt(inner) => Value::Number(Number::from(*inner)),
        LiteralValue::String(inner) => Value::String(inner.to_owned()),
    };

    to_string(&value).into_diagnostic()
}

macro_rules! literal_value {
    ($value:expr, $format:expr) => {
        #[cfg(feature = "json")]
        {
            if $format.is_json() {
                return lit_to_json_string($value);
            }
        }

        #[cfg(feature = "toml")]
        {
            if $format.is_toml() {
                return lit_to_toml_string($value);
            }
        }

        #[cfg(feature = "yaml")]
        {
            if $format.is_yaml() {
                return lit_to_yaml_string($value);
            }
        }
    };
}

/// Renders template files from a schema.
pub struct TemplateRenderer {
    depth: usize,
    options: TemplateOptions,
}

impl TemplateRenderer {
    pub fn with_format(format: Format) -> Self {
        Self::new(TemplateOptions {
            format,
            ..TemplateOptions::default()
        })
    }

    pub fn new(options: TemplateOptions) -> Self {
        let mut depth = 0;

        // In a JSON document, all fields are indented by
        // default because of the outer object block
        if options.format.is_json() {
            depth = 1;
        }

        Self { depth, options }
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
    fn is_reference(&self, name: &str) -> bool {
        false
    }

    fn render_array(&mut self, _array: &ArrayType) -> RenderResult {
        Ok("[]".into())
    }

    fn render_boolean(&mut self, boolean: &BooleanType) -> RenderResult {
        if let Some(default) = &boolean.default {
            literal_value!(default, &self.options.format);
        }

        Ok("false".into())
    }

    fn render_enum(&mut self, enu: &EnumType) -> RenderResult {
        if let Some(value) = enu.values.first() {
            literal_value!(value, &self.options.format);
        }

        self.render_null()
    }

    fn render_float(&mut self, float: &FloatType) -> RenderResult {
        if let Some(default) = &float.default {
            literal_value!(default, &self.options.format);
        }

        Ok("0.0".into())
    }

    fn render_integer(&mut self, integer: &IntegerType) -> RenderResult {
        if let Some(default) = &integer.default {
            literal_value!(default, &self.options.format);
        }

        Ok("0".into())
    }

    fn render_literal(&mut self, literal: &LiteralType) -> RenderResult {
        if let Some(value) = &literal.value {
            literal_value!(value, &self.options.format);
        }

        self.render_null()
    }

    fn render_null(&mut self) -> RenderResult {
        Ok("null".into())
    }

    fn render_object(&mut self, object: &ObjectType) -> RenderResult {
        Ok("{}".into())
    }

    fn render_reference(&mut self, reference: &str) -> RenderResult {
        Ok(reference.into())
    }

    fn render_string(&mut self, string: &StringType) -> RenderResult {
        if let Some(default) = &string.default {
            literal_value!(default, &self.options.format);
        }

        Ok("\"\"".into())
    }

    fn render_struct(&mut self, structure: &StructType) -> RenderResult {
        Ok("{}".into())
        // self.depth += 1;

        // let mut out = vec![];
        // let indent = self.indent();

        // for field in &structure.fields {
        //     if field.hidden {
        //         continue;
        //     }

        //     let name = &field.name;

        //     let mut prefix = "";
        //     let mut key = "".to_owned();
        //     let mut joiner = ": ";
        //     let mut value = "";
        //     let mut suffix = "";

        //     match &self.options.format {
        //         #[cfg(feature = "json")]
        //         Format::Json => {
        //             key = format!("\"{}\"", name);
        //         }

        //         #[cfg(feature = "toml")]
        //         Format::Toml => {
        //             key = format!("[{}]", name);
        //             joiner = "\n";
        //         }

        //         #[cfg(feature = "yaml")]
        //         Format::Yaml => {
        //             key = format!("{}", name);
        //         }

        //         _ => continue,
        //     };

        //     // let mut row = format!("{}{}", indent, field.name.as_ref().unwrap());

        //     // if field.optional {
        //     //     row.push_str("?: ");
        //     // } else {
        //     //     row.push_str(": ");
        //     // }

        //     // row.push_str(&self.render_schema(&field.type_of)?);

        //     // if matches!(self.options.object_format, ObjectFormat::Interface) {
        //     //     row.push(';');
        //     // } else {
        //     //     row.push(',');
        //     // }

        //     // let mut tags = vec![];

        //     // if let Some(default) = field.type_of.get_default() {
        //     //     tags.push(format!("@default {}", self.lit_to_string(default)));
        //     // }

        //     // if let Some(deprecated) = &field.deprecated {
        //     //     tags.push(if deprecated.is_empty() {
        //     //         "@deprecated".to_owned()
        //     //     } else {
        //     //         format!("@deprecated {}", deprecated)
        //     //     });
        //     // }

        //     // if let Some(env_var) = &field.env_var {
        //     //     tags.push(format!("@envvar {}", env_var));
        //     // }

        //     // out.push(self.wrap_in_comment(field.description.as_ref(), tags, row));
        // }

        // self.depth -= 1;

        // Ok(format!("{{\n{}\n{}}}", out.join("\n"), self.indent()))
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

        if self.options.format.is_json() {
            output = format!("{{\n{}\n}}", output);
        }

        Ok(output)
    }
}
