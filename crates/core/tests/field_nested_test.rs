use quote::format_ident;
use schematic_core::container::Container;
use schematic_core::field_value::FieldNestedIdent;
use syn::parse_quote;

mod field_nested {
    use super::*;

    #[test]
    fn word() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(nested)]
                a: NestedConfig,
            }
        });
        let fields = container.inner.get_fields();

        assert!(fields[0].value.nested);
        assert_eq!(
            fields[0].value.nested_ident.as_ref().unwrap(),
            &FieldNestedIdent::Unknown(format_ident!("NestedConfig"))
        );
    }

    #[test]
    fn bool_true() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(nested = true)]
                a: NestedConfig,
            }
        });
        let fields = container.inner.get_fields();

        assert!(fields[0].value.nested);
        assert_eq!(
            fields[0].value.nested_ident.as_ref().unwrap(),
            &FieldNestedIdent::Unknown(format_ident!("NestedConfig"))
        );
    }

    #[test]
    fn bool_false() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(nested = false)]
                a: NestedConfig,
            }
        });
        let fields = container.inner.get_fields();

        assert!(!fields[0].value.nested);
        assert!(fields[0].value.nested_ident.is_none());
    }

    #[test]
    fn explicit_ident() {
        let container = Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(nested = NestedConfig)]
                a: NestedConfig,
            }
        });
        let fields = container.inner.get_fields();

        assert!(fields[0].value.nested);
        assert_eq!(
            fields[0].value.nested_ident.as_ref().unwrap(),
            &FieldNestedIdent::Unknown(format_ident!("NestedConfig"))
        );
    }

    #[test]
    #[should_panic(expected = "UnexpectedType(\"paren\")")]
    fn panics_invalid_expr() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(nested = (1 + 1))]
                a: NestedConfig,
            }
        });
    }

    #[test]
    #[should_panic(
        expected = "Unable to extract the nested configuration identifier from `Vec<Option<Box<NestedConfig>>>`. Try explicitly passing the identifier with `nested = ConfigName`."
    )]
    fn panics_cant_find_ident() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(nested)]
                a: Vec<Option<Box<NestedConfig>>>,
            }
        });
    }

    #[test]
    #[should_panic(
        expected = "Too many segments for `sub::NestedConfig`, only a single identifier is allowed."
    )]
    fn panics_too_many_segments() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(nested = sub::NestedConfig)]
                a: NestedConfig,
            }
        });
    }

    #[test]
    #[should_panic(
        expected = "Nested configuration identifier `OtherConfig` does not exist within `Vec<NestedConfig>`."
    )]
    fn panics_cant_find_custom_ident() {
        Container::from(parse_quote! {
            #[derive(Config)]
            struct Example {
                #[setting(nested = OtherConfig)]
                a: Vec<NestedConfig>,
            }
        });
    }
}
