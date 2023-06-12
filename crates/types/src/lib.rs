mod arrays;
mod literals;
mod numbers;
mod objects;
mod strings;
mod structs;
mod tuples;
mod unions;

pub use arrays::*;
pub use literals::*;
pub use numbers::*;
pub use objects::*;
pub use strings::*;
pub use structs::*;
pub use tuples::*;
pub use unions::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum SchemaType {
    Boolean,
    Null,
    #[default]
    Unknown,
    Array(ArrayType),
    Float(FloatType),
    Integer(IntegerType),
    Literal(LiteralType),
    Object(ObjectType),
    Struct(StructType),
    String(StringType),
    Union(UnionType),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SchemaField {
    pub name: Option<String>,
    pub description: Option<String>,
    pub type_of: SchemaType,
    pub deprecated: bool,
    pub hidden: bool,
    pub nullable: bool,
    pub optional: bool,
    pub read_only: bool,
    pub write_only: bool,
}

pub trait Schematic {
    fn generate_schema() -> SchemaType {
        SchemaType::Unknown
    }
}

// CORE

impl<T: Schematic> Schematic for Box<T> {
    fn generate_schema() -> SchemaType {
        T::generate_schema()
    }
}

impl<T: Schematic> Schematic for Option<T> {
    fn generate_schema() -> SchemaType {
        SchemaType::Union(UnionType {
            variant_types: vec![Box::new(T::generate_schema()), Box::new(SchemaType::Null)],
            ..UnionType::default()
        })
    }
}
