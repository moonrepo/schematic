use schematic_types::*;
use std::collections::HashSet;

pub type RenderResult = miette::Result<String>;

pub trait SchemaRenderer {
    fn render_array(&self, array: &ArrayType) -> RenderResult;

    fn render_boolean(&self) -> RenderResult;

    fn render_float(&self, float: &FloatType) -> RenderResult;

    fn render_integer(&self, integer: &IntegerType) -> RenderResult;

    fn render_literal(&self, literal: &LiteralType) -> RenderResult;

    fn render_null(&self) -> RenderResult;

    fn render_object(&self, object: &ObjectType) -> RenderResult;

    fn render_string(&self, string: &StringType) -> RenderResult;

    fn render_struct(&self, structure: &StructType) -> RenderResult;

    fn render_tuple(&self, tuple: &TupleType) -> RenderResult;

    fn render_union(&self, uni: &UnionType) -> RenderResult;

    fn render_unknown(&self) -> RenderResult;

    fn render_schema(&self, schema: &SchemaType) -> RenderResult;

    fn render(&mut self, schemas: &[SchemaType], references: &HashSet<String>) -> RenderResult;
}
