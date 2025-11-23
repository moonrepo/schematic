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
            error.to_full_string(),
            "Failed to parse BaseConfig. setting: invalid type: integer `123`, expected a boolean at line 1 column 16"
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
            error.to_full_string(),
            "Failed to parse BaseConfig. nested.setting: invalid type: integer `123`, expected a boolean at line 1 column 28"
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

        println!("{}", error.to_full_string());

        assert!(
            predicate::str::contains("setting: invalid type: integer `123`, expected a boolean")
                .eval(&error.to_full_string())
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

        println!("{}", error.to_full_string());

        assert!(
            predicate::str::contains(
                "nested.setting: invalid type: integer `123`, expected a boolean"
            )
            .eval(&error.to_full_string())
        )
    }
}

#[cfg(feature = "toml")]
mod toml {
    use super::*;

    #[test]
    fn invalid_type() {
        let error = ConfigLoader::<BaseConfig>::new()
            .code("setting = 123", "code.toml")
            .unwrap()
            .load()
            .err()
            .unwrap();

        assert_eq!(
            error.to_full_string(),
            "Failed to parse BaseConfig. setting: invalid type: integer `123`, expected a boolean"
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
            error.to_full_string(),
            "Failed to parse BaseConfig. nested.setting: invalid type: integer `123`, expected a boolean"
        )
    }
}

#[cfg(feature = "yml")]
mod yaml {
    use super::*;

    #[test]
    fn invalid_type() {
        let error = ConfigLoader::<BaseConfig>::new()
            .code("---\nsetting: 123", "code.yaml")
            .unwrap()
            .load()
            .err()
            .unwrap();

        assert_eq!(
            error.to_full_string(),
            "Failed to parse BaseConfig. setting: invalid type: integer `123`, expected a boolean"
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
            error.to_full_string(),
            "Failed to parse BaseConfig. nested.setting: invalid type: integer `123`, expected a boolean"
        )
    }
}
