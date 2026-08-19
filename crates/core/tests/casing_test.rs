use schematic_core::container::Container;
use syn::parse_quote;

// There is no default case. A name is used exactly as it was written unless
// `rename_all` asks otherwise, and the casing is resolved here rather than
// only forwarded to serde, so schemas and `settings()` describe the key that
// is actually accepted.
mod fields {
    use super::*;

    fn names(input: syn::DeriveInput) -> Vec<String> {
        Container::from(input)
            .inner
            .get_fields()
            .iter()
            .map(|field| field.get_name())
            .collect()
    }

    #[test]
    fn leaves_names_alone_without_rename_all() {
        assert_eq!(
            names(parse_quote! {
                #[derive(Config)]
                struct Example {
                    some_field_name: String,
                }
            }),
            vec!["some_field_name"]
        );
    }

    #[test]
    fn applies_the_config_attribute() {
        assert_eq!(
            names(parse_quote! {
                #[derive(Config)]
                #[config(rename_all = "camelCase")]
                struct Example {
                    some_field_name: String,
                    another: usize,
                }
            }),
            vec!["someFieldName", "another"]
        );
    }

    #[test]
    fn applies_the_serde_attribute() {
        assert_eq!(
            names(parse_quote! {
                #[derive(Config)]
                #[serde(rename_all = "kebab-case")]
                struct Example {
                    some_field_name: String,
                }
            }),
            vec!["some-field-name"]
        );
    }

    #[test]
    fn config_takes_precedence_over_serde() {
        assert_eq!(
            names(parse_quote! {
                #[derive(Config)]
                #[config(rename_all = "camelCase")]
                #[serde(rename_all = "kebab-case")]
                struct Example {
                    some_field_name: String,
                }
            }),
            vec!["someFieldName"]
        );
    }

    // An explicit rename is used verbatim, never re-cased
    #[test]
    fn does_not_case_an_explicit_rename() {
        assert_eq!(
            names(parse_quote! {
                #[derive(Config)]
                #[config(rename_all = "camelCase")]
                struct Example {
                    #[setting(rename = "kept_as_is")]
                    some_field_name: String,
                    #[serde(rename = "also_kept")]
                    another_one: String,
                }
            }),
            vec!["kept_as_is", "also_kept"]
        );
    }

    #[test]
    fn supports_every_serde_case() {
        // `lowercase`/`UPPERCASE` only change case, they don't restructure
        // the name, so a snake_case field is already lowercase
        for (format, expected) in [
            ("lowercase", "some_field_name"),
            ("UPPERCASE", "SOME_FIELD_NAME"),
            ("PascalCase", "SomeFieldName"),
            ("camelCase", "someFieldName"),
            ("snake_case", "some_field_name"),
            ("SCREAMING_SNAKE_CASE", "SOME_FIELD_NAME"),
            ("kebab-case", "some-field-name"),
            ("SCREAMING-KEBAB-CASE", "SOME-FIELD-NAME"),
        ] {
            let got = names(parse_quote! {
                #[derive(Config)]
                #[config(rename_all = #format)]
                struct Example {
                    some_field_name: String,
                }
            });

            assert_eq!(got, vec![expected], "`{format}` produced the wrong name");
        }
    }

    #[test]
    #[should_panic(expected = "Unknown `rename_all` value `nope`")]
    fn rejects_an_unknown_case() {
        names(parse_quote! {
            #[derive(Config)]
            #[config(rename_all = "nope")]
            struct Example {
                some_field_name: String,
            }
        });
    }
}

mod variants {
    use super::*;

    fn names(input: syn::DeriveInput) -> Vec<String> {
        Container::from(input)
            .inner
            .get_variants()
            .iter()
            .map(|variant| variant.get_name())
            .collect()
    }

    #[test]
    fn leaves_names_alone_without_rename_all() {
        assert_eq!(
            names(parse_quote! {
                #[derive(Config)]
                enum Example {
                    SomeVariant,
                    AnotherOne,
                }
            }),
            vec!["SomeVariant", "AnotherOne"]
        );
    }

    #[test]
    fn applies_rename_all() {
        assert_eq!(
            names(parse_quote! {
                #[derive(Config)]
                #[serde(rename_all = "kebab-case")]
                enum Example {
                    SomeVariant,
                    AnotherOne,
                }
            }),
            vec!["some-variant", "another-one"]
        );
    }

    #[test]
    fn applies_rename_all_to_tuple_variants() {
        assert_eq!(
            names(parse_quote! {
                #[derive(Config)]
                #[config(rename_all = "snake_case")]
                enum Example {
                    SomeVariant(String),
                    AnotherOne(bool),
                }
            }),
            vec!["some_variant", "another_one"]
        );
    }

    #[test]
    fn does_not_case_an_explicit_rename() {
        assert_eq!(
            names(parse_quote! {
                #[derive(Config)]
                #[config(rename_all = "kebab-case")]
                enum Example {
                    #[setting(rename = "KeptAsIs")]
                    SomeVariant,
                    AnotherOne,
                }
            }),
            vec!["KeptAsIs", "another-one"]
        );
    }
}

// Environment variables stay derived from the Rust name, so a casing choice
// for the serialized shape doesn't reshape variable names.
mod env_keys {
    use super::*;

    fn keys(input: syn::DeriveInput) -> Vec<String> {
        Container::from(input)
            .inner
            .get_fields()
            .iter()
            .filter_map(|field| field.get_env_var().map(|key| format!("{key:?}")))
            .collect()
    }

    #[test]
    fn are_not_affected_by_rename_all() {
        let without = keys(parse_quote! {
            #[derive(Config)]
            #[config(env_prefix = "APP_")]
            struct Example {
                some_field_name: String,
            }
        });

        let with = keys(parse_quote! {
            #[derive(Config)]
            #[config(env_prefix = "APP_", rename_all = "camelCase")]
            struct Example {
                some_field_name: String,
            }
        });

        assert_eq!(without, with);
        assert!(
            without[0].contains("SOME_FIELD_NAME"),
            "got {:?}",
            without[0]
        );
    }
}
