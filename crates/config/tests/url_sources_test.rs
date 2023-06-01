use schematic::*;

#[derive(Debug, Config)]
pub struct Config {
    boolean: bool,
    string: String,
    number: usize,
    vector: Vec<String>,
}

fn get_url(path: &str) -> String {
    format!("https://raw.githubusercontent.com/moonrepo/schematic/master/crates/config/tests/__fixtures__/{}", path)
}

#[test]
fn can_create_url_source() {
    let source = Source::new("https://some/path/config.yml", None).unwrap();

    assert_eq!(
        source,
        Source::Url {
            url: "https://some/path/config.yml".to_owned(),
            format: SourceFormat::Yaml,
        }
    );
}

#[test]
#[should_panic(expected = "HttpsOnly")]
fn errors_on_http() {
    ConfigLoader::<Config>::new()
        .url("http://some/path/config.yml")
        .unwrap()
        .load()
        .unwrap();
}

#[test]
#[should_panic(expected = "HttpsOnly")]
fn errors_on_www() {
    ConfigLoader::<Config>::new()
        .url("www.domain.com/some/path/config.yml")
        .unwrap()
        .load()
        .unwrap();
}

#[cfg(feature = "json")]
#[test]
fn loads_json_files() {
    let result = ConfigLoader::<Config>::new()
        .url(get_url("json/one.json"))
        .unwrap()
        .url(get_url("json/two.json"))
        .unwrap()
        .url(get_url("json/three.json"))
        .unwrap()
        .url(get_url("json/four.json"))
        .unwrap()
        .url(get_url("json/five.json"))
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
fn loads_toml_files() {
    let result = ConfigLoader::<Config>::new()
        .url(get_url("toml/one.toml"))
        .unwrap()
        .url(get_url("toml/two.toml"))
        .unwrap()
        .url(get_url("toml/three.toml"))
        .unwrap()
        .url(get_url("toml/four.toml"))
        .unwrap()
        .url(get_url("toml/five.toml"))
        .unwrap()
        .load()
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "bar");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.vector, vec!["x", "y", "z"]);
}

#[test]
fn loads_yaml_files() {
    let result = ConfigLoader::<Config>::new()
        .url(get_url("yaml/one.yml"))
        .unwrap()
        .url(get_url("yaml/two.yml"))
        .unwrap()
        .url(get_url("yaml/three.yml"))
        .unwrap()
        .url(get_url("yaml/four.yml"))
        .unwrap()
        .url(get_url("yaml/five.yml"))
        .unwrap()
        .load()
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "bar");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.vector, vec!["x", "y", "z"]);
}
