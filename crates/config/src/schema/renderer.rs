use indexmap::IndexMap;
use schematic_types::*;
use std::collections::HashSet;

pub type RenderResult<T = String> = miette::Result<T>;

/// Renders [`SchemaType`]s to a distinct format (derived from generic `O`)
/// for use within a [`SchemaGenerator`].
pub trait SchemaRenderer<O = String> {
    /// Return true of the provided name is a referenced type.
    fn is_reference(&self, name: &str) -> bool;

    /// Render an [`ArrayType`] to a string.
    fn render_array(&mut self, array: &ArrayType) -> RenderResult<O>;

    /// Render a boolean type to a string.
    fn render_boolean(&mut self) -> RenderResult<O>;

    /// Render an [`EnumType`] to a string.
    fn render_enum(&mut self, enu: &EnumType) -> RenderResult<O>;

    /// Render a [`FloatType`] to a string.
    fn render_float(&mut self, float: &FloatType) -> RenderResult<O>;

    /// Render an [`IntegerType`] to a string.
    fn render_integer(&mut self, integer: &IntegerType) -> RenderResult<O>;

    /// Render a [`LiteralType`] to a string.
    fn render_literal(&mut self, literal: &LiteralType) -> RenderResult<O>;

    /// Render a null type to a string.
    fn render_null(&mut self) -> RenderResult<O>;

    /// Render an [`ObjectType`] to a string.
    fn render_object(&mut self, object: &ObjectType) -> RenderResult<O>;

    /// Render a referenced type to a string.
    fn render_reference(&mut self, reference: &str) -> RenderResult<O>;

    /// Render a [`StringType`] to a string.
    fn render_string(&mut self, string: &StringType) -> RenderResult<O>;

    /// Render a [`StructType`] to a string.
    fn render_struct(&mut self, structure: &StructType) -> RenderResult<O>;

    /// Render a [`TupleType`] to a string.
    fn render_tuple(&mut self, tuple: &TupleType) -> RenderResult<O>;

    /// Render a [`UnionType`] to a string.
    fn render_union(&mut self, uni: &UnionType) -> RenderResult<O>;

    /// Render an unknown type to a string.
    fn render_unknown(&mut self) -> RenderResult<O>;

    /// Render all possible variants of the provided [`SchemaType`] to a string.
    /// If a variant has an explicit name, and that name is a reference, return
    /// the name instead of rendering the type.
    fn render_schema(&mut self, schema: &SchemaType) -> RenderResult<O> {
        if let Some(name) = schema.get_name() {
            if self.is_reference(name) {
                return self.render_reference(name);
            }
        }

        self.render_schema_without_reference(schema)
    }

    /// Like [`render_schema`] but does not check for references.
    fn render_schema_without_reference(&mut self, schema: &SchemaType) -> RenderResult<O> {
        match schema {
            SchemaType::Boolean => self.render_boolean(),
            SchemaType::Null => self.render_null(),
            SchemaType::Unknown => self.render_unknown(),
            SchemaType::Array(array) => self.render_array(array),
            SchemaType::Enum(enu) => self.render_enum(enu),
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

    /// Render the list of [`SchemaType`]s to a string, in the order they are listed.
    /// References between types can be resolved using the provided `references` set.
    fn render(
        &mut self,
        schemas: &IndexMap<String, SchemaType>,
        references: &HashSet<String>,
    ) -> RenderResult;
}
