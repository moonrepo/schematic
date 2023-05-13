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
        let error = ConfigLoader::<BaseConfig>::new(SourceFormat::Json)
            .code(r#"{ "setting": 123 }"#)
            .unwrap()
            .load()
            .err()
            .unwrap();

        assert_eq!(error.to_string(), "Failed to parse JSON setting `setting`.")
    }

    #[test]
    fn invalid_nested_type() {
        let error = ConfigLoader::<BaseConfig>::new(SourceFormat::Json)
            .code(r#"{ "nested": { "setting": 123 } }"#)
            .unwrap()
            .load()
            .err()
            .unwrap();

        assert_eq!(
            error.to_string(),
            "Failed to parse JSON setting `nested.setting`."
        )
    }
}

#[cfg(feature = "json")]
mod toml {
    use super::*;

    #[test]
    fn invalid_type() {
        let error = ConfigLoader::<BaseConfig>::new(SourceFormat::Toml)
            .code("setting = 123")
            .unwrap()
            .load()
            .err()
            .unwrap();

        assert_eq!(error.to_string(), "Failed to parse TOML setting `setting`.")
    }

    #[test]
    fn invalid_nested_type() {
        let error = ConfigLoader::<BaseConfig>::new(SourceFormat::Toml)
            .code("[nested]\nsetting = 123")
            .unwrap()
            .load()
            .err()
            .unwrap();

        assert_eq!(
            error.to_string(),
            "Failed to parse TOML setting `nested.setting`."
        )
    }
}

#[cfg(feature = "yaml")]
mod yaml {
    use super::*;

    #[test]
    fn invalid_type() {
        let error = ConfigLoader::<BaseConfig>::new(SourceFormat::Yaml)
            .code("---\nsetting: 123")
            .unwrap()
            .load()
            .err()
            .unwrap();

        assert_eq!(error.to_string(), "Failed to parse YAML setting `setting`.")
    }

    #[test]
    fn invalid_nested_type() {
        let error = ConfigLoader::<BaseConfig>::new(SourceFormat::Yaml)
            .code("---\nnested:\n  setting: 123")
            .unwrap()
            .load()
            .err()
            .unwrap();

        assert_eq!(
            error.to_string(),
            "Failed to parse YAML setting `nested.setting`."
        )
    }
}
