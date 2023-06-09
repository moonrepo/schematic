use super::types::*;
use crate::config::ConfigSchema;

impl schematic::ConfigSchema for String {
    fn generate_schema() -> Schema {
        Schema::Type {
            name: "String".into(),
            type_of: Type::String,
        }
    }
}
