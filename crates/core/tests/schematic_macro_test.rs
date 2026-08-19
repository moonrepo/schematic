mod utils;

use quote::ToTokens;
use schematic_core::container::Container;
use starbase_sandbox::assert_snapshot;
use syn::parse_quote;
use utils::pretty;

// The standalone derive is the same container with the flag flipped
fn schematic_only(input: syn::DeriveInput) -> Container {
    let mut container = Container::from(input);
    container.schematic_only = true;
    container
}

// The standalone derive emits a single `Schematic` impl, with no partial
// type alongside it. That is the whole difference from `Config`.
mod standalone {
    use super::*;

    #[test]
    fn named_struct() {
        let output = schematic_only(parse_quote! {
            /// Container docs.
            #[derive(Schematic)]
            struct Example {
                a: bool,
                /// Field docs.
                b: Vec<String>,
            }
        });

        assert_snapshot!(pretty(output.to_token_stream()));
    }

    #[test]
    fn unnamed_struct() {
        let output = schematic_only(parse_quote! {
            #[derive(Schematic)]
            struct Example(bool, usize);
        });

        assert_snapshot!(pretty(output.to_token_stream()));
    }

    #[test]
    fn unnamed_enum() {
        let output = schematic_only(parse_quote! {
            #[derive(Schematic)]
            enum Example {
                A(bool),
                B(String),
            }
        });

        assert_snapshot!(pretty(output.to_token_stream()));
    }

    #[test]
    fn unit_enum() {
        let output = schematic_only(parse_quote! {
            #[derive(Schematic)]
            enum Example {
                A,
                B,
            }
        });

        assert_snapshot!(pretty(output.to_token_stream()));
    }

    #[test]
    fn supports_renames() {
        let output = schematic_only(parse_quote! {
            #[derive(Schematic)]
            #[schematic(rename = "Renamed")]
            struct Example {
                a: bool,
            }
        });

        assert_snapshot!(pretty(output.to_token_stream()));
    }
}

// A generic type cannot derive at all unless the parameters are carried
// through to the impl.
mod generics {
    use super::*;

    #[test]
    fn supports_type_parameters() {
        let output = schematic_only(parse_quote! {
            #[derive(Schematic)]
            struct Example<T> {
                inner: T,
            }
        });

        assert_snapshot!(pretty(output.to_token_stream()));
    }

    #[test]
    fn supports_bounds() {
        let output = schematic_only(parse_quote! {
            #[derive(Schematic)]
            struct Example<T: Schematic + Clone> {
                inner: T,
            }
        });

        assert_snapshot!(pretty(output.to_token_stream()));
    }

    #[test]
    fn supports_where_clauses() {
        let output = schematic_only(parse_quote! {
            #[derive(Schematic)]
            struct Example<T>
            where
                T: Schematic,
            {
                inner: T,
            }
        });

        assert_snapshot!(pretty(output.to_token_stream()));
    }

    #[test]
    fn supports_lifetimes_and_defaults() {
        let output = schematic_only(parse_quote! {
            #[derive(Schematic)]
            struct Example<'a, T = bool> {
                inner: T,
                name: &'a str,
            }
        });

        assert_snapshot!(pretty(output.to_token_stream()));
    }
}
