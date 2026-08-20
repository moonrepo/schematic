mod utils;

use schematic::*;
use utils::*;

#[derive(Debug, Config)]
pub struct Config {
    boolean: bool,
    string: String,
    number: usize,
    vector: Vec<String>,
}

fn get_url(path: &str) -> String {
    format!(
        "https://raw.githubusercontent.com/moonrepo/schematic/master/crates/schematic/tests/__fixtures__/{path}"
    )
}

#[test]
fn can_create_url_source() {
    let source = Source::new("https://some/path/config.yml", None).unwrap();

    assert_eq!(
        source,
        Source::Url {
            url: "https://some/path/config.yml".to_owned(),
            content: None
        }
    );
}

mod preload {
    use super::*;
    use starbase_sandbox::create_empty_sandbox;
    use std::fs;

    const YAML: &str = "boolean: true\nstring: preloaded\nnumber: 9\nvector: [a, b]";

    // A URL that resolves to nothing, so any attempt to fetch it fails fast
    // instead of reaching the network
    const UNREACHABLE: &str = "https://127.0.0.1:1/config.yml";

    // A reserved TLD (RFC 2606) that never resolves, and is not loopback, so
    // it counts as insecure
    const INSECURE: &str = "http://schematic.invalid/config.yml";

    fn preloaded(url: &str, content: &str) -> Source {
        Source::Url {
            url: url.to_owned(),
            content: Some(content.to_owned()),
        }
    }

    #[test]
    fn content_is_used_instead_of_requesting() {
        let result = ConfigLoader::<Config>::new()
            .source(preloaded(UNREACHABLE, YAML))
            .unwrap()
            .load()
            .unwrap();

        assert!(result.config.boolean);
        assert_eq!(result.config.string, "preloaded");
        assert_eq!(result.config.number, 9);
        assert_eq!(result.config.vector, vec!["a", "b"]);
    }

    #[test]
    fn content_layers_with_other_sources() {
        let result = ConfigLoader::<Config>::new()
            .source(preloaded(UNREACHABLE, YAML))
            .unwrap()
            .code("string: overridden", "code.yml")
            .unwrap()
            .load()
            .unwrap();

        // The later layer wins, the preloaded one still supplies the rest
        assert_eq!(result.config.string, "overridden");
        assert_eq!(result.config.number, 9);
    }

    #[test]
    #[should_panic(expected = "HttpsOnly")]
    fn content_does_not_bypass_the_https_check() {
        ConfigLoader::<Config>::new()
            .source(preloaded(INSECURE, YAML))
            .unwrap()
            .load()
            .unwrap();
    }

    #[test]
    fn loopback_counts_as_secure() {
        let result = ConfigLoader::<Config>::new()
            .source(preloaded("http://127.0.0.1:1/config.yml", YAML))
            .unwrap()
            .load()
            .unwrap();

        assert_eq!(result.config.string, "preloaded");
    }

    #[tokio::test]
    async fn reads_from_the_cacher_instead_of_requesting() {
        let sandbox = create_empty_sandbox();
        fs::write(sandbox.path().join("config.yml"), YAML).unwrap();

        let mut loader = ConfigLoader::<Config>::new();
        loader.set_cacher(SandboxCacher {
            root: sandbox.path().to_owned(),
        });

        // Would fail if it went to the network
        loader.url_preload(UNREACHABLE).await.unwrap();

        let result = loader.load().unwrap();

        assert_eq!(result.config.string, "preloaded");
    }

    #[tokio::test]
    async fn stores_what_it_read_on_the_source() {
        let sandbox = create_empty_sandbox();
        fs::write(sandbox.path().join("config.yml"), YAML).unwrap();

        let mut loader = ConfigLoader::<Config>::new();
        loader.set_cacher(SandboxCacher {
            root: sandbox.path().to_owned(),
        });
        loader.url_preload(UNREACHABLE).await.unwrap();

        let layers = loader.load().unwrap().layers;

        assert_eq!(
            layers[0].source,
            Source::Url {
                url: UNREACHABLE.to_owned(),
                content: Some(YAML.to_owned()),
            }
        );
    }

