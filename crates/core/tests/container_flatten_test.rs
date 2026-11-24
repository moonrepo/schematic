mod utils;

use schematic_core::container::Container;
use starbase_sandbox::assert_snapshot;
use syn::parse_quote;
use utils::pretty;

// Serde deserializes a struct with a flattened setting as a map, which reads a
// parent's keys without taking them, so the partial says so for its parent's
// check, and whether it accepts unknown fields
#[test]
fn marks_a_partial_that_flattens() {
    let container = Container::from(parse_quote! {
        #[derive(Config)]
        #[config(allow_unknown_fields)]
        struct Example {
            a: bool,
            #[setting(flatten, nested)]
            child: Child,
        }
    });

    assert_snapshot!(pretty(container.impl_partial()));
}

#[test]
fn marks_nothing_without_flatten() {
    let container = Container::from(parse_quote! {
        #[derive(Config)]
        struct Example {
            a: bool,
        }
    });

    let output = pretty(container.impl_partial());

    assert!(!output.contains("FLATTENS"), "{output}");
    assert!(!output.contains("ALLOWS_UNKNOWN_FIELDS"), "{output}");
}

#[test]
fn asserts_a_flattened_config_can_be_loaded() {
    let container = Container::from(parse_quote! {
        #[derive(Config)]
        struct Example {
            #[setting(flatten, nested)]
            child: Child,
            #[setting(flatten, nested)]
            optional: Option<child::Other>,
        }
    });

    assert_snapshot!(pretty(container.impl_partial_checks()));
}

// Serde discards the error of a flattened `Option`, as a flattened type may be
// absent, but a map is only ever empty, so its partial keeps the error, unless
// `partial(serde(...))` deserializes it already
#[test]
fn keeps_the_errors_of_a_flattened_map() {
    let container = Container::from(parse_quote! {
        #[derive(Config)]
        #[config(allow_unknown_fields)]
        struct Example {
            #[setting(flatten)]
            values: HashMap<String, String>,
            #[setting(flatten, nested)]
            plugins: Option<BTreeMap<String, Plugin>>,
            #[setting(flatten, nested)]
            child: Child,
            #[setting(flatten, partial(serde(deserialize_with = "custom")))]
            custom: HashMap<String, String>,
        }
    });

    assert_snapshot!(pretty(container.impl_partial_type()));
}

// A flattened collection of configs is a map, which the config's items don't
// read keys from, and a generic config's constant couldn't name its parameters
#[test]
fn skips_a_flattened_collection_or_generic_config() {
    let collection = Container::from(parse_quote! {
        #[derive(Config)]
        struct Example {
            #[setting(flatten, nested)]
            plugins: HashMap<String, Plugin>,
        }
    });

    assert!(collection.impl_partial_checks().is_empty());

    let generic = Container::from(parse_quote! {
        #[derive(Config)]
        struct Example<T> {
            #[setting(flatten, nested)]
            child: Child,
            value: T,
        }
    });

    assert!(generic.impl_partial_checks().is_empty());
}
