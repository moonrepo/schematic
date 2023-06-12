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
}
