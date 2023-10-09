mod common;
mod common_schema;
mod config;
mod config_enum;
mod utils;

#[cfg(feature = "schema")]
mod schematic;

use proc_macro::TokenStream;

// #[derive(Config)]
#[proc_macro_derive(Config, attributes(config, setting))]
pub fn config(item: TokenStream) -> TokenStream {
    config::macro_impl(item)
}

// #[derive(ConfigEnum)]
#[proc_macro_derive(ConfigEnum, attributes(config, variant))]
pub fn config_enum(item: TokenStream) -> TokenStream {
    config_enum::macro_impl(item)
}

// // #[derive(Schematic)]
// #[cfg(feature = "schema")]
// #[proc_macro_derive(Schematic, attributes(schematic))]
// pub fn schematic(item: TokenStream) -> TokenStream {
//     schematic::macro_impl(item)
// }
