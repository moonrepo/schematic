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

use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;

/// Defines a schema that represents the shape of the implementing type.
pub trait Schematic {
    /// Define a name for this schema type. Names are required for non-primitive values
    /// as a means to link references, and avoid cycles.
    fn schema_name() -> Option<String> {
        None
    }

    /// Create and return a schema that models the structure of the implementing type.
    /// The schema can be used to generate code, documentation, or other artifacts.
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.build()
    }
}

/// Resolve the schema name of a type, for composing the name of a generic
/// type from its arguments.
///
/// Types that define a `schema_name` use it as-is. Anything else falls back
/// to a cleaned up form of its Rust type name, so that instantiations over
/// unnamed types (primitives, collections) still resolve to distinct names
/// instead of silently colliding.
pub fn schema_name_of<T: Schematic + ?Sized>() -> String {
    if let Some(name) = T::schema_name() {
        return name;
    }

    let mut name = String::new();

    // `alloc::vec::Vec<alloc::string::String>` becomes `VecString`
    for chunk in
        std::any::type_name::<T>().split(|c: char| !(c.is_alphanumeric() || c == '_' || c == ':'))
    {
        let Some(segment) = chunk.rsplit("::").next() else {
            continue;
        };

        let mut chars = segment.chars();

        if let Some(first) = chars.next() {
            name.extend(first.to_uppercase());
            name.push_str(chars.as_str());
        }
    }

    name
}

// CORE

impl Schematic for () {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.set_type_and_build(SchemaType::Null)
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

impl<T: Schematic + ?Sized> Schematic for Box<T> {
    fn build_schema(schema: SchemaBuilder) -> Schema {
        T::build_schema(schema)
    }
}

impl<T: Schematic + ?Sized> Schematic for Rc<T> {
    fn build_schema(schema: SchemaBuilder) -> Schema {
        T::build_schema(schema)
    }
}

impl<T: Schematic + ?Sized> Schematic for Arc<T> {
    fn build_schema(schema: SchemaBuilder) -> Schema {
        T::build_schema(schema)
    }
}

impl<T: Schematic + ToOwned + ?Sized> Schematic for Cow<'_, T> {
    fn build_schema(schema: SchemaBuilder) -> Schema {
        T::build_schema(schema)
    }
}

impl<T: Schematic> Schematic for Option<T> {
    fn build_schema(mut schema: SchemaBuilder) -> Schema {
        schema.nullable(schema.infer::<T>())
    }
}
