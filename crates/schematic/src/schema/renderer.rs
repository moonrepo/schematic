use indexmap::IndexMap;
use schematic_types::*;

pub type RenderResult<T = String> = miette::Result<T>;

/// The error of a renderer that doesn't implement rendering a type.
fn unsupported<R: ?Sized, O>(kind: &str) -> RenderResult<O> {
    Err(miette::miette!(
        "{} does not support rendering {kind}.",
        std::any::type_name::<R>()
    ))
}

/// Renders [`SchemaType`]s to a distinct format (derived from generic `O`)
/// for use within a [`SchemaGenerator`](super::SchemaGenerator).
///
/// Every `render_*` method defaults to an error that names the type, so a
/// renderer only needs to implement the types it supports, along with
/// [`is_reference`](Self::is_reference) and [`render`](Self::render).
pub trait SchemaRenderer<O = String> {
    /// Return true of the provided name is a referenced type.
    fn is_reference(&self, name: &str) -> bool;

    /// Render an [`ArrayType`] to a string.
    fn render_array(&mut self, _array: &ArrayType, _schema: &Schema) -> RenderResult<O> {
        unsupported::<Self, O>("an array")
    }

    /// Render a boolean type to a string.
    fn render_boolean(&mut self, _boolean: &BooleanType, _schema: &Schema) -> RenderResult<O> {
        unsupported::<Self, O>("a boolean")
    }

    /// Render an [`EnumType`] to a string.
    fn render_enum(&mut self, _enu: &EnumType, _schema: &Schema) -> RenderResult<O> {
        unsupported::<Self, O>("an enum")
    }

    /// Render a [`FloatType`] to a string.
    fn render_float(&mut self, _float: &FloatType, _schema: &Schema) -> RenderResult<O> {
        unsupported::<Self, O>("a float")
    }

    /// Render an [`IntegerType`] to a string.
    fn render_integer(&mut self, _integer: &IntegerType, _schema: &Schema) -> RenderResult<O> {
        unsupported::<Self, O>("an integer")
    }

    /// Render a [`LiteralType`] to a string.
    fn render_literal(&mut self, _literal: &LiteralType, _schema: &Schema) -> RenderResult<O> {
        unsupported::<Self, O>("a literal")
    }

    /// Render a null type to a string.
    fn render_null(&mut self, _schema: &Schema) -> RenderResult<O> {
        unsupported::<Self, O>("a null")
    }

    /// Render an [`ObjectType`] to a string.
    fn render_object(&mut self, _object: &ObjectType, _schema: &Schema) -> RenderResult<O> {
        unsupported::<Self, O>("an object")
    }

    /// Render a referenced type to a string.
    fn render_reference(&mut self, _reference: &str, _schema: &Schema) -> RenderResult<O> {
        unsupported::<Self, O>("a reference")
    }

    /// Render a [`StringType`] to a string.
    fn render_string(&mut self, _string: &StringType, _schema: &Schema) -> RenderResult<O> {
        unsupported::<Self, O>("a string")
    }

    /// Render a [`StructType`] to a string.
    fn render_struct(&mut self, _structure: &StructType, _schema: &Schema) -> RenderResult<O> {
        unsupported::<Self, O>("a struct")
    }

    /// Render a [`TaggedType`], an enum variant that serde writes with its
    /// name, to a string. Defaults to rendering the struct serde writes for it
    /// ([`TaggedType::to_schema`]), so a renderer only implements this to
    /// shape each tag format itself.
    fn render_tagged(&mut self, _tagged: &TaggedType, schema: &Schema) -> RenderResult<O> {
        let mut expanded = schema.clone();
        expanded.expand_tagged();

        self.render_schema_without_reference(&expanded)
    }

    /// Render a [`TupleType`] to a string.
    fn render_tuple(&mut self, _tuple: &TupleType, _schema: &Schema) -> RenderResult<O> {
        unsupported::<Self, O>("a tuple")
    }

    /// Render a [`UnionType`] to a string.
    fn render_union(&mut self, _uni: &UnionType, _schema: &Schema) -> RenderResult<O> {
        unsupported::<Self, O>("a union")
    }

    /// Render an unknown type to a string.
    fn render_unknown(&mut self, _schema: &Schema) -> RenderResult<O> {
        unsupported::<Self, O>("an unknown type")
    }

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

    /// Like [`Self::render_schema`] but does not check for references.
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
            SchemaType::Tagged(tagged) => self.render_tagged(tagged, schema),
            SchemaType::Tuple(tuple) => self.render_tuple(tuple, schema),
            SchemaType::Union(uni) => self.render_union(uni, schema),
            SchemaType::Reference { name, .. } => self.render_reference(name, schema),
        }
    }

    /// Render the list of [`Schema`]s to a string, in the order they are listed.
    /// References between types can be resolved using the provided `schemas` map.
    fn render(&mut self, schemas: IndexMap<String, Schema>) -> RenderResult;
}
