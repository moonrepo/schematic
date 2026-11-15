use schematic::*;

fn list(items: &[&str]) -> MergeList<String> {
    MergeList::new(items.iter().map(|item| item.to_string()).collect())
}

fn append(items: &[&str]) -> MergeList<String> {
    MergeList::append(items.iter().map(|item| item.to_string()).collect())
}

fn prepend(items: &[&str]) -> MergeList<String> {
    MergeList::prepend(items.iter().map(|item| item.to_string()).collect())
}

fn json(value: &str) -> Result<MergeList<String>, serde_json::Error> {
    serde_json::from_str(value)
}

mod merging {
    use super::*;

    #[test]
    fn replaces_with_a_list() {
        assert_eq!(list(&["a"]).merge(list(&["b"])), list(&["b"]));
        assert_eq!(append(&["a"]).merge(list(&["b"])), list(&["b"]));
    }

    #[test]
    fn adds_around_the_previous_items() {
        assert_eq!(*list(&["a"]).merge(append(&["b"])), ["a", "b"]);
        assert_eq!(*list(&["a"]).merge(prepend(&["b"])), ["b", "a"]);
        assert_eq!(
            *list(&["a"]).merge(json(r#"{ "prepend": ["p"], "append": ["z"] }"#).unwrap()),
            ["p", "a", "z"]
        );
    }

    #[test]
    fn stays_relative_until_merged_into_a_list() {
        let added = append(&["b"]).merge(prepend(&["a"])).merge(append(&["c"]));

        assert!(added.is_relative());
        assert_eq!(*added, ["a", "b", "c"]);

        let merged = list(&["x"]).merge(added);

        assert!(!merged.is_relative());
        assert_eq!(*merged, ["a", "x", "b", "c"]);
    }

    // Layers are merged into each other before the result is merged over
    // the defaults, which only gives the same result if merging is associative
    #[test]
    fn is_associative() {
        let lists = || {
            [
                list(&["l1", "l2"]),
                append(&["a1", "a2"]),
                prepend(&["p1", "p2"]),
                json(r#"{ "prepend": ["p3"], "append": ["a3"] }"#).unwrap(),
                json(r#"{ "prepend": ["p4"], "replace": ["r4"], "append": ["a4"] }"#).unwrap(),
                MergeList::default(),
                append(&[]),
            ]
        };

        for a in lists() {
            for b in lists() {
                for c in lists() {
                    assert_eq!(
                        a.clone().merge(b.clone()).merge(c.clone()),
                        a.clone().merge(b.clone().merge(c.clone())),
                        "{a:?} {b:?} {c:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn merge_fn_merges() {
        assert_eq!(
            merge::merge_list(list(&["a"]), append(&["b"]), &())
                .unwrap()
                .unwrap()
                .into_vec(),
            vec!["a", "b"]
        );
    }
}

mod serialization {
    use super::*;

    #[test]
    fn parses_a_list() {
        assert_eq!(json(r#"["a", "b"]"#).unwrap(), list(&["a", "b"]));
    }

    #[test]
    fn parses_a_map() {
        assert_eq!(json(r#"{ "append": ["a"] }"#).unwrap(), append(&["a"]));
        assert_eq!(json(r#"{ "prepend": ["a"] }"#).unwrap(), prepend(&["a"]));
        assert_eq!(json(r#"{ "replace": ["a"] }"#).unwrap(), list(&["a"]));
        assert_eq!(
            json(r#"{ "replace": ["r"], "append": ["a"], "prepend": ["p"] }"#).unwrap(),
            list(&["p", "r", "a"])
        );
        assert_eq!(json("{}").unwrap(), append(&[]));
    }

    #[test]
    fn errors_for_an_unknown_key() {
        assert_eq!(
            json(r#"{ "push": ["a"] }"#).unwrap_err().to_string(),
            "unknown field `push`, expected one of `append`, `prepend`, `replace` at line 1 column 8"
        );
    }

    #[test]
    fn errors_for_a_duplicate_key() {
        assert_eq!(
            json(r#"{ "append": ["a"], "append": ["b"] }"#)
                .unwrap_err()
                .to_string(),
            "duplicate field `append` at line 1 column 27"
        );
    }

    #[test]
    fn errors_for_another_type() {
        assert_eq!(
            json("5").unwrap_err().to_string(),
            "invalid type: integer `5`, expected a list, or a map of items to append, prepend, or replace at line 1 column 1"
        );
    }

    #[test]
    fn serializes_by_how_it_merges() {
        let serialize = |value: &MergeList<String>| serde_json::to_string(value).unwrap();

        assert_eq!(serialize(&list(&["a"])), r#"["a"]"#);
        assert_eq!(serialize(&append(&["a"])), r#"{"append":["a"]}"#);
        assert_eq!(serialize(&prepend(&["a"])), r#"{"prepend":["a"]}"#);
        assert_eq!(serialize(&append(&[])), "{}");

        let both = list(&["x"]).merge(json(r#"{ "prepend": ["p"], "append": ["a"] }"#).unwrap());
        assert_eq!(serialize(&both), r#"["p","x","a"]"#);

        let relative = json(r#"{ "prepend": ["p"], "append": ["a"] }"#).unwrap();
        assert_eq!(serialize(&relative), r#"{"prepend":["p"],"append":["a"]}"#);
        assert_eq!(json(&serialize(&relative)).unwrap(), relative);
    }
}

mod loading {
    use super::*;

    // Merged with `merge_list` without saying so, except `ports`, which
    // names it, as it still may
    #[derive(Debug, Config)]
    struct App {
        #[setting(default = MergeList::new(vec!["default".to_owned()]))]
        hosts: MergeList<String>,

        #[setting(merge = merge::merge_list, validate = validate::min_length(1))]
        ports: MergeList<u16>,

        optional: Option<MergeList<String>>,

        #[setting(merge = merge::replace)]
        replaced: MergeList<String>,

        #[setting(nested)]
        mode: Mode,
    }

    #[derive(Debug, Config)]
    enum Mode {
        #[setting(default)]
        Off,
        On(MergeList<String>, bool),
    }

    fn load(sources: &[&str]) -> Result<App, ConfigError> {
        let mut loader = ConfigLoader::<App>::new();

        for (index, source) in sources.iter().enumerate() {
            loader.code(*source, format!("{index}.yml")).unwrap();
        }

        loader.load().map(|result| result.config)
    }

    #[test]
    fn each_layer_chooses_how_to_merge() {
        let config = load(&[
            "hosts: [a, b]\nports: [1]",
            "hosts: { append: [c] }\nports: { prepend: [0] }",
            "hosts: { prepend: [z] }",
        ])
        .unwrap();

        assert_eq!(*config.hosts, ["z", "a", "b", "c"]);
        assert_eq!(*config.ports, [0, 1]);
    }

    #[test]
    fn adds_to_the_defaults() {
        let config = load(&["hosts: { append: [a] }\nports: [1]"]).unwrap();

        assert_eq!(*config.hosts, ["default", "a"]);
    }

    #[test]
    fn replaces_the_defaults() {
        let config = load(&["hosts: [a]\nports: [1]"]).unwrap();

        assert_eq!(*config.hosts, ["a"]);
    }

    #[test]
    fn adds_to_nothing_without_a_default() {
        let config = load(&["ports: { append: [1, 2] }"]).unwrap();

        assert_eq!(*config.ports, [1, 2]);
        assert_eq!(config.ports.iter().sum::<u16>(), 3);
    }

    #[test]
    fn infers_the_merge_for_an_optional_list() {
        let config = load(&[
            "ports: [1]\noptional: { append: [a] }",
            "optional: { append: [b] }",
        ])
        .unwrap();

        assert_eq!(*config.optional.unwrap(), ["a", "b"]);
    }

    #[test]
    fn infers_the_merge_for_a_variant_value() {
        let config = ConfigLoader::<App>::new()
            .code(
                r#"{ "ports": [1], "mode": { "On": [["a"], true] } }"#,
                "a.json",
            )
            .unwrap()
            .code(
                r#"{ "mode": { "On": [{ "append": ["b"] }, false] } }"#,
                "b.json",
            )
            .unwrap()
            .load()
            .unwrap()
            .config;

        let Mode::On(list, flag) = config.mode else {
            panic!("expected On");
        };

        assert_eq!(*list, ["a", "b"]);
        assert!(!flag);
    }

    #[test]
    fn an_explicit_merge_wins() {
        let config = load(&["ports: [1]\nreplaced: [a]", "replaced: { append: [b] }"]).unwrap();

        assert_eq!(*config.replaced, ["b"]);
    }

    #[test]
    fn parses_any_format() {
        let config = ConfigLoader::<App>::new()
            .code("ports = [1]\nhosts = { append = [\"a\"] }", "a.toml")
            .unwrap()
            .code(r#"{ "hosts": { "prepend": ["z"] } }"#, "b.json")
            .unwrap()
            .load()
            .unwrap()
            .config;

        assert_eq!(*config.hosts, ["z", "default", "a"]);
    }

    #[test]
    fn validates_the_merged_items() {
        let error = load(&["ports: [1]", "ports: { replace: [] }"])
            .err()
            .unwrap();

        assert!(
            error
                .to_full_string()
                .contains("ports: length is lower than 1"),
            "{}",
            error.to_full_string()
        );
    }

    #[test]
    fn reports_where_a_map_is_wrong() {
        let error = load(&["ports: { push: [1] }"]).err().unwrap();

        assert!(
            error
                .to_full_string()
                .contains("unknown field `push`, expected one of `append`, `prepend`, `replace`"),
            "{}",
            error.to_full_string()
        );
    }
}

mod schema {
    use super::*;
    use schematic::schema::{JsonSchemaRenderer, SchemaGenerator, TypeScriptRenderer};

    #[derive(Config)]
    #[allow(dead_code)]
    struct App {
        hosts: MergeList<String>,
    }

    fn generator() -> SchemaGenerator {
        let mut generator = SchemaGenerator::default();
        generator.add::<App>();
        generator
    }

    #[test]
    fn renders_typescript() {
        let output = generator().render(TypeScriptRenderer::default()).unwrap();

        assert!(
            output.contains(
                "hosts: string[] | {\n\t\tappend?: string[];\n\t\tprepend?: string[];\n\t\treplace?: string[];\n\t};"
            ),
            "{output}"
        );
    }

    #[test]
    fn renders_json_schema() {
        let output = generator().render(JsonSchemaRenderer::default()).unwrap();

        starbase_sandbox::assert_snapshot!(output);
    }
}
