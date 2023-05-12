use schematic::*;
use std::path::PathBuf;

#[derive(Debug, Config)]
pub struct Config {
    boolean: bool,
    string: String,
    number: usize,
    vector: Vec<String>,
}

#[cfg(feature = "json")]
#[tokio::test]
async fn loads_json_files() {
    let root =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("tests/__fixtures__/json");

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
        .await
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "bar");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.vector, vec!["x", "y", "z"]);
}

#[cfg(feature = "toml")]
#[tokio::test]
async fn loads_toml_files() {
    let root =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("tests/__fixtures__/toml");

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
        .await
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "bar");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.vector, vec!["x", "y", "z"]);
}

#[tokio::test]
async fn loads_yaml_files() {
    let root =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("tests/__fixtures__/yaml");

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
        .await
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "bar");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.vector, vec!["x", "y", "z"]);
}
