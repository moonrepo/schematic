use proc_macro2::TokenStream;

pub fn pretty(tokens: TokenStream) -> String {
    prettyplease::unparse(&syn::parse_file(&tokens.to_string()).unwrap())
}