    #[tokio::test]
    async fn strips_a_bom_from_cached_content() {
        let sandbox = create_empty_sandbox();
        fs::write(sandbox.path().join("config.yml"), format!("\u{feff}{YAML}")).unwrap();

        let mut loader = ConfigLoader::<Config>::new();
        loader.set_cacher(SandboxCacher {
            root: sandbox.path().to_owned(),
        });
        loader.url_preload(UNREACHABLE).await.unwrap();

        assert_eq!(loader.load().unwrap().config.string, "preloaded");
    }

    // The tests below reach the network, like the rest of this file

    #[tokio::test]
    async fn preloads_and_loads_over_the_network() {
        let mut loader = ConfigLoader::<Config>::new();

        loader.url_preload(get_url("yaml/one.yml")).await.unwrap();
        loader.url_preload(get_url("yaml/two.yml")).await.unwrap();

        let result = loader.load().unwrap();

        assert_eq!(result.config.string, "foo");
        assert_eq!(result.config.vector, vec!["a", "b", "c"]);
    }

    #[tokio::test]
    async fn preloaded_layers_merge_in_declaration_order() {
        let mut loader = ConfigLoader::<Config>::new();

        for name in ["yaml/one.yml", "yaml/two.yml", "yaml/three.yml"] {
            loader.url_preload(get_url(name)).await.unwrap();
        }

        let result = loader.load().unwrap();

        // `one` sets string to foo, `three` overrides it to bar
        assert_eq!(result.config.string, "bar");
        assert!(result.config.boolean);
        assert_eq!(result.config.vector, vec!["a", "b", "c"]);
    }

    // Preloading only helps for the URLs that were actually preloaded. A URL
    // added with `url()` is still fetched lazily through `reqwest::blocking`
    // during `load()`, which cannot run inside an async runtime. Every URL has
    // to be preloaded, or `load()` has to be moved off the runtime.
    #[tokio::test]
    #[should_panic(expected = "Cannot drop a runtime")]
    async fn a_lazy_url_still_blocks_when_loaded_from_async() {
        let mut loader = ConfigLoader::<Config>::new();

        loader.url_preload(get_url("yaml/one.yml")).await.unwrap();
        loader.url(get_url("yaml/two.yml")).unwrap();

        loader.load().unwrap();
    }

    #[tokio::test]
    async fn writes_to_the_cacher_on_a_miss() {
        let sandbox = create_empty_sandbox();
        let cached = sandbox.path().join("one.yml");

        let mut loader = ConfigLoader::<Config>::new();
        loader.set_cacher(SandboxCacher {
            root: sandbox.path().to_owned(),
        });

        assert!(!cached.exists());

        loader.url_preload(get_url("yaml/one.yml")).await.unwrap();

        assert!(cached.exists());
        assert!(fs::read_to_string(cached).unwrap().contains("string"));
    }

    // Preloading does not guard the scheme the way the lazy path does, so an
    // insecure URL is requested first and only rejected once it is parsed
    #[tokio::test]
    async fn requests_an_insecure_url_before_rejecting_it() {
        let error = ConfigLoader::<Config>::new()
            .url_preload(INSECURE)
            .await
            .err()
            .unwrap();

        assert!(
            matches!(error, ConfigError::ReadUrlFailed { .. }),
            "expected the request to be attempted, got {error:?}"
        );
    }
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

#[cfg(feature = "pkl")]
#[test]
fn loads_pkl_files() {
    use starbase_sandbox::create_empty_sandbox;

    let sandbox = create_empty_sandbox();

    let result = ConfigLoader::<Config>::new()
        .set_cacher(SandboxCacher {
            root: sandbox.path().to_owned(),
        })
        .url(get_url("pkl/one.pkl"))
        .unwrap()
        .url(get_url("pkl/two.pkl"))
        .unwrap()
        .url(get_url("pkl/three.pkl"))
        .unwrap()
        .url(get_url("pkl/four.pkl"))
        .unwrap()
        .url(get_url("pkl/five.pkl"))
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
