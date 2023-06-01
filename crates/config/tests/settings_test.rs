use std::collections::HashMap;

use schematic::*;

#[derive(Default)]
pub struct Context {
    fail: bool,
}

#[derive(Debug, Config, Eq, PartialEq)]
#[config(context = Context)]
pub struct StandardSettings {
    req: String,
    #[setting(default = "abc")]
    req_default: String,
    opt: Option<String>,
}

mod standard {
    use super::*;

    #[test]
    fn returns_defaults() {
        let config = ConfigLoader::<StandardSettings>::json()
            .load()
            .unwrap()
            .config;

        assert_eq!(config.req, "");
        assert_eq!(config.req_default, "abc");
        assert_eq!(config.opt, None);
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
    fn returns_defaults() {
        let config = ConfigLoader::<NestedSettings>::json()
            .load()
            .unwrap()
            .config;

        assert_eq!(config.nested_req.req, "");
        assert_eq!(config.nested_req.req_default, "abc");
        assert_eq!(config.nested_req.opt, None);
        assert!(config.nested_opt.is_none());
    }

    #[test]
    fn applies_defaults_for_optional() {
        let config = ConfigLoader::<NestedSettings>::json()
            .code(r#"{ "nestedOpt": { "req": "xyz" } }"#)
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
    fn returns_defaults() {
        let config = ConfigLoader::<NestedVecSettings>::json()
            .load()
            .unwrap()
            .config;

        assert_eq!(config.nested_req, Vec::<StandardSettings>::new());
        assert!(config.nested_opt.is_none());
    }

    #[test]
    fn applies_defaults_for_items() {
        let config = ConfigLoader::<NestedVecSettings>::json()
            .code(
                r#"
{
	"nestedReq": [{ "req": "xyz" }],
	"nestedOpt": [{ "opt": "hij" }]
}"#,
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
    fn returns_defaults() {
        let config = ConfigLoader::<NestedMapSettings>::json()
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
    fn applies_defaults_for_items() {
        let config = ConfigLoader::<NestedMapSettings>::json()
            .code(
                r#"
{
	"nestedReq": { "key": { "req": "xyz" } },
	"nestedOpt": { "key": { "opt": "hij" } }
}"#,
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
}
