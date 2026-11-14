mod utils;

use schematic::*;
use utils::get_fixture_path;

#[derive(Debug, Config)]
struct ExtendsString {
    #[setting(extend)]
    extends: String,
    #[setting(merge = merge::append_vec)]
    value: Vec<usize>,
}

#[derive(Debug, Config)]
struct ExtendsStringOptional {
    #[setting(extend)]
    extends: Option<String>,
    #[setting(merge = merge::append_vec)]
    value: Vec<usize>,
}

#[derive(Config)]
struct ExtendsList {
    #[setting(extend)]
    extends: Vec<String>,
    #[setting(merge = merge::append_vec)]
    value: Vec<usize>,
}

#[derive(Config)]
struct ExtendsEnum {
    #[setting(extend)]
    extends: schematic::ExtendsFrom,
    #[setting(merge = merge::append_vec)]
    value: Vec<usize>,
}

#[test]
fn extends_from_chain_in_order() {
    let root = get_fixture_path("extending");

    let result = ConfigLoader::<ExtendsString>::new()
        .file(root.join("base.yml"))
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.extends, "");
    assert_eq!(result.config.value, vec![3, 2, 1]);

    assert_eq!(
        result
            .layers
            .into_iter()
            .map(|l| l.source)
            .collect::<Vec<_>>(),
        vec![
            // Source::Defaults,
            Source::File {
                path: root.join("./string2.yml"),
                required: true,
            },
            Source::File {
                path: root.join("./string1.yml"),
                required: true,
            },
            Source::File {
                path: root.join("./base.yml"),
                required: true,
            },
            // Source::EnvVars,
        ]
    );
}

#[test]
fn extends_from_chain_in_order_using_list() {
    let root = get_fixture_path("extending");

    let result = ConfigLoader::<ExtendsList>::new()
        .file(root.join("base-list.yml"))
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.extends, Vec::<String>::new());
    assert_eq!(result.config.value, vec![3, 2, 4, 1]);

    assert_eq!(
        result
            .layers
            .into_iter()
            .map(|l| l.source)
            .collect::<Vec<_>>(),
        vec![
            // Source::Defaults,
            Source::File {
                path: root.join("./string2.yml"),
                required: true,
            },
            Source::File {
                path: root.join("./list1.yml"),
                required: true,
            },
            Source::File {
                path: root.join("./list2.yml"),
                required: true,
            },
            Source::File {
                path: root.join("./base-list.yml"),
                required: true,
            },
            // Source::EnvVars,
        ]
    );
}

#[test]
fn extends_from_chain_in_order_using_both_enum() {
    let root = get_fixture_path("extending");

    let result = ConfigLoader::<ExtendsEnum>::new()
        .file(root.join("base-both.yml"))
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.extends, ExtendsFrom::default());

    // Both `list1.yml` and `string1.yml` extend `string2.yml`, which is only loaded once
    assert_eq!(result.config.value, vec![3, 2, 2, 4, 1]);

    assert_eq!(
        result
            .layers
            .into_iter()
            .map(|l| l.source)
            .collect::<Vec<_>>(),
        vec![
            // Source::Defaults,
            Source::File {
                path: root.join("./string2.yml"),
                required: true,
            },
            Source::File {
                path: root.join("./list1.yml"),
                required: true,
            },
            Source::File {
                path: root.join("./string1.yml"),
                required: true,
            },
            Source::File {
                path: root.join("list2.yml"),
                required: true,
            },
            Source::File {
                path: root.join("./base-both.yml"),
                required: true,
            },
            // Source::EnvVars,
        ]
    );
}

#[test]
fn extends_from_optional() {
    let root = get_fixture_path("extending");

    let result = ConfigLoader::<ExtendsStringOptional>::new()
        .file(root.join("string2.yml"))
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.extends, None);
    assert_eq!(result.config.value, vec![3]);

    assert_eq!(
        result
            .layers
            .into_iter()
            .map(|l| l.source)
            .collect::<Vec<_>>(),
        vec![
            // Source::Defaults,
            Source::File {
                path: root.join("./string2.yml"),
                required: true,
            },
            // Source::EnvVars,
        ]
    );
}

#[test]
fn errors_on_cycle() {
    let root = get_fixture_path("extending");

    let error = ConfigLoader::<ExtendsEnum>::new()
        .set_root(&root)
        .file(root.join("cycle-a.yml"))
        .unwrap()
        .load()
        .err()
        .unwrap();

    let ConfigError::ExtendsCycle(chain) = &error else {
        panic!("expected a cycle error, received {error:?}");
    };

    assert_eq!(
        chain,
        &["cycle-a.yml", "cycle-b.yml", "cycle-c.yml", "cycle-a.yml"]
    );
}

#[test]
fn errors_on_self_extend() {
    let sandbox = starbase_sandbox::create_empty_sandbox();
    sandbox.create_file("self.yml", "extends: ./self.yml\nvalue: [1]");

    let error = ConfigLoader::<ExtendsEnum>::new()
        .file(sandbox.path().join("self.yml"))
        .unwrap()
        .load()
        .err()
        .unwrap();

    assert!(matches!(error, ConfigError::ExtendsCycle(_)));
}

#[test]
fn loads_a_source_extended_twice_once() {
    let root = get_fixture_path("extending");

    let result = ConfigLoader::<ExtendsEnum>::new()
        .file(root.join("diamond.yml"))
        .unwrap()
        .load()
        .unwrap();

    // The base comes first, as the lowest precedence of both that extend it
    assert_eq!(result.config.value, vec![4, 2, 3, 1]);
    assert_eq!(result.layers.len(), 4);
}

#[test]
fn skips_an_extended_source_that_was_added_to_the_loader() {
    let root = get_fixture_path("extending");

    let result = ConfigLoader::<ExtendsEnum>::new()
        .file(root.join("diamond-base.yml"))
        .unwrap()
        .file(root.join("diamond-left.yml"))
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.value, vec![4, 2]);
}

#[cfg(feature = "renderer_json_schema")]
#[test]
fn generates_json_schema() {
    use starbase_sandbox::{assert_snapshot, create_empty_sandbox};

    let sandbox = create_empty_sandbox();
    let file = sandbox.path().join("schema.json");

    let mut generator = schema::SchemaGenerator::default();
    generator.add::<ExtendsString>();
    generator.add::<ExtendsStringOptional>();
    generator.add::<ExtendsList>();
    generator.add::<ExtendsEnum>();
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
    generator.add::<ExtendsString>();
    generator.add::<ExtendsStringOptional>();
    generator.add::<ExtendsList>();
    generator.add::<ExtendsEnum>();
    generator
        .generate(&file, schema::typescript::TypeScriptRenderer::default())
        .unwrap();

    assert!(file.exists());
    assert_snapshot!(std::fs::read_to_string(file).unwrap());
}
