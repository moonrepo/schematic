mod config;
mod utils;

use proc_macro::TokenStream;

#[proc_macro_derive(Config, attributes(config, setting))]
pub fn config(item: TokenStream) -> TokenStream {
    config::macro_impl(item)
}
