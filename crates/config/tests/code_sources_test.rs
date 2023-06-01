use schematic::*;

#[derive(Debug, Config)]
pub struct Config {
    boolean: bool,
    string: String,
    number: usize,
    vector: Vec<String>,
}

#[test]
fn can_create_code_source() {
    let source = Source::new("string: foo", None).unwrap();

    assert_eq!(
        source,
        Source::Code {
            code: "string: foo".to_owned(),
        }
    );
}

#[test]
fn handles_one_layer() {
    let a = r"
string: foo
";

    let result = ConfigLoader::<Config>::new()
        .code(a)
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
        .code(a)
        .unwrap()
        .code(b)
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
        .code(a)
        .unwrap()
        .code(b)
        .unwrap()
        .code(c)
        .unwrap()
        .code(d)
        .unwrap()
        .code(e)
        .unwrap()
        .load()
        .unwrap();

    assert!(!result.config.boolean);
    assert_eq!(result.config.string, "bar");
    assert_eq!(result.config.number, 123);
    assert_eq!(result.config.vector, vec!["x", "y", "z"]);
}
