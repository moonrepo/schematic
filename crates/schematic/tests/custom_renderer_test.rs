//! A custom renderer only implements the types it supports.
#![allow(dead_code)]

use schematic::schema::*;
use schematic::*;

#[derive(Schematic)]
pub struct Strings {
    a: String,
    b: String,
}

#[derive(Schematic)]
pub struct WithBool {
    a: String,
    b: bool,
}

/// Renders a struct of strings as `name: string` lines, and nothing else.
#[derive(Default)]
struct LinesRenderer;

impl SchemaRenderer for LinesRenderer {
    fn is_reference(&self, _name: &str) -> bool {
        false
    }

    fn render_string(&mut self, _string: &StringType, _schema: &Schema) -> RenderResult {
        Ok("string".into())
    }

    fn render_struct(&mut self, structure: &StructType, _schema: &Schema) -> RenderResult {
        let mut lines = vec![];

        for (name, field) in structure.sorted_fields() {
            lines.push(format!("{name}: {}", self.render_schema(&field.schema)?));
        }

        Ok(lines.join("\n"))
    }

    fn render(&mut self, schemas: indexmap::IndexMap<String, Schema>) -> RenderResult {
        let root = schemas.values().last().unwrap();

        self.render_schema(root)
    }
}

fn render<T: Schematic>() -> miette::Result<String> {
    let mut generator = SchemaGenerator::default();
    generator.add::<T>();
    generator.render(LinesRenderer)
}

#[test]
fn renders_the_types_it_implements() {
    assert_eq!(render::<Strings>().unwrap(), "a: string\nb: string");
}

#[test]
fn errors_for_a_type_it_doesnt() {
    let error = render::<WithBool>().unwrap_err().to_string();

    assert!(
        error.contains("does not support rendering a boolean"),
        "{error}"
    );
    assert!(error.contains("LinesRenderer"), "{error}");
}

#[test]
fn renders_to_a_string_like_it_generates_a_file() {
    let sandbox = starbase_sandbox::create_empty_sandbox();
    let file = sandbox.path().join("out.txt");

    let mut generator = SchemaGenerator::default();
    generator.add::<Strings>();
    generator.generate(&file, LinesRenderer).unwrap();

    assert_eq!(
        std::fs::read_to_string(file).unwrap(),
        format!("{}\n", generator.render(LinesRenderer).unwrap())
    );
}
