use std::path::PathBuf;

use schematic::*;

#[derive(Debug, Config)]
pub struct Config {
    boolean: bool,
    string: String,
    number: usize,
    vector: Vec<String>,
}

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

    assert_eq!(result.config.boolean, false);
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

    assert_eq!(result.config.boolean, false);
    assert_eq!(result.config.string, "bar");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.vector, vec!["x", "y", "z"]);
}
