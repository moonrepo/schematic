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
            .code(r#"{ "setting": 123 }"#, Format::Json)
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
            .code(r#"{ "nested": { "setting": 123 } }"#, Format::Json)
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

#[cfg(feature = "json")]
mod toml {
    use super::*;

    #[test]
    fn invalid_type() {
        let error = ConfigLoader::<BaseConfig>::new()
            .code("setting = 123", Format::Toml)
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
            .code("[nested]\nsetting = 123", Format::Toml)
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

#[cfg(feature = "yaml")]
mod yaml {
    use super::*;

    #[test]
    fn invalid_type() {
        let error = ConfigLoader::<BaseConfig>::new()
            .code("---\nsetting: 123", Format::Yaml)
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
            .code("---\nnested:\n  setting: 123", Format::Yaml)
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
