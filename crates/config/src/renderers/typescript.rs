use crate::schema::SchemaRenderer;
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
    pub enum_format: EnumFormat,
    pub exclude_partial: bool,
    pub object_format: ObjectFormat,
}

#[derive(Default)]
pub struct TypeScriptRenderer {
    options: TypeScriptOptions,
}

impl TypeScriptRenderer {
    pub fn new(options: TypeScriptOptions) -> Self {
        Self { options }
    }
}

impl SchemaRenderer for TypeScriptRenderer {
    fn render(
        &self,
        schemas: &[SchemaType],
        references: &HashSet<String>,
    ) -> miette::Result<String> {
        let mut outputs = vec![
            "// Automatically generated by schematic. DO NOT MODIFY!".to_string(),
            "/* eslint-disable */".to_string(),
        ];

        Ok(outputs.join("\n\n"))
    }
}
