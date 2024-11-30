use crate::common::Macro;
use crate::utils::instrument_quote;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub struct SchematicMacro<'l>(pub Macro<'l>);

impl ToTokens for SchematicMacro<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let cfg = &self.0;
        let name = cfg.name;

        let schema_name = cfg.get_name();
        let schema_impl = cfg.type_of.generate_schema(&cfg.attrs);
        let instrument = instrument_quote();

        tokens.extend(quote! {
            #[automatically_derived]
            impl schematic::Schematic for #name {
                fn schema_name() -> Option<String> {
                    Some(#schema_name.into())
                }

                #instrument
                fn build_schema(mut schema: schematic::SchemaBuilder) -> schematic::Schema {
                    use schematic::schema::*;

                    #schema_impl
                }
            }
        });
    }
}
