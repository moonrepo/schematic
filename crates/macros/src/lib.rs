mod config;
mod config_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(Config, attributes(config, setting))]
pub fn config(item: TokenStream) -> TokenStream {
    config_macro::macro_impl(item)
}
