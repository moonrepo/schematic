mod arrays;
mod bools;
mod enums;
mod externals;
mod literals;
mod numbers;
mod objects;
mod schema;
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
pub use schema_type::*;
pub use strings::*;
pub use structs::*;
pub use tuples::*;
pub use unions::*;

/// Defines a schema that represents the shape of the implementing type.
pub trait Schematic {
    /// Create and return a schema that models the structure of the implementing type.
    /// The schema can be used to generate code, documentation, or other artifacts.
    fn generate_schema() -> SchemaType {
        SchemaType::Unknown
    }
}

// CORE

impl Schematic for () {
    fn generate_schema() -> SchemaType {
        SchemaType::Null
    }
}

impl<T: Schematic> Schematic for &T {
    fn generate_schema() -> SchemaType {
        T::generate_schema()
    }
}

impl<T: Schematic> Schematic for &mut T {
    fn generate_schema() -> SchemaType {
        T::generate_schema()
    }
}

impl<T: Schematic> Schematic for Box<T> {
    fn generate_schema() -> SchemaType {
        T::generate_schema()
    }
}

impl<T: Schematic> Schematic for Option<T> {
    fn generate_schema() -> SchemaType {
        SchemaType::nullable(T::generate_schema())
    }
}
