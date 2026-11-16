#![allow(dead_code)]

use schematic::*;

#[derive(Debug, Config)]
pub struct NestedConfig {
    setting: bool,
}

#[derive(Debug, Config)]
pub struct BaseConfig {
    setting: bool,
    #[setting(nested)]
    nested: NestedConfig,
}

fn strip_ansi(s: &str) -> String {
    String::from_utf8(strip_ansi_escapes::strip(s)).unwrap()
}

/// Load `code` as `file`, and return the path of the parse error and the
/// content from where its span begins, with the text the span labels.
#[allow(unused)]
fn locate_error(code: &str, file: &str) -> (String, String, String) {
    let error = ConfigLoader::<BaseConfig>::new()
        .code(code, file)
        .unwrap()
        .load()
        .err()
        .unwrap();

    let ConfigError::Parser { error, .. } = error else {
        panic!("expected a parser error, got {error:?}");
    };

    let span = error.span.expect("a span");
    let start = span.offset();

    (
        error.path,
        code[start..].to_owned(),
        code[start..start + span.len()].to_owned(),
    )
}

#[cfg(feature = "json")]
mod json {
    use super::*;

    #[test]
    fn invalid_type() {
        let error = ConfigLoader::<BaseConfig>::new()
            .code(r#"{ "setting": 123 }"#, "code.json")
            .unwrap()
            .load()
            .err()
            .unwrap();

        assert_eq!(
            strip_ansi(&error.to_full_string()),
            "Failed to parse code.json. setting: invalid type: integer `123`, expected a boolean at line 1 column 16"
        )
    }

    #[test]
    fn invalid_nested_type() {
        let error = ConfigLoader::<BaseConfig>::new()
            .code(r#"{ "nested": { "setting": 123 } }"#, "code.json")
            .unwrap()
            .load()
            .err()
            .unwrap();

        assert_eq!(
            strip_ansi(&error.to_full_string()),
            "Failed to parse code.json. nested.setting: invalid type: integer `123`, expected a boolean at line 1 column 28"
        )
    }
}

#[cfg(feature = "pkl")]
mod pkl {
    use super::*;
    use starbase_sandbox::{locate_fixture, predicates::prelude::*};

    #[test]
    fn invalid_type() {
        let error = ConfigLoader::<BaseConfig>::new()
            .file(locate_fixture("pkl").join("invalid-type.pkl"))
            .unwrap()
            .load()
            .err()
            .unwrap();

        let error_str = strip_ansi(&error.to_full_string());
        println!("{}", error_str);

        assert!(
            predicate::str::contains("setting: invalid type: integer `123`, expected a boolean")
                .eval(&error_str)
        )
    }

    #[test]
    fn invalid_nested_type() {
        let error = ConfigLoader::<BaseConfig>::new()
            .file(locate_fixture("pkl").join("invalid-nested-type.pkl"))
            .unwrap()
            .load()
            .err()
            .unwrap();

        let error_str = strip_ansi(&error.to_full_string());
        println!("{}", error_str);

        assert!(
            predicate::str::contains(
                "nested.setting: invalid type: integer `123`, expected a boolean"
            )
            .eval(&error_str)
        )
    }
}

#[cfg(feature = "toml")]
mod toml {
    use super::*;

    // A syntax error is a parse error of the file, like a type error, rather
    // than a bare message
    #[test]
    fn invalid_syntax() {
        let error = ConfigLoader::<BaseConfig>::new()
            .code("setting = ", "code.toml")
            .unwrap()
            .set_help("See the docs")
            .load()
            .err()
            .unwrap();

        assert_eq!(
            strip_ansi(&error.to_full_string()),
            "Failed to parse code.toml. .: string values must be quoted, expected literal string"
        );

        let ConfigError::Parser {
            location,
            error,
            help,
        } = error
        else {
            panic!("expected a parser error, got {error:?}");
        };

        assert_eq!(location, "code.toml");
        assert_eq!(help.as_deref(), Some("See the docs"));
        assert_eq!(error.path, ".");
        assert_eq!(error.span.map(|span| span.offset()), Some(10));
    }

    #[test]
    fn invalid_type() {
        let error = ConfigLoader::<BaseConfig>::new()
            .code("setting = 123", "code.toml")
            .unwrap()
            .load()
            .err()
            .unwrap();

        assert_eq!(
            strip_ansi(&error.to_full_string()),
            "Failed to parse code.toml. setting: invalid type: integer `123`, expected a boolean"
        )
    }

    #[test]
    fn invalid_nested_type() {
        let error = ConfigLoader::<BaseConfig>::new()
            .code("[nested]\nsetting = 123", "code.toml")
            .unwrap()
            .load()
            .err()
            .unwrap();

        assert_eq!(
            strip_ansi(&error.to_full_string()),
            "Failed to parse code.toml. nested.setting: invalid type: integer `123`, expected a boolean"
        )
    }
}

#[cfg(feature = "yaml")]
mod yaml {
    use super::*;

    // Merge keys are applied to a value first, which holds no location, so the
    // error is located by deserializing the content directly
    #[test]
    fn locates_a_type_error() {
        let (path, from, _) = locate_error("nested:\n  setting: 123", "code.yaml");

        assert_eq!(path, "nested.setting");
        assert_eq!(from, "123");
    }

    // The content without its merge keys applied fails elsewhere, at `<<`,
    // which would label the wrong place
    #[test]
    fn leaves_an_error_from_a_merge_key_unlocated() {
        let error = ConfigLoader::<BaseConfig>::new()
            .code("nested:\n  <<: { setting: 123 }", "code.yaml")
            .unwrap()
            .load()
            .err()
            .unwrap();

        let ConfigError::Parser { error, .. } = error else {
            panic!("expected a parser error, got {error:?}");
        };

        assert_eq!(error.path, "nested.setting");
        assert_eq!(error.span, None);
    }

    #[test]
    fn locates_an_unknown_field() {
        let (path, from, _) = locate_error("setting: true\nfoo: 1", "code.yaml");

        assert_eq!(path, "foo");
        assert_eq!(from, "foo: 1");
    }

    #[test]
    fn invalid_type() {
        let error = ConfigLoader::<BaseConfig>::new()
            .code("---\nsetting: 123", "code.yaml")
            .unwrap()
            .load()
            .err()
            .unwrap();

        assert_eq!(
            strip_ansi(&error.to_full_string()),
            "Failed to parse code.yaml. setting: invalid type: integer `123`, expected a boolean"
        )
    }

    #[test]
    fn invalid_nested_type() {
        let error = ConfigLoader::<BaseConfig>::new()
            .code("---\nnested:\n  setting: 123", "code.yaml")
            .unwrap()
            .load()
            .err()
            .unwrap();

        assert_eq!(
            strip_ansi(&error.to_full_string()),
            "Failed to parse code.yaml. nested.setting: invalid type: integer `123`, expected a boolean"
        )
    }
}

#[cfg(feature = "ron")]
mod ron {
    use super::*;

    #[test]
    fn locates_a_type_error() {
        let (path, from, _) = locate_error("(nested: (setting: 1))", "code.ron");

        assert_eq!(path, "nested.setting");
        assert_eq!(from, "1))");
    }

    #[test]
    fn labels_an_unknown_field() {
        let (path, _, label) = locate_error("(foo: true)", "code.ron");

        assert_eq!(path, "foo");
        assert_eq!(label, "foo");
    }

    #[test]
    fn locates_a_syntax_error() {
        let (path, from, _) = locate_error("(setting: )", "code.ron");

        assert_eq!(path, "setting");
        assert_eq!(from, ")");
    }

    // As `ron::from_str` does, rather than ignoring what follows
    #[test]
    fn errors_for_trailing_characters() {
        let (path, from, _) = locate_error("(setting: true) x", "code.ron");

        assert_eq!(path, ".");
        assert_eq!(from, "x");
    }
}

// Every format feature is enabled here, so the help asks whether a format
// was registered, as a loader without `new` has none
mod no_matching_format {
    use super::*;
    use miette::Diagnostic;

    #[test]
    fn asks_for_a_registered_format() {
        let error = ConfigLoader::<BaseConfig>::default()
            .code("{}", "code.json")
            .unwrap()
            .load()
            .err()
            .unwrap();

        assert!(matches!(error, ConfigError::NoMatchingFormat { .. }));
        assert_eq!(
            error.help().map(|help| help.to_string()).as_deref(),
            Some(
                "Is there a format registered for the file extension? ConfigLoader::new registers one for each format feature that's enabled."
            )
        );
    }
}
