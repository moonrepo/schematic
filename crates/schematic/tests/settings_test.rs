use schematic::*;
use serial_test::serial;
use std::collections::HashMap;
use std::env;

fn test_string<T>(_: &String, _: &T, context: &Context) -> Result<(), ValidateError> {
    if context.fail {
        return Err(ValidateError::new("invalid"));
    }

    Ok(())
}

fn assert_validation_error(error: ConfigError, count: usize) {
    if let ConfigError::Validator { error: inner, .. } = error {
        assert_eq!(inner.len(), count);
    } else {
        panic!("expected validation error");
    }
}

#[derive(Default)]
pub struct Context {
    fail: bool,
}

#[derive(Debug, Config, Eq, PartialEq)]
#[config(context = Context)]
pub struct StandardSettings {
    #[setting(validate = test_string)]
    req: String,
    #[setting(default = "abc", validate = test_string)]
    req_default: String,
    #[setting(env = "OPT_ENV", validate = test_string)]
    opt: Option<String>,
}

mod standard {
    use super::*;

    #[test]
    #[serial]
    fn returns_defaults() {
        let config = ConfigLoader::<StandardSettings>::new()
            .load()
            .unwrap()
            .config;

        assert_eq!(config.req, "");
        assert_eq!(config.req_default, "abc");
        assert_eq!(config.opt, None);
    }

    #[test]
    #[serial]
    fn inherits_env() {
        env::set_var("OPT_ENV", "env");

        let config = ConfigLoader::<StandardSettings>::new()
            .load()
            .unwrap()
            .config;

        assert_eq!(config.opt, Some("env".to_owned()));

        env::remove_var("OPT_ENV");
    }

    #[test]
    #[serial]
    fn validates_all() {
        let error = ConfigLoader::<StandardSettings>::new()
            .load_with_context(&Context { fail: true })
            .err()
            .unwrap();

        assert_validation_error(error, 2);
    }
}

#[derive(Config)]
#[config(context = Context)]
pub struct NestedSettings {
    #[setting(nested)]
    nested_req: StandardSettings,
    #[setting(nested)]
    nested_opt: Option<StandardSettings>,
}

mod nested {
    use super::*;

    #[test]
    #[serial]
    fn returns_defaults() {
        let config = ConfigLoader::<NestedSettings>::new().load().unwrap().config;

        assert_eq!(config.nested_req.req, "");
        assert_eq!(config.nested_req.req_default, "abc");
        assert_eq!(config.nested_req.opt, None);
        assert!(config.nested_opt.is_none());
    }

