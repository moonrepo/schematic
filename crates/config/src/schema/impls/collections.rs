// use crate::config::ConfigSchema;
// use crate::schema::types::*;

// impl<T: ConfigSchema> ConfigSchema for Vec<T> {
//     fn generate_schema() -> Schema {
//         let inner = T::generate_schema();

//         if let Schema::Type {
//             ref mut nullable, ..
//         } = schema
//         {
//             *nullable = true;
//         }

//         schema
//     }
// }
