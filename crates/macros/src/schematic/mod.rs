use crate::common::Macro;
use crate::utils::extract_comment;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub struct SchematicMacro<'l>(pub Macro<'l>);

impl<'l> ToTokens for SchematicMacro<'l> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let cfg = &self.0;
        let name = cfg.name;
        let schema = cfg.type_of.generate_schema(
            name,
            extract_comment(&cfg.attrs),
            cfg.get_casing_format(),
            cfg.get_tagged_format(),
        );

        tokens.extend(quote! {
            #[automatically_derived]
            impl schematic::Schematic for #name {
                fn generate_schema() -> schematic::SchemaType {
                    use schematic::schema::*;
                    #schema
                }
            }
        });
    }
}
