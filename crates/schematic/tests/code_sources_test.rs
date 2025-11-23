use schematic::*;

#[derive(Debug, Config)]
pub struct Config {
    boolean: bool,
    string: String,
    number: usize,
    vector: Vec<String>,
    float: f32,
}

// #[test]
// fn can_create_code_source() {
//     let source = Source::new("string: foo", None).unwrap();

//     assert_eq!(
//         source,
//         Source::Code {
//             code: "string: foo".to_owned(),
//             format: "code.yaml",
//         }
//     );
// }

#[test]
fn supports_bom() {
    let a = "\u{feff}---\nstring: foo";

    let result = ConfigLoader::<Config>::new()
        .code(a, "code.yaml")
        .unwrap()
        .load()
        .unwrap();

    assert_eq!(result.config.string, "foo");
}

#[test]
fn handles_one_layer() {
    let a = r"
string: foo
float: 1.23
";

    let result = ConfigLoader::<Config>::new()
        .code(a, "code.yaml")
        .unwrap()
        .load()
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "foo");
    assert_eq!(result.config.number, 0);
    assert_eq!(result.config.vector, Vec::<String>::new());
    assert_eq!(result.config.float, 1.23);
}

#[test]
fn merges_two_layers() {
    let a = r"
string: foo
";
    let b = r"
vector: [a, b, c]
";

    let result = ConfigLoader::<Config>::new()
        .code(a, "code.yaml")
        .unwrap()
        .code(b, "code.yaml")
        .unwrap()
        .load()
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "foo");
    assert_eq!(result.config.number, 0);
    assert_eq!(result.config.vector, vec!["a", "b", "c"]);
}

#[test]
fn merges_many_layers() {
    let a = r"
string: foo
";
    let b = r"
vector: [a, b, c]
";
    let c = r"
boolean: true
string: bar
";
    let d = r"
boolean: false
number: 123
";
    let e = r"
vector: [x, y, z]
";

    let result = ConfigLoader::<Config>::new()
        .code(a, "code.yaml")
        .unwrap()
        .code(b, "code.yaml")
        .unwrap()
        .code(c, "code.yaml")
        .unwrap()
        .code(d, "code.yaml")
        .unwrap()
        .code(e, "code.yaml")
        .unwrap()
        .load()
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "bar");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.vector, vec!["x", "y", "z"]);
}

#[cfg(feature = "renderer_json_schema")]
#[test]
fn generates_json_schema() {
    use starbase_sandbox::{assert_snapshot, create_empty_sandbox};

    let sandbox = create_empty_sandbox();
    let file = sandbox.path().join("schema.json");

    let mut generator = schema::SchemaGenerator::default();
    generator.add::<Config>();
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
    generator.add::<Config>();
    generator
        .generate(&file, schema::typescript::TypeScriptRenderer::default())
        .unwrap();

    assert!(file.exists());
    assert_snapshot!(std::fs::read_to_string(file).unwrap());
}
