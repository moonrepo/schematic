use indexmap::IndexMap;
use schematic_types::*;
use std::collections::HashSet;

pub type RenderResult<T = String> = miette::Result<T>;

/// Renders [`SchemaType`]s to a distinct format (derived from generic `O`)
/// for use within a [`SchemaGenerator`].
pub trait SchemaRenderer<'gen, O = String> {
    /// Return true of the provided name is a referenced type.
    fn is_reference(&self, name: &str) -> bool;

    /// Render an [`ArrayType`] to a string.
    fn render_array(&mut self, array: &ArrayType, schema: &Schema) -> RenderResult<O>;

    /// Render a boolean type to a string.
    fn render_boolean(&mut self, boolean: &BooleanType, schema: &Schema) -> RenderResult<O>;

    /// Render an [`EnumType`] to a string.
    fn render_enum(&mut self, enu: &EnumType, schema: &Schema) -> RenderResult<O>;

    /// Render a [`FloatType`] to a string.
    fn render_float(&mut self, float: &FloatType, schema: &Schema) -> RenderResult<O>;

    /// Render an [`IntegerType`] to a string.
    fn render_integer(&mut self, integer: &IntegerType, schema: &Schema) -> RenderResult<O>;

    /// Render a [`LiteralType`] to a string.
    fn render_literal(&mut self, literal: &LiteralType, schema: &Schema) -> RenderResult<O>;

    /// Render a null type to a string.
    fn render_null(&mut self, schema: &Schema) -> RenderResult<O>;

    /// Render an [`ObjectType`] to a string.
    fn render_object(&mut self, object: &ObjectType, schema: &Schema) -> RenderResult<O>;

    /// Render a referenced type to a string.
    fn render_reference(&mut self, reference: &str, schema: &Schema) -> RenderResult<O>;

    /// Render a [`StringType`] to a string.
    fn render_string(&mut self, string: &StringType, schema: &Schema) -> RenderResult<O>;

    /// Render a [`StructType`] to a string.
    fn render_struct(&mut self, structure: &StructType, schema: &Schema) -> RenderResult<O>;

    /// Render a [`TupleType`] to a string.
    fn render_tuple(&mut self, tuple: &TupleType, schema: &Schema) -> RenderResult<O>;

    /// Render a [`UnionType`] to a string.
    fn render_union(&mut self, uni: &UnionType, schema: &Schema) -> RenderResult<O>;

    /// Render an unknown type to a string.
    fn render_unknown(&mut self, schema: &Schema) -> RenderResult<O>;

    /// Render all possible variants of the provided [`Schema`] to a string.
    /// If a variant has an explicit name, and that name is a reference, return
    /// the name instead of rendering the type.
    fn render_schema(&mut self, schema: &Schema) -> RenderResult<O> {
        if let Some(name) = &schema.name {
            if self.is_reference(name) {
                return self.render_reference(name, schema);
            }
        }

        self.render_schema_without_reference(schema)
    }

    /// Like [`render_schema`] but does not check for references.
    fn render_schema_without_reference(&mut self, schema: &Schema) -> RenderResult<O> {
        match &schema.ty {
            SchemaType::Null => self.render_null(schema),
            SchemaType::Unknown => self.render_unknown(schema),
            SchemaType::Array(array) => self.render_array(array, schema),
            SchemaType::Boolean(boolean) => self.render_boolean(boolean, schema),
            SchemaType::Enum(enu) => self.render_enum(enu, schema),
            SchemaType::Float(float) => self.render_float(float, schema),
            SchemaType::Integer(integer) => self.render_integer(integer, schema),
            SchemaType::Literal(literal) => self.render_literal(literal, schema),
            SchemaType::Object(object) => self.render_object(object, schema),
            SchemaType::Struct(structure) => self.render_struct(structure, schema),
            SchemaType::String(string) => self.render_string(string, schema),
            SchemaType::Tuple(tuple) => self.render_tuple(tuple, schema),
            SchemaType::Union(uni) => self.render_union(uni, schema),
            SchemaType::Reference(name) => self.render_reference(name, schema),
        }
    }

    /// Render the list of [`Schema`]s to a string, in the order they are listed.
    /// References between types can be resolved using the provided `references` set.
    fn render(
        &mut self,
        schemas: &'gen IndexMap<String, Schema>,
        references: &'gen HashSet<String>,
    ) -> RenderResult;
}
