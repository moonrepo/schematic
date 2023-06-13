use crate::schema::{RenderResult, SchemaRenderer};
use schematic_types::*;
use std::collections::HashSet;

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
    pub const_enum: bool,
    pub enum_format: EnumFormat,
    pub object_format: ObjectFormat,
}

/// Renders TypeScript types from a schema.
#[derive(Default)]
pub struct TypeScriptRenderer {
    options: TypeScriptOptions,
    references: HashSet<String>,
}

impl TypeScriptRenderer {
    pub fn new(options: TypeScriptOptions) -> Self {
        Self {
            options,
            references: HashSet::new(),
        }
    }

    fn is_string_union_enum(&self, enu: &EnumType) -> bool {
        matches!(self.options.enum_format, EnumFormat::Union) || enu.variants.is_none()
    }

    fn export_type_alias(&self, name: &str, value: String) -> RenderResult {
        Ok(format!("export type {} = {};", name, value))
    }

    fn export_enum_type(&self, name: &str, enu: &EnumType) -> RenderResult {
        let value = self.render_enum(enu)?;

        if self.is_string_union_enum(enu) {
            return self.export_type_alias(name, value);
        }

        let out = format!("enum {} {{\n{}\n}}", name, value);

        Ok(if self.options.const_enum {
            format!("export const {}", out)
        } else {
            format!("export {}", out)
        })
    }

    fn export_object_type(&self, name: &str, structure: &StructType) -> RenderResult {
        let value = self.render_struct(structure)?;

        if matches!(self.options.object_format, ObjectFormat::Interface) {
            return Ok(format!("export interface {} {}", name, value));
        }

        self.export_type_alias(name, value)
    }

    fn render_enum_or_union(&self, enu: &EnumType) -> RenderResult {
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

        let mut fields = vec![];

        for variant in enu.variants.as_ref().unwrap() {
            if variant.hidden {
                continue;
            }

            if let Some(variant_name) = &variant.name {
                let mut field = if matches!(self.options.enum_format, EnumFormat::ValuedEnum) {
                    format!(
                        "\t{} = {},",
                        variant_name,
                        self.render_schema(&variant.type_of)?
                    )
                } else {
                    format!("\t{},", variant_name)
                };

                if let Some(comment) = &variant.description {
                    field = self.wrap_in_comment(comment.trim(), field);
                }

                fields.push(field);
            }
        }

        Ok(fields.join("\n"))
    }

    fn wrap_in_comment(&self, comment: &str, value: String) -> String {
        if comment.starts_with('*') {
            let mut out = vec!["\t/**".to_owned()];

            for line in comment.split('\n') {
                out.push(format!("\t {}", line.trim()));
            }

            out.push("\t */".to_owned());

            format!("{}\n{}", out.join("\n"), value)
        } else {
            format!("\t// {}\n{}", comment.trim(), value)
        }
    }
}

impl SchemaRenderer<String> for TypeScriptRenderer {
    fn is_reference(&self, name: &str) -> bool {
        self.references.contains(name)
    }

    fn render_array(&self, array: &ArrayType) -> RenderResult {
        let out = self.render_schema(&array.items_type)?;

        Ok(if out.contains('|') {
            format!("({})[]", out)
        } else {
            format!("{}[]", out)
        })
    }

    fn render_boolean(&self) -> RenderResult {
        Ok("boolean".into())
    }

    fn render_enum(&self, enu: &EnumType) -> RenderResult {
        self.render_enum_or_union(enu)
    }

    fn render_float(&self, _: &FloatType) -> RenderResult {
        Ok("number".into())
    }

    fn render_integer(&self, _: &IntegerType) -> RenderResult {
        Ok("number".into())
    }

    fn render_literal(&self, literal: &LiteralType) -> RenderResult {
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

    fn render_null(&self) -> RenderResult {
        Ok("null".into())
    }

    fn render_object(&self, object: &ObjectType) -> RenderResult {
        Ok(format!(
            "Record<{}, {}>",
            self.render_schema(&object.key_type)?,
            self.render_schema(&object.value_type)?
        ))
    }

    fn render_reference(&self, reference: &str) -> RenderResult {
        Ok(reference.into())
    }

    fn render_string(&self, _: &StringType) -> RenderResult {
        Ok("string".into())
    }

    fn render_struct(&self, structure: &StructType) -> RenderResult {
        let mut out = vec![];

        for field in &structure.fields {
            if field.hidden {
                continue;
            }

            let mut row = format!("\t{}", field.name.as_ref().unwrap());

            if field.optional {
                row.push_str("?: ");
            } else {
                row.push_str(": ");
            }

            row.push_str(&self.render_schema(&field.type_of)?);

            if field.nullable && !row.contains(" null") {
                row.push_str(" | null");
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

        Ok(format!("{{\n{}\n}}", out.join("\n")))
    }

    fn render_tuple(&self, tuple: &TupleType) -> RenderResult {
        let mut items = vec![];

        for item in &tuple.items_types {
            items.push(self.render_schema(item)?);
        }

        Ok(format!("[{}]", items.join(", ")))
    }

    fn render_union(&self, uni: &UnionType) -> RenderResult {
        let mut items = vec![];

        for item in &uni.variants_types {
            items.push(self.render_schema(item)?);
        }

        Ok(items.join(" | "))
    }

    fn render_unknown(&self) -> RenderResult {
        Ok("unknown".into())
    }

    fn render(&mut self, schemas: &[SchemaType], references: &HashSet<String>) -> RenderResult {
        self.references.extend(references.to_owned());

        let mut outputs = vec![
            "// Automatically generated by schematic. DO NOT MODIFY!".to_string(),
            "/* eslint-disable */".to_string(),
        ];

        for schema in schemas {
            if let Some(name) = schema.get_name() {
                outputs.push(match schema {
                    SchemaType::Enum(inner) => self.export_enum_type(name, inner)?,
                    SchemaType::Struct(inner) => self.export_object_type(name, inner)?,
                    _ => {
                        self.export_type_alias(name, self.render_schema_without_reference(schema)?)?
                    }
                });
            }
        }

        Ok(outputs.join("\n\n"))
    }
}
