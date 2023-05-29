mod utils;

use crate::utils::get_fixture_path;
use schematic::*;
use std::path::PathBuf;

#[derive(Debug, Config)]
pub struct Config {
    boolean: bool,
    string: String,
    number: usize,
    vector: Vec<String>,
}

#[test]
fn can_create_file_source() {
    let source = Source::new("file://some/path/config.yml", None).unwrap();

    assert_eq!(
        source,
        Source::File {
            path: PathBuf::from("some/path/config.yml"),
            required: true,
        }
    );

    let source = Source::new("./some/path/config.yml", None).unwrap();

    assert_eq!(
        source,
        Source::File {
            path: PathBuf::from("./some/path/config.yml"),
            required: true,
        }
    );

    let source = Source::new("/some/path/config.yml", None).unwrap();

    assert_eq!(
        source,
        Source::File {
            path: PathBuf::from("/some/path/config.yml"),
            required: true,
        }
    );

    let source = Source::new("some/path/config.yml", None).unwrap();

    assert_eq!(
        source,
        Source::File {
            path: PathBuf::from("some/path/config.yml"),
            required: true,
        }
    );
}

#[test]
fn can_create_file_source_with_parent() {
    let parent = Source::File {
        path: PathBuf::from("/root/config.yml"),
        required: true,
    };

    let source = Source::new("file://some/path/config.yml", Some(&parent)).unwrap();

    assert_eq!(
        source,
        Source::File {
            path: PathBuf::from("/root/some/path/config.yml"),
            required: true,
        }
    );

    let source = Source::new("./some/path/config.yml", Some(&parent)).unwrap();

    assert_eq!(
        source,
        Source::File {
            path: PathBuf::from("/root/some/path/config.yml"),
            required: true,
        }
    );

    let source = Source::new("/some/path/config.yml", Some(&parent)).unwrap();

    assert_eq!(
        source,
        Source::File {
            path: PathBuf::from("/some/path/config.yml"),
            required: true,
        }
    );

    let source = Source::new("some/path/config.yml", Some(&parent)).unwrap();

    assert_eq!(
        source,
        Source::File {
            path: PathBuf::from("/root/some/path/config.yml"),
            required: true,
        }
    );
}

#[cfg(feature = "json")]
#[test]
fn loads_json_files() {
    let root = get_fixture_path("json");

    let result = ConfigLoader::<Config>::new(SourceFormat::Json)
        .file(root.join("one.json"))
        .unwrap()
        .file(root.join("two.json"))
        .unwrap()
        .file(root.join("three.json"))
        .unwrap()
        .file(root.join("four.json"))
        .unwrap()
        .file(root.join("five.json"))
        .unwrap()
        .load()
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "bar");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.vector, vec!["x", "y", "z"]);
}

#[cfg(feature = "json")]
#[test]
fn loads_json_file_optional() {
    let root = get_fixture_path("json");

    let result = ConfigLoader::<Config>::new(SourceFormat::Json)
        .file_optional(root.join("missing.json"))
        .unwrap()
        .load()
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "");
    assert_eq!(result.config.number, 0);
}

#[cfg(feature = "toml")]
#[test]
fn loads_toml_files() {
    let root = get_fixture_path("toml");

    let result = ConfigLoader::<Config>::new(SourceFormat::Toml)
        .file(root.join("one.toml"))
        .unwrap()
        .file(root.join("two.toml"))
        .unwrap()
        .file(root.join("three.toml"))
        .unwrap()
        .file(root.join("four.toml"))
        .unwrap()
        .file(root.join("five.toml"))
        .unwrap()
        .load()
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "bar");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.vector, vec!["x", "y", "z"]);
}

#[cfg(feature = "toml")]
#[test]
fn loads_toml_file_optional() {
    let root = get_fixture_path("toml");

    let result = ConfigLoader::<Config>::new(SourceFormat::Toml)
        .file_optional(root.join("missing.toml"))
        .unwrap()
        .load()
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "");
    assert_eq!(result.config.number, 0);
}

#[test]
fn loads_yaml_files() {
    let root = get_fixture_path("yaml");

    let result = ConfigLoader::<Config>::new(SourceFormat::Yaml)
        .file(root.join("one.yml"))
        .unwrap()
        .file(root.join("two.yml"))
        .unwrap()
        .file(root.join("three.yml"))
        .unwrap()
        .file(root.join("four.yml"))
        .unwrap()
        .file(root.join("five.yml"))
        .unwrap()
        .load()
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "bar");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.vector, vec!["x", "y", "z"]);
}

#[cfg(feature = "yaml")]
#[test]
fn loads_yaml_file_optional() {
    let root = get_fixture_path("yaml");

    let result = ConfigLoader::<Config>::new(SourceFormat::Yaml)
        .file_optional(root.join("missing.yaml"))
        .unwrap()
        .load()
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "");
    assert_eq!(result.config.number, 0);
}
