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
    pub exclude_references: HashSet<String>,

    /// Map of relative import file paths to a list of types to import.
    /// Will be rendered as an `import type {} from 'path';` declaration.
    pub external_types: HashMap<String, HashSet<String>>,

    /// Character(s) to use for indentation.
    pub indent_char: String,

    /// Format to render objects, either an `interface` or `type`.
    pub object_format: ObjectFormat,
}

/// Renders TypeScript types from a schema.
#[derive(Default)]
pub struct TypeScriptRenderer {
    depth: usize,
    options: TypeScriptOptions,
    references: HashSet<String>,
}

impl TypeScriptRenderer {
    pub fn new(options: TypeScriptOptions) -> Self {
        Self {
            depth: 0,
            options,
            references: HashSet::new(),
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
        self.options.exclude_references.contains(name)
    }

    fn is_external(&self, name: &str) -> bool {
        for externals in self.options.external_types.values() {
            if externals.contains(name) {
                return true;
            }
        }

        false
    }

    fn is_string_union_enum(&self, enu: &EnumType) -> bool {
        matches!(self.options.enum_format, EnumFormat::Union)
            || enu.variants.is_none()
            || self.options.disable_references
    }

    fn export_type_alias(&mut self, name: &str, value: String) -> RenderResult {
        Ok(format!("export type {} = {};", name, value))
    }

    fn export_enum_type(&mut self, name: &str, enu: &EnumType) -> RenderResult {
        let value = self.render_enum(enu)?;

        if self.is_string_union_enum(enu) {
            return self.export_type_alias(name, value);
        }

        let out = format!("enum {} {}", name, value);

        Ok(if self.options.const_enum {
            format!("export const {}", out)
        } else {
            format!("export {}", out)
        })
    }

    fn export_object_type(&mut self, name: &str, structure: &StructType) -> RenderResult {
        let value = self.render_struct(structure)?;

        if matches!(self.options.object_format, ObjectFormat::Interface) {
            return Ok(format!("export interface {} {}", name, value));
        }

        self.export_type_alias(name, value)
    }

    fn render_enum_or_union(&mut self, enu: &EnumType) -> RenderResult {
        if self.is_string_union_enum(enu) {
            return self.render_union(&UnionType {
                variants_types: enu
                    .values
                    .iter()
                    .map(|v| Box::new(SchemaType::Literal(v.clone())))
                    .collect(),
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

            if let Some(variant_name) = &variant.name {
                let mut field = if matches!(self.options.enum_format, EnumFormat::ValuedEnum) {
                    format!(
                        "{}{} = {},",
                        indent,
                        variant_name,
                        self.render_schema(&variant.type_of)?
                    )
                } else {
                    format!("{}{},", indent, variant_name)
                };

                if let Some(comment) = &variant.description {
                    field = self.wrap_in_comment(comment.trim(), field);
                }

                out.push(field);
            }
        }

        self.depth -= 1;

        Ok(format!("{{\n{}\n{}}}", out.join("\n"), self.indent()))
    }

    fn wrap_in_comment(&self, comment: &str, value: String) -> String {
        let indent = self.indent();

        if comment.starts_with('*') {
            let mut out = vec![format!("{}/**", indent)];

            for line in comment.split('\n') {
                out.push(format!("{} {}", indent, line.trim()));
            }

            out.push(format!("{} */", indent));

            format!("{}\n{}", out.join("\n"), value)
        } else {
            format!("{}// {}\n{}", indent, comment.trim(), value)
        }
    }
}

impl SchemaRenderer<String> for TypeScriptRenderer {
    fn is_reference(&self, name: &str) -> bool {
        if self.options.disable_references {
            return false;
        }

        if self.references.contains(name) {
            return true;
        }

        self.is_external(name)
    }

    fn render_array(&mut self, array: &ArrayType) -> RenderResult {
        let out = self.render_schema(&array.items_type)?;

        Ok(if out.contains('|') {
            format!("({})[]", out)
        } else {
            format!("{}[]", out)
        })
    }

    fn render_boolean(&mut self) -> RenderResult {
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
            return Ok(match value {
                LiteralValue::Bool(inner) => inner.to_string(),
                LiteralValue::Float(inner) => inner.to_owned(),
                LiteralValue::Int(inner) => inner.to_string(),
                LiteralValue::UInt(inner) => inner.to_string(),
                LiteralValue::String(inner) => format!("'{inner}'"),
            });
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
                .map(|v| format!("'{}'", v))
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

            let mut row = format!("{}{}", indent, field.name.as_ref().unwrap());

            if field.optional {
                row.push_str("?: ");
            } else {
                row.push_str(": ");
            }

            if structure.partial || field.nullable {
                row.push_str(&self.render_schema(&SchemaType::nullable(field.type_of.clone()))?);
            } else {
                row.push_str(&self.render_schema(&field.type_of)?);
            }

            if matches!(self.options.object_format, ObjectFormat::Interface) {
                row.push(';');
            } else {
                row.push(',');
            }

            if let Some(comment) = &field.description {
                row = self.wrap_in_comment(comment.trim(), row);
            }

            out.push(row);
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
        schemas: &IndexMap<String, SchemaType>,
        references: &HashSet<String>,
    ) -> RenderResult {
        self.references.extend(references.to_owned());

        let mut outputs = vec![
            "// Automatically generated by schematic. DO NOT MODIFY!".to_string(),
            "/* eslint-disable */".to_string(),
        ];

        let mut imports = vec![];

        for (import, types) in &self.options.external_types {
            let mut imported_types = types.iter().cloned().collect::<Vec<_>>();
            imported_types.sort();

            imports.push(format!(
                "import type {{ {} }} from '{}';",
                imported_types.join(", "),
                import
            ));
        }

        if !imports.is_empty() {
            outputs.push(imports.join("\n"));
        }

        for (name, schema) in schemas {
            if self.is_excluded(name) {
                continue;
            }

            outputs.push(match schema {
                SchemaType::Enum(inner) => self.export_enum_type(name, inner)?,
                SchemaType::Struct(inner) => self.export_object_type(name, inner)?,
                _ => {
                    let out = self.render_schema_without_reference(schema)?;
                    self.export_type_alias(name, out)?
                }
            });
        }

        Ok(outputs.join("\n\n"))
    }
}
