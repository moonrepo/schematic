mod utils;

use schematic::*;
use serial_test::serial;
use std::env;
use utils::get_fixture_path;

fn validate_port(port: &usize, _: &PartialApp, _: &(), _: bool) -> ValidateResult {
    if *port == 0 {
        return Err(ValidateError::new("port can't be 0"));
    }

    Ok(())
}

#[derive(Debug, Config)]
struct App {
    #[setting(default = 3000, validate = validate_port)]
    port: usize,

    #[setting(default = "localhost")]
    host: String,

    secure: bool,

    #[setting(nested)]
    nested: Nested,
}

#[derive(Debug, Config)]
struct Nested {
    #[setting(default = "info")]
    level: String,
}

fn cli(port: Option<usize>) -> PartialApp {
    PartialApp {
        port,
        ..Default::default()
    }
}

#[test]
fn loads_a_partial_on_its_own() {
    let result = ConfigLoader::<App>::new()
        .partial("CLI arguments", cli(Some(8080)))
        .load()
        .unwrap();

    assert_eq!(result.config.port, 8080);
    assert_eq!(result.config.host, "localhost");
    assert_eq!(result.config.nested.level, "info");
}

#[test]
fn merges_in_the_order_added() {
    let code = "port: 4000\nsecure: true";

    let after = ConfigLoader::<App>::new()
        .code(code, "code.yml")
        .unwrap()
        .partial("CLI arguments", cli(Some(8080)))
        .load()
        .unwrap();

    assert_eq!(after.config.port, 8080);
    assert!(after.config.secure);

    let before = ConfigLoader::<App>::new()
        .partial("CLI arguments", cli(Some(8080)))
        .code(code, "code.yml")
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(before.config.port, 4000);
}

#[test]
fn merges_nested_partials() {
    let partial = PartialApp {
        nested: Some(PartialNested {
            level: Some("debug".into()),
        }),
        ..Default::default()
    };

    let result = ConfigLoader::<App>::new()
        .code("host: example.com", "code.yml")
        .unwrap()
        .partial("CLI arguments", partial)
        .load()
        .unwrap();

    assert_eq!(result.config.host, "example.com");
    assert_eq!(result.config.nested.level, "debug");
}

#[test]
fn reports_the_partial_as_a_layer() {
    let result = ConfigLoader::<App>::new()
        .code("port: 4000", "code.yml")
        .unwrap()
        .partial("CLI arguments", cli(Some(8080)))
        .load()
        .unwrap();

    assert_eq!(
        result
            .layers
            .iter()
            .map(|layer| layer.source.clone())
            .collect::<Vec<_>>(),
        vec![
            Source::Defaults,
            Source::Code {
                code: "port: 4000".into(),
                path: "code.yml".into(),
            },
            Source::Memory {
                name: "CLI arguments".into(),
            },
        ]
    );
    assert_eq!(result.layers[2].partial, cli(Some(8080)));
}

#[test]
fn validates_the_partial_by_name() {
    let error = ConfigLoader::<App>::new()
        .partial("CLI arguments", cli(Some(0)))
        .load()
        .err()
        .unwrap();

    assert_eq!(
        error.to_full_string(),
        "Failed to validate CLI arguments. \n  port: port can't be 0"
    );
}

// Its own type, so that no other test reads the variable
#[derive(Debug, Config)]
#[config(env_prefix = "MEMORY_")]
struct EnvApp {
    port: usize,
}

#[test]
#[serial]
fn sits_below_env_vars() {
    unsafe { env::set_var("MEMORY_PORT", "9000") };

    let result = ConfigLoader::<EnvApp>::new()
        .partial("CLI arguments", PartialEnvApp { port: Some(8080) })
        .load();

    unsafe { env::remove_var("MEMORY_PORT") };

    assert_eq!(result.unwrap().config.port, 9000);
}

#[test]
fn errors_for_a_memory_source_without_a_partial() {
    let mut loader = ConfigLoader::<App>::new();

    let error = loader
        .source(Source::Memory {
            name: "CLI arguments".into(),
        })
        .err()
        .unwrap();

    assert!(matches!(error, ConfigError::MemoryWithoutPartial(name) if name == "CLI arguments"));
}

mod extends {
    use super::*;

    #[derive(Debug, Config)]
    struct Extending {
        #[setting(extend)]
        extends: Option<String>,

        #[setting(merge = merge::append_vec)]
        value: Vec<usize>,
    }

    fn cli(extends: &str) -> PartialExtending {
        PartialExtending {
            extends: Some(extends.into()),
            value: Some(vec![0]),
        }
    }

    #[test]
    fn extends_an_absolute_file() {
        let base = get_fixture_path("extending").join("string1.yml");

        let result = ConfigLoader::<Extending>::new()
            .partial("CLI arguments", cli(base.to_str().unwrap()))
            .load()
            .unwrap();

        assert_eq!(result.config.extends, None);
        assert_eq!(result.config.value, vec![3, 2, 0]);
    }

    #[test]
    fn extends_a_relative_file_from_the_working_directory() {
        // Tests run within the package root
        let result = ConfigLoader::<Extending>::new()
            .partial(
                "CLI arguments",
                cli("./tests/__fixtures__/extending/string2.yml"),
            )
            .load()
            .unwrap();

        assert_eq!(result.config.value, vec![3, 0]);
        assert_eq!(
            result.layers[1].source,
            Source::File {
                path: "./tests/__fixtures__/extending/string2.yml".into(),
                required: true,
            }
        );
    }
}
