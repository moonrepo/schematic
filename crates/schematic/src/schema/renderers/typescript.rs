use crate::schema::{RenderResult, SchemaRenderer};
use indexmap::IndexMap;
use schematic_types::*;
use std::collections::{HashMap, HashSet};

/// Format of a TypeScript enum.
#[derive(Default)]
pub enum EnumFormat {
    /// Native enum: `enum Name { Foo, Bar }`
    Enum,
    /// Native enum with values: `enum Name { Foo = 'foo', Bar = 'bar' }`
    ValuedEnum,
    /// String union: `type Name = 'foo' | 'bar';`
    #[default]
    Union,
}

/// Format of a TypeScript object.
#[derive(Default)]
pub enum ObjectFormat {
    /// Interface: `interface Foo {}`
    #[default]
    Interface,
    /// Type alias: `type Foo = {};`
    Type,
}

/// Options to control the rendered TypeScript output.
#[derive(Default)]
pub struct TypeScriptOptions {
    /// Render a `const enum` instead of a `enum`.
    pub const_enum: bool,

    /// Disable references and render all types inline recursively.
    pub disable_references: bool,

    /// Format to render enums, either an `enum` or a `type` union.
    pub enum_format: EnumFormat,

    /// List of references to exclude from exporting as a type.
    pub exclude_references: Vec<String>,

    /// Map of relative import file paths to a list of types to import.
    /// Will be rendered as an `import type {} from 'path';` declaration.
    pub external_types: HashMap<String, Vec<String>>,

    /// Character(s) to use for indentation.
    pub indent_char: String,

    /// Format to render objects, either an `interface` or `type`.
    pub object_format: ObjectFormat,
}

/// Renders TypeScript types from a schema.
#[derive(Default)]
pub struct TypeScriptRenderer<'gen> {
    depth: usize,
    options: TypeScriptOptions,
    references: Option<&'gen HashSet<String>>,
}

impl<'gen> TypeScriptRenderer<'gen> {
    pub fn new(options: TypeScriptOptions) -> Self {
        Self {
            depth: 0,
            options,
            references: None,
        }
    }

    fn indent(&self) -> String {
        let chars = if self.options.indent_char.is_empty() {
            "\t"
        } else {
            &self.options.indent_char
        };

        if self.depth == 0 {
            String::new()
        } else {
            chars.repeat(self.depth)
        }
    }

    fn is_excluded(&self, name: &str) -> bool {
        self.options.exclude_references.iter().any(|r| r == name)
    }

    fn is_external(&self, name: &str) -> bool {
        for externals in self.options.external_types.values() {
            if externals.iter().any(|e| e == name) {
                return true;
            }
        }

        false
    }

    fn is_string_union_enum(&self, enu: &EnumType) -> bool {
        matches!(self.options.enum_format, EnumFormat::Union)
            // No variants
            || enu.variants.is_none()
            // Unit enum with a fallback variant
            || enu.variants.as_ref().is_some_and(|v| v.len() != enu.values.len())
            || self.options.disable_references
    }

    fn export_type_alias(&mut self, name: &str, value: String) -> RenderResult {
        Ok(format!("export type {name} = {value};"))
    }

    fn export_enum_type(&mut self, name: &str, enu: &EnumType) -> RenderResult {
        let value = self.render_enum(enu)?;

        let output = if self.is_string_union_enum(enu) {
            self.export_type_alias(name, value)?
        } else {
            let out = format!("enum {name} {value}");

            if self.options.const_enum {
                format!("export const {out}")
            } else {
                format!("export {out}")
            }
        };

        // Ok(self.wrap_in_comment(enu.description.as_ref(), vec![], output))
        Ok(output)
    }

    fn export_object_type(&mut self, name: &str, structure: &StructType) -> RenderResult {
        let value = self.render_struct(structure)?;

        let output = if matches!(self.options.object_format, ObjectFormat::Interface) {
            format!("export interface {name} {value}")
        } else {
            self.export_type_alias(name, value)?
        };

        // Ok(self.wrap_in_comment(structure.description.as_ref(), vec![], output))
        Ok(output)
    }

