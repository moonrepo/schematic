use crate::config::ConfigSchema;
use crate::schema::types::*;

impl<T: ConfigSchema> ConfigSchema for Option<T> {
    fn generate_schema() -> Schema {
        let mut schema = T::generate_schema();

        if let Schema::Type {
            ref mut nullable, ..
        } = schema
        {
            *nullable = true;
        }

        schema
    }
}
