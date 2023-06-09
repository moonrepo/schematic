use schematic::*;

#[derive(Debug, Config)]
pub struct Config {
    boolean: bool,
    string: String,
    number: usize,
    vector: Vec<String>,
}

// #[test]
// fn can_create_code_source() {
//     let source = Source::new("string: foo", None).unwrap();

//     assert_eq!(
//         source,
//         Source::Code {
//             code: "string: foo".to_owned(),
//             format: Format::Yaml,
//         }
//     );
// }

#[test]
fn handles_one_layer() {
    let a = r"
string: foo
";

    let result = ConfigLoader::<Config>::new()
        .code(a, Format::Yaml)
        .unwrap()
        .load()
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "foo");
    assert_eq!(result.config.number, 0);
    assert_eq!(result.config.vector, Vec::<String>::new());
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
        .code(a, Format::Yaml)
        .unwrap()
        .code(b, Format::Yaml)
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
        .code(a, Format::Yaml)
        .unwrap()
        .code(b, Format::Yaml)
        .unwrap()
        .code(c, Format::Yaml)
        .unwrap()
        .code(d, Format::Yaml)
        .unwrap()
        .code(e, Format::Yaml)
        .unwrap()
        .load()
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "bar");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.vector, vec!["x", "y", "z"]);
}

#[cfg(feature = "typescript")]
#[test]
fn generates_typescript() {
    use starbase_sandbox::{assert_snapshot, create_empty_sandbox};

    let sandbox = create_empty_sandbox();
    let file = sandbox.path().join("config.ts");

    let mut generator = typescript::TypeScriptGenerator::new(file.clone());
    generator.add::<Config>().unwrap();
    generator.generate().unwrap();

    assert!(file.exists());
    assert_snapshot!(std::fs::read_to_string(file).unwrap());
}
