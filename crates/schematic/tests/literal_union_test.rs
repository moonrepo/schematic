#![allow(dead_code)]

// A literal union mixes named values with a value of another type, like
// `"local" | "remote" | boolean`. Named values are unit variants, and the
// variant holding a value is marked `#[serde(untagged)]`.

use schematic::*;

/// When to cache.
#[derive(Clone, Config, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Cache {
    Local,
    #[setting(default)]
    Remote,
    #[serde(untagged)]
    Enabled(bool),
}

derive_enum!(
    /// How many threads to use.
    #[derive(ConfigEnum, Default)]
    pub enum Threads {
        #[default]
        Auto,
        Off,
        #[serde(untagged)]
        Count(u32),
    }
);

#[derive(Clone, Config, Debug)]
pub struct LiteralConfig {
    #[setting(nested)]
    cache: Cache,
    #[setting(nested)]
    cache_opt: Option<Cache>,
    threads: Threads,
    threads_opt: Option<Threads>,
}

fn load(code: &str, file: &str) -> Result<LiteralConfig, String> {
    ConfigLoader::<LiteralConfig>::new()
        .code(code, file)
        .unwrap()
        .load()
        .map(|result| result.config)
        .map_err(|error| error.to_full_string())
}

#[test]
fn uses_the_default_variants() {
    let config = load("{}", "code.json").unwrap();

    assert_eq!(config.cache, Cache::Remote);
    assert_eq!(config.cache_opt, None);
    assert_eq!(config.threads, Threads::Auto);
    assert_eq!(config.threads_opt, None);
}

#[cfg(feature = "json")]
mod json {
    use super::*;

    #[test]
    fn loads_named_values() {
        let config = load(
            r#"{ "cache": "local", "cache_opt": "remote", "threads": "off" }"#,
            "code.json",
        )
        .unwrap();

        assert_eq!(config.cache, Cache::Local);
        assert_eq!(config.cache_opt, Some(Cache::Remote));
        assert_eq!(config.threads, Threads::Off);
    }

    #[test]
    fn loads_untagged_values() {
        let config = load(
            r#"{ "cache": false, "cache_opt": true, "threads": 8, "threads_opt": 2 }"#,
            "code.json",
        )
        .unwrap();

        assert_eq!(config.cache, Cache::Enabled(false));
        assert_eq!(config.cache_opt, Some(Cache::Enabled(true)));
        assert_eq!(config.threads, Threads::Count(8));
        assert_eq!(config.threads_opt, Some(Threads::Count(2)));
    }
}

#[cfg(feature = "toml")]
mod toml {
    use super::*;

    #[test]
    fn loads_named_values() {
        let config = load("cache = \"local\"\nthreads = \"off\"", "code.toml").unwrap();

        assert_eq!(config.cache, Cache::Local);
        assert_eq!(config.threads, Threads::Off);
    }

    #[test]
    fn loads_untagged_values() {
        let config = load("cache = true\nthreads = 8", "code.toml").unwrap();

        assert_eq!(config.cache, Cache::Enabled(true));
        assert_eq!(config.threads, Threads::Count(8));
    }

    #[test]
    fn lists_the_named_values_on_failure() {
        assert_eq!(
            load("cache = 1", "code.toml").unwrap_err(),
            "Failed to parse LiteralConfig. cache: failed to parse as any variant of PartialCache:\n- expected one of \"local\", \"remote\", found 1\n- enabled: expected a boolean, found 1"
        );
    }
}

#[cfg(feature = "yaml")]
mod yaml {
    use super::*;

    #[test]
    fn loads_named_values() {
        let config = load("cache: local\nthreads: off", "code.yml").unwrap();

        assert_eq!(config.cache, Cache::Local);
        assert_eq!(config.threads, Threads::Off);
    }

    #[test]
    fn loads_untagged_values() {
        let config = load("cache: true\nthreads: 8", "code.yml").unwrap();

        assert_eq!(config.cache, Cache::Enabled(true));
        assert_eq!(config.threads, Threads::Count(8));
    }

    #[test]
    fn lists_the_named_values_on_failure() {
        assert_eq!(
            load("cache: qux", "code.yml").unwrap_err(),
            "Failed to parse LiteralConfig. cache: failed to parse as any variant of PartialCache:\n- expected one of \"local\", \"remote\", found \"qux\"\n- enabled: expected a boolean, found \"qux\""
        );
    }
}

mod renderers {
    use super::*;
    use schematic::schema::*;
    use starbase_sandbox::{assert_snapshot, create_empty_sandbox};
    use std::fs;

    fn render<O, R: SchemaRenderer<O>>(file_name: &str, renderer: R) -> String {
        let sandbox = create_empty_sandbox();
        let file = sandbox.path().join(file_name);

        let mut generator = SchemaGenerator::default();
        generator.add::<LiteralConfig>();
        generator.generate(&file, renderer).unwrap();

        fs::read_to_string(file).unwrap()
    }

    #[cfg(feature = "renderer_api_docs")]
    #[test]
    fn api_docs() {
        assert_snapshot!(render("docs.md", ApiDocsRenderer::default()));
    }

    #[cfg(feature = "renderer_json_schema")]
    #[test]
    fn json_schema() {
        assert_snapshot!(render("schema.json", JsonSchemaRenderer::default()));
    }

    #[cfg(feature = "renderer_pkl_schema")]
    #[test]
    fn pkl_schema() {
        assert_snapshot!(render("module.pkl", PklSchemaRenderer::default()));
    }

    #[cfg(feature = "renderer_typescript")]
    #[test]
    fn typescript() {
        assert_snapshot!(render("types.ts", TypeScriptRenderer::default()));
    }

    #[cfg(all(feature = "renderer_template", feature = "json"))]
    #[test]
    fn jsonc_template() {
        assert_snapshot!(render(
            "config.jsonc",
            JsoncTemplateRenderer::new(TemplateOptions::default())
        ));
    }

    #[cfg(all(feature = "renderer_template", feature = "toml"))]
    #[test]
    fn toml_template() {
        assert_snapshot!(render(
            "config.toml",
            TomlTemplateRenderer::new(TemplateOptions::default())
        ));
    }

    #[cfg(all(feature = "renderer_template", feature = "yaml"))]
    #[test]
    fn yaml_template() {
        assert_snapshot!(render(
            "config.yml",
            YamlTemplateRenderer::new(TemplateOptions::default())
        ));
    }
}
