use schematic::*;
use serial_test::serial;
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Config)]
struct App {
    #[setting(default = 3000, env = "LAYERS_PORT")]
    port: usize,

    #[setting(default = "localhost")]
    host: String,

    #[setting(env = "LAYERS_DEBUG")]
    debug: bool,
}

fn sources(result: &ConfigLoadResult<App>) -> Vec<Source> {
    result
        .layers
        .iter()
        .map(|layer| layer.source.clone())
        .collect()
}

#[test]
#[serial]
fn reports_defaults_first_and_env_last() {
    unsafe { env::set_var("LAYERS_DEBUG", "true") };

    let result = ConfigLoader::<App>::new()
        .code("port: 4000", "code.yml")
        .unwrap()
        .load();

    unsafe { env::remove_var("LAYERS_DEBUG") };

    let result = result.unwrap();

    assert_eq!(
        sources(&result),
        vec![
            Source::Defaults,
            Source::Code {
                code: "port: 4000".into(),
                path: "code.yml".into(),
            },
            Source::EnvVars,
        ]
    );

    // Each layer holds only what it set
    assert_eq!(
        result.layers[0].partial,
        PartialApp {
            port: Some(3000),
            host: Some("localhost".into()),
            debug: Some(false),
        }
    );
    assert_eq!(
        result.layers[2].partial,
        PartialApp {
            debug: Some(true),
            ..Default::default()
        }
    );

    assert_eq!(result.config.port, 4000);
    assert!(result.config.debug);
}

#[test]
#[serial]
fn env_wins_over_every_source() {
    unsafe { env::set_var("LAYERS_PORT", "5000") };

    let result = ConfigLoader::<App>::new()
        .code("port: 4000", "code.yml")
        .unwrap()
        .partial(
            "CLI arguments",
            PartialApp {
                port: Some(8080),
                ..Default::default()
            },
        )
        .load();

    unsafe { env::remove_var("LAYERS_PORT") };

    let result = result.unwrap();

    assert_eq!(result.config.port, 5000);
    assert_eq!(sources(&result).last(), Some(&Source::EnvVars));
}

#[test]
#[serial]
fn leaves_out_env_when_none_is_set() {
    let result = ConfigLoader::<App>::new().load().unwrap();

    assert_eq!(sources(&result), vec![Source::Defaults]);
    assert_eq!(result.config.host, "localhost");
}

#[test]
fn errors_when_added_as_a_source() {
    let mut loader = ConfigLoader::<App>::new();

    for source in [Source::Defaults, Source::EnvVars] {
        let error = loader.source(source.clone()).err().unwrap();

        assert!(
            matches!(&error, ConfigError::LoaderOnlySource(name) if name == source.as_str()),
            "{error:?}"
        );
    }
}

mod run_once {
    use super::*;

    static CALLS: AtomicUsize = AtomicUsize::new(0);

    fn count_calls(_: &()) -> DefaultValueResult<usize> {
        Ok(Some(CALLS.fetch_add(1, Ordering::SeqCst)))
    }

    // Its own type, so that no other test calls the handler
    #[derive(Debug, Config)]
    struct Counted {
        #[setting(default = count_calls)]
        calls: usize,
    }

    // A handler that isn't idempotent, like finding an unused port, would
    // otherwise report a default other than the one that was used
    #[test]
    fn calls_a_default_handler_once_per_load() {
        let result = ConfigLoader::<Counted>::new().load().unwrap();

        assert_eq!(CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(result.layers[0].partial.calls, Some(result.config.calls));
    }
}

mod finalize {
    use super::*;

    fn double(value: usize, _: &()) -> TransformResult<usize> {
        Ok(value * 2)
    }

    #[derive(Debug, Config)]
    struct Transformed {
        #[setting(default = 1, transform = double)]
        value: usize,

        #[setting(default = "unused")]
        other: String,
    }

    #[test]
    fn only_transforms() {
        let partial = PartialTransformed {
            value: Some(5),
            other: None,
        }
        .finalize(&())
        .unwrap();

        assert_eq!(partial.value, Some(10));
        assert_eq!(partial.other, None);
    }

    #[test]
    fn transforms_after_every_layer() {
        let config = ConfigLoader::<Transformed>::new()
            .code("value: 3", "code.yml")
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(config.value, 6);
        assert_eq!(config.other, "unused");
    }
}

mod root_enum {
    use super::*;

    #[derive(Debug, Config)]
    enum Mode {
        #[setting(default = "auto")]
        Named(String),
        Off,
    }

    // The defaults are a layer, so a root enum's default variant gets its
    // value, the same as `Default`
    #[test]
    fn applies_the_default_variant_value() {
        let config = ConfigLoader::<Mode>::new().load().unwrap().config;

        assert!(
            matches!(&config, Mode::Named(value) if value == "auto"),
            "{config:?}"
        );
        assert!(matches!(Mode::default(), Mode::Named(value) if value == "auto"));
    }
}
