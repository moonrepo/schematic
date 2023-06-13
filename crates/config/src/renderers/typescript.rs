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
    pub exclude_partial: bool,
    pub object_format: ObjectFormat,
}

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

    pub fn export_type_alias(&self, name: &str, value: &str) -> RenderResult {
        Ok(format!("export type {} = {};", name, value))
    }

    pub fn export_enum_type(&self, name: &str, uni: &UnionType) -> RenderResult {
        if matches!(
            self.options.enum_format,
            EnumFormat::Enum | EnumFormat::ValuedEnum
        ) {
            let Some(variants) = &uni.variants else {
                return self.export_type_alias(name, &self.render_union(uni)?);
            };

            let mut fields = vec![];

            for variant in variants {
                if variant.hidden {
                    continue;
                }

                if let Some(variant_name) = &variant.name {
                    if matches!(self.options.enum_format, EnumFormat::ValuedEnum) {
                        fields.push(format!(
                            "\t{} = {},",
                            variant_name,
                            self.render_schema(&variant.type_of)?
                        ));
                    } else {
                        fields.push(format!("\t{},", variant_name));
                    }
                }
            }

            let out = format!("enum {} {{\n{}\n}}", name, fields.join("\n"));

            return Ok(if self.options.const_enum {
                format!("export const {}", out)
            } else {
                format!("export {}", out)
            });
        }

        self.export_type_alias(name, &self.render_union(uni)?)
    }

    pub fn export_object_type(&self, name: &str, structure: &StructType) -> RenderResult {
        let value = self.render_struct(structure)?;

        if matches!(self.options.object_format, ObjectFormat::Interface) {
            return Ok(format!("export interface {} {}", name, value));
        }

        self.export_type_alias(name, &value)
    }
}

impl SchemaRenderer for TypeScriptRenderer {
    fn render_array(&self, array: &ArrayType) -> RenderResult {
        Ok(format!("{}[]", self.render_schema(&array.items_type)?))
    }

    fn render_boolean(&self) -> RenderResult {
        Ok("boolean".into())
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
                row.push_str(";");
            } else {
                row.push_str(",");
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

    fn render_schema(&self, schema: &SchemaType) -> RenderResult {
        if let Some(name) = schema.get_name() {
            if self.references.contains(name) {
                return Ok(name.clone());
            }
        }

        match schema {
            SchemaType::Boolean => self.render_boolean(),
            SchemaType::Null => self.render_null(),
            SchemaType::Unknown => self.render_unknown(),
            SchemaType::Array(array) => self.render_array(array),
            SchemaType::Float(float) => self.render_float(float),
            SchemaType::Integer(integer) => self.render_integer(integer),
            SchemaType::Literal(literal) => self.render_literal(literal),
            SchemaType::Object(object) => self.render_object(object),
            SchemaType::Struct(structure) => self.render_struct(structure),
            SchemaType::String(string) => self.render_string(string),
            SchemaType::Tuple(tuple) => self.render_tuple(tuple),
            SchemaType::Union(uni) => self.render_union(uni),
        }
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
                    SchemaType::Struct(inner) => self.export_object_type(name, inner)?,
                    SchemaType::Union(inner) => self.export_enum_type(name, inner)?,
                    _ => self.export_type_alias(name, &self.render_schema(schema)?)?,
                });
            }
        }

        Ok(outputs.join("\n\n"))
    }
}