    fn render_enum_or_union(&mut self, enu: &EnumType) -> RenderResult {
        if self.is_string_union_enum(enu) {
            // Map using variants instead of values (when available),
            // so that the fallback variant is included
            let variants_types = if let Some(variants) = &enu.variants {
                variants
                    .iter()
                    .filter_map(|v| {
                        if v.hidden {
                            None
                        } else {
                            Some(v.schema.clone())
                        }
                    })
                    .collect::<Vec<_>>()
            } else {
                enu.values
                    .iter()
                    .map(|v| Box::new(Schema::literal_value(v.clone())))
                    .collect::<Vec<_>>()
            };

            return self.render_union(&UnionType {
                variants_types,
                variants: enu.variants.clone(),
                ..Default::default()
            });
        }

        self.depth += 1;

        let mut out = vec![];
        let indent = self.indent();

        for variant in enu.variants.as_ref().unwrap() {
            if variant.hidden {
                continue;
            }

            let field = if matches!(self.options.enum_format, EnumFormat::ValuedEnum) {
                format!(
                    "{}{} = {},",
                    indent,
                    variant.name,
                    self.render_schema(&variant.schema)?
                )
            } else {
                format!("{}{},", indent, variant.name)
            };

            let mut tags = vec![];

            if let Some(default) = variant.schema.get_default() {
                tags.push(format!("@default {}", self.lit_to_string(default)));
            }

            out.push(self.wrap_in_comment(variant.description.as_ref(), tags, field));
        }

        self.depth -= 1;

        Ok(format!("{{\n{}\n{}}}", out.join("\n"), self.indent()))
    }

    fn lit_to_string(&self, lit: &LiteralValue) -> String {
        match lit {
            LiteralValue::Bool(inner) => inner.to_string(),
            LiteralValue::F32(inner) => inner.to_string(),
            LiteralValue::F64(inner) => inner.to_string(),
            LiteralValue::Int(inner) => inner.to_string(),
            LiteralValue::UInt(inner) => inner.to_string(),
            LiteralValue::String(inner) => format!("'{inner}'"),
        }
    }

    fn wrap_in_comment(
        &self,
        comment: Option<&String>,
        tags: Vec<String>,
        value: String,
    ) -> String {
        let indent = self.indent();
        let mut lines = vec![];

        if let Some(comment) = comment {
            lines.extend(
                comment
                    .trim()
                    .split('\n')
                    .map(|c| c.trim().to_owned())
                    .collect::<Vec<_>>(),
            );
        }

        if !tags.is_empty() {
            if !lines.is_empty() {
                lines.push("".to_owned());
            }

            lines.extend(tags);
        }

        if lines.is_empty() {
            return value;
        }

        if lines.len() == 1 {
            return format!("{}/** {} */\n{}", indent, lines[0], value);
        }

        let mut out = vec![format!("{}/**", indent)];

        for line in lines {
            if line.is_empty() {
                out.push(format!("{} *", indent));
            } else {
                out.push(format!("{} * {}", indent, line.trim()));
            }
        }

        out.push(format!("{} */", indent));

        format!("{}\n{}", out.join("\n"), value)
    }
}

