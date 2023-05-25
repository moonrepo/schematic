mod config;
mod config_enum;
mod utils;

use proc_macro::TokenStream;

#[proc_macro_derive(Config, attributes(config, setting))]
pub fn config(item: TokenStream) -> TokenStream {
    config::macro_impl(item)
}

#[proc_macro_derive(ConfigEnum, attributes(config, variant))]
pub fn config_enum(item: TokenStream) -> TokenStream {
    config_enum::macro_impl(item)
}
