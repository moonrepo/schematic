mod arrays;
mod bools;
mod enums;
mod externals;
mod literals;
mod numbers;
mod objects;
mod schema;
mod schema_builder;
mod schema_type;
mod strings;
mod structs;
mod tuples;
mod unions;

pub use arrays::*;
pub use bools::*;
pub use enums::*;
pub use literals::*;
pub use numbers::*;
pub use objects::*;
pub use schema::*;
pub use schema_builder::*;
pub use schema_type::*;
pub use strings::*;
pub use structs::*;
pub use tuples::*;
pub use unions::*;

/// Defines a schema that represents the shape of the implementing type.
pub trait Schematic {
    /// Define a name for this schema type. Names are required for non-primitive values
    /// as a means to link references, and avoid cycles.
    fn schema_name() -> Option<String> {
        None
    }

    /// Create and return a schema that models the structure of the implementing type.
    /// The schema can be used to generate code, documentation, or other artifacts.
    fn build_schema(schema: SchemaBuilder) -> Schema {
        schema.build()
    }
}

// CORE

impl Schematic for () {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.custom(SchemaType::Null);
        schema.build()
    }
}

impl<T: Schematic> Schematic for &T {
    fn build_schema(schema: SchemaBuilder) -> Schema {
        T::build_schema(schema)
    }
}

impl<T: Schematic> Schematic for &mut T {
    fn build_schema(schema: SchemaBuilder) -> Schema {
        T::build_schema(schema)
    }
}

impl<T: Schematic> Schematic for Box<T> {
    fn build_schema(schema: SchemaBuilder) -> Schema {
        T::build_schema(schema)
    }
}

impl<T: Schematic> Schematic for Option<T> {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.custom(schema.infer_type::<T>());
        schema.nullable();
        schema.build()
    }
}
