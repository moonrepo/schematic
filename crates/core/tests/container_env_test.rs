mod utils;

use schematic_core::container::Container;
use starbase_sandbox::assert_snapshot;
use syn::parse_quote;
use utils::pretty;

mod container_env {
    use super::*;

    #[test]
    fn can_set_vars() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                no_var: String,
                no_var_nested: NestedConfig,
                #[setting(env = "STR")]
                a: String,
                #[setting(env = "BOOL")]
                b: bool,
                #[setting(env = "INT")]
                c: usize,
                #[setting(nested)]
                d: NestedConfig,
                #[setting(nested = CustomConfig)]
                e: CustomConfig,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_env_values()));
    }

    #[test]
    fn can_set_prefix() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            #[config(env_prefix = "PREFIX_")]
            struct Example {
                #[setting(env = "OVERRIDE")]
                a: String,
                b: bool,
                c: usize,
                #[setting(nested)]
                d: NestedConfig,
                #[setting(nested = CustomConfig)]
                e: CustomConfig,
                #[setting(nested, env_prefix = "NESTED_")]
                f: NestedConfig,
                #[setting(nested = CustomConfig, env_prefix = "NESTED_")]
                g: CustomConfig,
            }
        });

        assert_snapshot!(pretty(container.impl_partial_env_values()));
    }
}
