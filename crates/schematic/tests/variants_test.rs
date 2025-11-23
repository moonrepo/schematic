use schematic::*;
use serial_test::serial;
use std::collections::HashMap;
use std::env;

fn test_list<T>(_: &[String], _: &T, context: &Context, _: bool) -> ValidateResult {
    if context.fail {
        return Err(ValidateError::new("invalid"));
    }

    Ok(())
}

fn test_map<T>(_: &HashMap<String, String>, _: &T, context: &Context, _: bool) -> ValidateResult {
    if context.fail {
        return Err(ValidateError::new("invalid"));
    }

    Ok(())
}

fn test_cfg<T>(_: &PartialProjectsConfig, _: &T, context: &Context, _: bool) -> ValidateResult {
    if context.fail {
        return Err(ValidateError::new("invalid"));
    }

    Ok(())
}

fn assert_validation_error(error: ConfigError, count: usize) {
    match error {
        ConfigError::Validator { error: inner, .. } => {
            assert_eq!(inner.errors.len(), count);
        }
        _ => {
            panic!("expected validation error");
        }
    }
}

#[derive(Default)]
pub struct Context {
    fail: bool,
}

#[derive(Debug, Config, Eq, PartialEq)]
#[config(context = Context)]
pub struct ProjectsConfig {
    #[setting(validate = test_list)]
    list: Vec<String>,
    #[setting(validate = test_map)]
    map: HashMap<String, String>,
}

#[derive(Debug, Config, Eq, PartialEq)]
#[config(context = Context, serde(untagged))]
pub enum Projects {
    #[setting(nested, validate = test_cfg)]
    Config(ProjectsConfig),
    #[setting(merge = merge::prepend_vec, validate = test_list)]
    List(Vec<String>),
    #[setting(default, merge = merge::merge_hashmap, validate = test_map)]
    Map(HashMap<String, String>),
}

#[derive(Debug, Config, Eq, PartialEq)]
#[config(context = Context)]
pub struct StandardSettings {
    #[setting(nested)]
    projects: Projects,
}

#[test]
#[serial]
fn returns_defaults() {
    let config = ConfigLoader::<StandardSettings>::new()
        .load()
        .unwrap()
        .config;

    assert_eq!(config.projects, Projects::Map(HashMap::new()));
}

mod loading {
    use super::*;

    #[test]
    fn loads_list() {
        let config = ConfigLoader::<StandardSettings>::new()
            .code(
                r#"
{
	"projects": ["foo", "bar", "baz"]
}"#,
                "code.json",
            )
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(
            config.projects,
            Projects::List(vec!["foo".into(), "bar".into(), "baz".into()])
        );
    }

    #[test]
    fn loads_map() {
        let config = ConfigLoader::<StandardSettings>::new()
            .code(
                r#"
{
	"projects": {
		"foo": "bar"
	}
}"#,
                "code.json",
            )
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(
            config.projects,
            Projects::Map(HashMap::from_iter([("foo".into(), "bar".into())]))
        );
    }

    #[test]
    fn loads_config() {
        let config = ConfigLoader::<StandardSettings>::new()
            .code(
                r#"
{
	"projects": {
		"list": ["baz"],
		"map": {
			"foo": "bar"
		}
	}
}"#,
                "code.json",
            )
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(
            config.projects,
            Projects::Config(ProjectsConfig {
                list: vec!["baz".into()],
                map: HashMap::from_iter([("foo".into(), "bar".into())])
            })
        );
    }
}

mod merging {
    use super::*;

    #[test]
    fn merges_list() {
        let config = ConfigLoader::<StandardSettings>::new()
            .code(
                r#"
{
	"projects": ["foo", "bar"]
}"#,
                "code.json",
            )
            .unwrap()
            .code(
                r#"
{
	"projects": ["baz", "qux"]
}"#,
                "code.json",
            )
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(
            config.projects,
            Projects::List(vec!["baz".into(), "qux".into(), "foo".into(), "bar".into()])
        );
    }

    #[test]
    fn merges_map() {
        let config = ConfigLoader::<StandardSettings>::new()
            .code(
                r#"
{
	"projects": {
		"foo": "bar"
	}
}"#,
                "code.json",
            )
            .unwrap()
            .code(
                r#"
{
	"projects": {
		"baz": "qux"
	}
}"#,
                "code.json",
            )
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(
            config.projects,
            Projects::Map(HashMap::from_iter([
                ("foo".into(), "bar".into()),
                ("baz".into(), "qux".into())
            ]))
        );
    }

    #[test]
    fn replaces_config() {
        let config = ConfigLoader::<StandardSettings>::new()
            .code(
                r#"
{
	"projects": {
		"list": ["qux", "bar"],
		"map": {
			"baz": "foo"
		}
	}
}"#,
                "code.json",
            )
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(
            config.projects,
            Projects::Config(ProjectsConfig {
                list: vec!["qux".into(), "bar".into()],
                map: HashMap::from_iter([("baz".into(), "foo".into())])
            })
        );
    }
}

mod validating {
    use super::*;

    #[test]
    fn validates_list() {
        let error = ConfigLoader::<StandardSettings>::new()
            .code(
                r#"
{
	"projects": ["foo", "bar", "baz"]
}"#,
                "code.json",
            )
            .unwrap()
            .load_with_context(&Context { fail: true })
            .err()
            .unwrap();

        assert_validation_error(error, 1);
    }

    #[test]
    fn validates_map() {
        let error = ConfigLoader::<StandardSettings>::new()
            .code(
                r#"
{
	"projects": {
		"foo": "bar"
	}
}"#,
                "code.json",
            )
            .unwrap()
            .load_with_context(&Context { fail: true })
            .err()
            .unwrap();

        assert_validation_error(error, 1);
    }

    #[test]
    fn validates_config() {
        let error = ConfigLoader::<StandardSettings>::new()
            .code(
                r#"
{
	"projects": {
		"list": ["baz"],
		"map": {
			"foo": "bar"
		}
	}
}"#,
                "code.json",
            )
            .unwrap()
            .load_with_context(&Context { fail: true })
            .err()
            .unwrap();

        // Also validates nested fields
        assert_validation_error(error, 3);
    }
}

#[cfg(feature = "renderer_json_schema")]
#[test]
fn generates_json_schema() {
    use starbase_sandbox::{assert_snapshot, create_empty_sandbox};

    let sandbox = create_empty_sandbox();
    let file = sandbox.path().join("schema.json");

    let mut generator = schema::SchemaGenerator::default();
    generator.add::<StandardSettings>();
    generator
        .generate(&file, schema::json_schema::JsonSchemaRenderer::default())
        .unwrap();

    assert!(file.exists());
    assert_snapshot!(std::fs::read_to_string(file).unwrap());
}

#[cfg(feature = "renderer_typescript")]
#[test]
fn generates_typescript() {
    use starbase_sandbox::{assert_snapshot, create_empty_sandbox};

    let sandbox = create_empty_sandbox();
    let file = sandbox.path().join("config.ts");

    let mut generator = schema::SchemaGenerator::default();
    generator.add::<StandardSettings>();
    generator
        .generate(&file, schema::typescript::TypeScriptRenderer::default())
        .unwrap();

    assert!(file.exists());
    assert_snapshot!(std::fs::read_to_string(file).unwrap());
}
