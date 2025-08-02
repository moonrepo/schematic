use proc_macro2::TokenStream;
// use schematic_core::container::Container;
// use schematic_core::field::Field;

pub fn pretty(tokens: TokenStream) -> String {
    prettyplease::unparse(&syn::parse_file(&tokens.to_string()).unwrap())
}

// pub fn get_field<'a>(container: &'a Container, key: &str) -> &'a Field {
//     container
//         .inner
//         .get_fields()
//         .into_iter()
//         .find(|field| field.ident.as_ref().is_some_and(|id| id == key))
//         .unwrap()
// }
