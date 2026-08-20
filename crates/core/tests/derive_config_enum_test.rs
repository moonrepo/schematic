mod utils;

use quote::ToTokens;
use schematic_core::container::{Container, ContainerMacro};
use starbase_sandbox::assert_snapshot;
use syn::parse_quote;
use utils::pretty;

// The same container as `Config`, rendering a different set of impls
fn config_enum(input: syn::DeriveInput) -> Container {
    let mut container = Container::from(input);
    container.macro_type = ContainerMacro::ConfigUnitEnum;
    container
}

mod rendering {
    use super::*;

    #[test]
    fn unit_enum() {
        let output = config_enum(parse_quote! {
            /// Container docs.
            #[derive(ConfigEnum)]
            enum Example {
                Info,
                Error,
                Off,
            }
        });

        assert_snapshot!(pretty(output.to_token_stream()));
    }

    #[test]
    fn supports_renames_and_aliases() {
        let output = config_enum(parse_quote! {
            #[derive(ConfigEnum)]
            #[config(rename_all = "kebab-case")]
            enum Example {
                VeryHigh,
                #[variant(rename = "custom")]
                Explicit,
                #[variant(alias = "second", alias = "third")]
                Aliased,
            }
        });

        assert_snapshot!(pretty(output.to_token_stream()));
    }

    #[test]
    fn supports_before_parse() {
        let output = config_enum(parse_quote! {
            #[derive(ConfigEnum)]
            #[config(before_parse = "lowercase")]
            enum Example {
                Info,
            }
        });

        assert_snapshot!(pretty(output.to_token_stream()));
    }

    #[test]
    fn supports_a_fallback() {
        let output = config_enum(parse_quote! {
            #[derive(ConfigEnum)]
            enum Example {
                Known,
                #[variant(fallback)]
                Other(String),
            }
        });

        assert_snapshot!(pretty(output.to_token_stream()));
    }

    #[test]
    fn supports_a_default() {
        let output = config_enum(parse_quote! {
            #[derive(ConfigEnum, Default)]
            enum Example {
                #[default]
                Info,
                Error,
            }
        });

        assert_snapshot!(pretty(output.to_token_stream()));
    }
}

mod validation {
    use super::*;

    fn render(input: syn::DeriveInput) -> String {
        pretty(config_enum(input).to_token_stream())
    }

    #[test]
    #[should_panic(expected = "Only unit variants are supported, unless marked as `fallback`")]
    fn rejects_a_non_unit_variant() {
        render(parse_quote! {
            #[derive(ConfigEnum)]
            enum Example {
                Known,
                Invalid(String),
            }
        });
    }

    #[test]
    #[should_panic(expected = "Only 1 unnamed field is supported for `fallback`")]
    fn rejects_a_multi_field_fallback() {
        render(parse_quote! {
            #[derive(ConfigEnum)]
            enum Example {
                Known,
                #[variant(fallback)]
                Other(String, usize),
            }
        });
    }

    #[test]
    #[should_panic(expected = "Only unnamed tuple variants are supported for `fallback`")]
    fn rejects_a_unit_fallback() {
        render(parse_quote! {
            #[derive(ConfigEnum)]
            enum Example {
                Known,
                #[variant(fallback)]
                Other,
            }
        });
    }

    #[test]
    #[should_panic(expected = "Only 1 fallback variant is supported")]
    fn rejects_multiple_fallbacks() {
        render(parse_quote! {
            #[derive(ConfigEnum)]
            enum Example {
                #[variant(fallback)]
                One(String),
                #[variant(fallback)]
                Two(String),
            }
        });
    }

    #[test]
    #[should_panic(expected = "Unknown `before_parse` value `nope`. Supported values are")]
    fn rejects_an_unknown_before_parse() {
        render(parse_quote! {
            #[derive(ConfigEnum)]
            #[config(before_parse = "nope")]
            enum Example {
                Info,
            }
        });
    }

    #[test]
    #[should_panic(expected = "Only enums are supported")]
    fn rejects_a_struct() {
        render(parse_quote! {
            #[derive(ConfigEnum)]
            struct Example {
                a: bool,
            }
        });
    }
}