impl<'gen> SchemaRenderer<'gen, String> for TypeScriptRenderer<'gen> {
    fn is_reference(&self, name: &str) -> bool {
        if self.options.disable_references {
            return false;
        }

        if self.references.is_some_and(|refs| refs.contains(name)) {
            return true;
        }

        self.is_external(name)
    }

    fn render_array(&mut self, array: &ArrayType) -> RenderResult {
        let out = self.render_schema(&array.items_type)?;

        Ok(if out.contains('|') {
            format!("({out})[]")
        } else {
            format!("{out}[]")
        })
    }

    fn render_boolean(&mut self, _boolean: &BooleanType) -> RenderResult {
        Ok("boolean".into())
    }

    fn render_enum(&mut self, enu: &EnumType) -> RenderResult {
        self.render_enum_or_union(enu)
    }

    fn render_float(&mut self, float: &FloatType) -> RenderResult {
        if let Some(values) = &float.enum_values {
            return Ok(values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" | "));
        }

        Ok("number".into())
    }

    fn render_integer(&mut self, integer: &IntegerType) -> RenderResult {
        if let Some(values) = &integer.enum_values {
            return Ok(values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" | "));
        }

        Ok("number".into())
    }

    fn render_literal(&mut self, literal: &LiteralType) -> RenderResult {
        if let Some(value) = &literal.value {
            return Ok(self.lit_to_string(value));
        }

        self.render_unknown()
    }

    fn render_null(&mut self) -> RenderResult {
        Ok("null".into())
    }

    fn render_object(&mut self, object: &ObjectType) -> RenderResult {
        Ok(format!(
            "Record<{}, {}>",
            self.render_schema(&object.key_type)?,
            self.render_schema(&object.value_type)?
        ))
    }

    fn render_reference(&mut self, reference: &str) -> RenderResult {
        Ok(reference.into())
    }

    fn render_string(&mut self, string: &StringType) -> RenderResult {
        if let Some(values) = &string.enum_values {
            return Ok(values
                .iter()
                .map(|v| format!("'{v}'"))
                .collect::<Vec<_>>()
                .join(" | "));
        }

        Ok("string".into())
    }

    fn render_struct(&mut self, structure: &StructType) -> RenderResult {
        self.depth += 1;

        let mut out = vec![];
        let indent = self.indent();

        for field in &structure.fields {
            if field.hidden {
                continue;
            }

            let mut row = format!("{}{}", indent, field.name);

            if field.optional {
                row.push_str("?: ");
            } else {
                row.push_str(": ");
            }

            row.push_str(&self.render_schema(&field.schema)?);

            if matches!(self.options.object_format, ObjectFormat::Interface) {
                row.push(';');
            } else {
                row.push(',');
            }

            let mut tags = vec![];

            if let Some(default) = field.schema.get_default() {
                tags.push(format!("@default {}", self.lit_to_string(default)));
            }

            if let Some(deprecated) = &field.deprecated {
                tags.push(if deprecated.is_empty() {
                    "@deprecated".to_owned()
                } else {
                    format!("@deprecated {deprecated}")
                });
            }

            if let Some(env_var) = &field.env_var {
                tags.push(format!("@envvar {env_var}"));
            }

            out.push(self.wrap_in_comment(field.description.as_ref(), tags, row));
        }

        self.depth -= 1;

        Ok(format!("{{\n{}\n{}}}", out.join("\n"), self.indent()))
    }

    fn render_tuple(&mut self, tuple: &TupleType) -> RenderResult {
        let mut items = vec![];

        for item in &tuple.items_types {
            items.push(self.render_schema(item)?);
        }

        Ok(format!("[{}]", items.join(", ")))
    }

    fn render_union(&mut self, uni: &UnionType) -> RenderResult {
        let mut items = vec![];

        for item in &uni.variants_types {
            items.push(self.render_schema(item)?);
        }

        Ok(items.join(" | "))
    }

    fn render_unknown(&mut self) -> RenderResult {
        Ok("unknown".into())
    }

    fn render(
        &mut self,
        schemas: &'gen IndexMap<String, Schema>,
        references: &'gen HashSet<String>,
    ) -> RenderResult {
        self.references = Some(references);

        let mut outputs = vec![
            "// Automatically generated by schematic. DO NOT MODIFY!".to_string(),
            "/* eslint-disable */".to_string(),
        ];

        let mut imports = vec![];

        for (import, types) in &self.options.external_types {
            let mut imported_types = types.to_vec();
            imported_types.sort();

            imports.push(format!(
                "import type {{ {} }} from '{import}';",
                imported_types.join(", "),
            ));
        }

        if !imports.is_empty() {
            outputs.push(imports.join("\n"));
        }

        for (name, schema) in schemas {
            if self.is_excluded(name) {
                continue;
            }

            outputs.push(match &schema.type_of {
                SchemaType::Enum(inner) => self.export_enum_type(name, inner)?,
                SchemaType::Struct(inner) => self.export_object_type(name, inner)?,
                _ => {
                    let out = self.render_schema(schema)?;
                    self.export_type_alias(name, out)?
                }
            });
        }

        Ok(outputs.join("\n\n"))
    }
}
