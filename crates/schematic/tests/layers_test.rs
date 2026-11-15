#![allow(dead_code)]

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

// The layers are read against what the sources picked, so they report the
// defaults and variables of a picked variant, and of an optional config
mod picked {
    use super::*;

    #[derive(Debug, Config)]
    #[config(env_prefix = "LAYERS_PICKED_")]
    #[serde(tag = "type")]
    enum Backend {
        #[setting(default)]
        Memory,
        Disk {
            #[setting(default = 10)]
            size: usize,
            path: Option<String>,
        },
    }

    #[test]
    #[serial]
    fn reports_the_defaults_and_env_of_a_picked_variant() {
        unsafe { env::set_var("LAYERS_PICKED_PATH", "/tmp") };

        let result = ConfigLoader::<Backend>::new()
            .code("type: Disk", "code.yml")
            .unwrap()
            .load();

        unsafe { env::remove_var("LAYERS_PICKED_PATH") };

        let result = result.unwrap();
        let partials = result
            .layers
            .iter()
            .map(|layer| (layer.source.clone(), layer.partial.clone()))
            .collect::<Vec<_>>();

        assert_eq!(
            partials,
            [
                (
                    Source::Defaults,
                    PartialBackend::Disk {
                        size: Some(10),
                        path: None,
                    }
                ),
                (
                    Source::code("type: Disk", "code.yml").unwrap(),
                    PartialBackend::Disk {
                        size: None,
                        path: None,
                    }
                ),
                (
                    Source::EnvVars,
                    PartialBackend::Disk {
                        size: None,
                        path: Some("/tmp".into()),
                    }
                ),
            ]
        );
    }

    #[derive(Debug, Config)]
    struct Child {
        #[setting(default = 1)]
        level: usize,
    }

    #[derive(Debug, Config)]
    struct Parent {
        #[setting(nested)]
        maybe: Option<Child>,
    }

    // A layer of defaults that would set nothing isn't reported at all
    #[test]
    fn reports_the_defaults_of_an_optional_config_only_when_present() {
        let present = ConfigLoader::<Parent>::new()
            .code("maybe: {}", "code.yml")
            .unwrap()
            .load()
            .unwrap();

        assert_eq!(present.layers[0].source, Source::Defaults);
        assert_eq!(
            present.layers[0].partial.maybe,
            Some(PartialChild { level: Some(1) })
        );

        let missing = ConfigLoader::<Parent>::new().load().unwrap();

        assert!(missing.layers.is_empty());
        assert!(missing.config.maybe.is_none());
    }

    #[derive(Debug, Config)]
    enum Named {
        #[setting(default)]
        A,
        B {
            #[setting(default = 1)]
            b: usize,
        },
    }

    // Finalizing only transforms, so a named variant gets nothing from it
    #[test]
    fn finalizing_applies_no_defaults_to_a_named_variant() {
        let partial = PartialNamed::B { b: None }.finalize(&()).unwrap();

        assert_eq!(partial, PartialNamed::B { b: None });
    }

    static CALLS: AtomicUsize = AtomicUsize::new(0);

    fn count_calls(_: &()) -> DefaultValueResult<usize> {
        Ok(Some(CALLS.fetch_add(1, Ordering::SeqCst)))
    }

    #[derive(Debug, Config)]
    #[serde(tag = "type")]
    enum Counted {
        #[setting(default)]
        A,
        B {
            #[setting(default = count_calls)]
            calls: usize,
        },
    }

    #[test]
    fn calls_a_default_handler_of_a_picked_variant_once() {
        let result = ConfigLoader::<Counted>::new()
            .code("type: B", "code.yml")
            .unwrap()
            .load()
            .unwrap();

        assert_eq!(CALLS.load(Ordering::SeqCst), 1);
        assert!(matches!(result.config, Counted::B { calls: 0 }));
    }
}
