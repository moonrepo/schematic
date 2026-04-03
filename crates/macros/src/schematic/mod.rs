use crate::common::Macro;
use crate::utils::instrument_quote;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

pub struct SchematicMacro<'l>(pub Macro<'l>);

impl ToTokens for SchematicMacro<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let cfg = &self.0;
        let name = cfg.name;

        let schema_name = cfg.get_name();
        let schema_impl = cfg.type_of.generate_schema(&cfg.attrs);
        let instrument = instrument_quote();
        let (impl_generics, ty_generics, where_clause) = cfg.generics.split_for_impl();

        tokens.extend(quote! {
            #[automatically_derived]
            impl #impl_generics schematic::Schematic for #name #ty_generics #where_clause {
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
