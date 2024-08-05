#[cfg(feature = "pkl")]
mod pkl {
    use schematic::*;
    use starbase_sandbox::locate_fixture;

    #[derive(Config)]
    struct VarConfig {
        list: Vec<String>,
    }

    #[test]
    fn supports_variables() {
        let root = locate_fixture("pkl");

        let result = ConfigLoader::<VarConfig>::new()
            .file(root.join("variables.pkl"))
            .unwrap()
            .load()
            .unwrap();

        assert_eq!(result.config.list, vec!["a", "b", "c"]);
    }
}