    #[test]
    #[serial]
    fn applies_defaults_for_optional() {
        let config = ConfigLoader::<NestedSettings>::new()
            .code(r#"{ "nestedOpt": { "req": "xyz" } }"#, Format::Json)
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(config.nested_req.req, "");
        assert_eq!(config.nested_req.req_default, "abc");

        let opt = config.nested_opt.unwrap();

        assert_eq!(opt.req, "xyz");
        assert_eq!(opt.req_default, "abc");
    }

    #[test]
    #[serial]
    fn inherits_env_for_each() {
        env::set_var("OPT_ENV", "env");

        let config = ConfigLoader::<NestedSettings>::new()
            .code(r#"{ "nestedOpt": { "req": "xyz" } }"#, Format::Json)
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(config.nested_req.opt, Some("env".to_owned()));
        assert_eq!(config.nested_opt.unwrap().opt, Some("env".to_owned()));

        env::remove_var("OPT_ENV");
    }

    #[test]
    #[serial]
    fn validates_all() {
        let error = ConfigLoader::<NestedSettings>::new()
            .code(r#"{ "nestedOpt": { "req": "xyz" } }"#, Format::Json)
            .unwrap()
            .load_with_context(&Context { fail: true })
            .err()
            .unwrap();

        // 1 instead of 3 since its running on a partial
        assert_validation_error(error, 1);
    }
}

#[derive(Config)]
#[config(context = Context)]
pub struct NestedVecSettings {
    #[setting(nested)]
    nested_req: Vec<StandardSettings>,
    #[setting(nested)]
    nested_opt: Option<Vec<StandardSettings>>,
}

mod nested_vec {
    use super::*;

    #[test]
    #[serial]
    fn returns_defaults() {
        let config = ConfigLoader::<NestedVecSettings>::new()
            .load()
            .unwrap()
            .config;

        assert_eq!(config.nested_req, Vec::<StandardSettings>::new());
        assert!(config.nested_opt.is_none());
    }

    #[test]
    #[serial]
    fn applies_defaults_for_items() {
        let config = ConfigLoader::<NestedVecSettings>::new()
            .code(
                r#"
{
	"nestedReq": [{ "req": "xyz" }],
	"nestedOpt": [{ "opt": "hij" }]
}"#,
                Format::Json,
            )
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(
            config.nested_req,
            vec![StandardSettings {
                req: "xyz".into(),
                req_default: "abc".into(),
                opt: None,
            }]
        );

        assert_eq!(
            config.nested_opt.unwrap(),
            vec![StandardSettings {
                req: "".into(),
                req_default: "abc".into(),
                opt: Some("hij".into()),
            }]
        );
    }

    #[test]
    #[serial]
    fn inherits_env_for_each() {
        env::set_var("OPT_ENV", "env");

        let config = ConfigLoader::<NestedVecSettings>::new()
            .code(
                r#"
{
	"nestedReq": [{ "req": "xyz" }],
	"nestedOpt": [{ "opt": "hij" }]
}"#,
                Format::Json,
            )
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(config.nested_req[0].opt, Some("env".to_owned()));
        assert_eq!(config.nested_opt.unwrap()[0].opt, Some("env".to_owned()));

        env::remove_var("OPT_ENV");
    }

    #[test]
    #[serial]
    fn validates_all() {
        let error = ConfigLoader::<NestedVecSettings>::new()
            .code(
                r#"
{
	"nestedReq": [{ "req": "1" }, { "req": "2" }],
	"nestedOpt": [{ "opt": "3" }]
}"#,
                Format::Json,
            )
            .unwrap()
            .load_with_context(&Context { fail: true })
            .err()
            .unwrap();

        // 3 instead of 6 since its running on a partial
        assert_validation_error(error, 3);
    }
}

#[derive(Config)]
#[config(context = Context)]
pub struct NestedMapSettings {
    #[setting(nested)]
    nested_req: HashMap<String, StandardSettings>,
    #[setting(nested)]
    nested_opt: Option<HashMap<String, StandardSettings>>,
}

mod nested_map {
    use super::*;

    #[test]
    #[serial]
    fn returns_defaults() {
        let config = ConfigLoader::<NestedMapSettings>::new()
            .load()
            .unwrap()
            .config;

        assert_eq!(
            config.nested_req,
            HashMap::<String, StandardSettings>::new()
        );
        assert!(config.nested_opt.is_none());
    }

    #[test]
    #[serial]
    fn applies_defaults_for_items() {
        let config = ConfigLoader::<NestedMapSettings>::new()
            .code(
                r#"
{
	"nestedReq": { "key": { "req": "xyz" } },
	"nestedOpt": { "key": { "opt": "hij" } }
}"#,
                Format::Json,
            )
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(
            config.nested_req,
            HashMap::from_iter([(
                "key".into(),
                StandardSettings {
                    req: "xyz".into(),
                    req_default: "abc".into(),
                    opt: None,
                }
            )])
        );

        assert_eq!(
            config.nested_opt.unwrap(),
            HashMap::from_iter([(
                "key".into(),
                StandardSettings {
                    req: "".into(),
                    req_default: "abc".into(),
                    opt: Some("hij".into()),
                }
            )])
        );
    }

    #[test]
    #[serial]
    fn inherits_env_for_each() {
        env::set_var("OPT_ENV", "env");

        let config = ConfigLoader::<NestedMapSettings>::new()
            .code(
                r#"
{
	"nestedReq": { "key": { "req": "xyz" } },
	"nestedOpt": { "key": { "opt": "hij" } }
}"#,
                Format::Json,
            )
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(
            config.nested_req.get("key").unwrap().opt,
            Some("env".to_owned())
        );
        assert_eq!(
            config.nested_opt.unwrap().get("key").unwrap().opt,
            Some("env".to_owned())
        );

        env::remove_var("OPT_ENV");
    }

    #[test]
    #[serial]
    fn validates_all() {
        let error = ConfigLoader::<NestedMapSettings>::new()
            .code(
                r#"
{
	"nestedReq": { "key1": { "req": "xyz" } },
	"nestedOpt": { "key2": { "opt": "hij" }, "key3": { "opt": "abc" } }
}"#,
                Format::Json,
            )
            .unwrap()
            .load_with_context(&Context { fail: true })
            .err()
            .unwrap();

        // 3 instead of 6 since its running on a partial
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
    generator.add::<NestedSettings>();
    generator.add::<NestedVecSettings>();
    generator.add::<NestedMapSettings>();
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
    generator.add::<NestedSettings>();
    generator.add::<NestedVecSettings>();
    generator.add::<NestedMapSettings>();
    generator
        .generate(&file, schema::typescript::TypeScriptRenderer::default())
        .unwrap();

    assert!(file.exists());
    assert_snapshot!(std::fs::read_to_string(file).unwrap());
}
