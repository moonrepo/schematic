//! A tagged enum variant holds its value apart from its tag, which renderers
//! shape per format, or render as the struct serde writes for it.
#![allow(dead_code)]

use indexmap::IndexMap;
use schematic::schema::*;
use schematic::*;

/// The git source's own docs.
#[derive(Schematic)]
pub struct GitSource {
    url: String,
}

#[derive(Schematic)]
#[serde(tag = "type")]
pub enum Source {
    /// The variant's docs.
    Git(GitSource),
    File {
        path: String,
    },
    None,
}

fn variants(schema: &Schema) -> Vec<&TaggedType> {
    let SchemaType::Union(uni) = &schema.ty else {
        panic!("{schema:?}");
    };

    uni.variants_types
        .iter()
        .map(|variant| match &variant.ty {
            SchemaType::Tagged(tagged) => tagged.as_ref(),
            _ => panic!("{variant:?}"),
        })
        .collect()
}

// The value is the named type's shared schema, so the tag and the variant's
// docs stay out of it
#[test]
fn holds_the_value_as_it_is() {
    let schema = SchemaBuilder::build_root::<Source>();
    let variants = variants(&schema);

    let git = variants[0];
    let value = git.value.as_ref().unwrap();

    assert_eq!(git.format, TagFormat::Internal { tag: "type".into() });
    assert_eq!(git.name, "Git");
    assert_eq!(value.name.as_deref(), Some("GitSource"));
    assert_eq!(
        value.description.as_deref(),
        Some("The git source's own docs.")
    );

    let SchemaType::Struct(structure) = &value.ty else {
        panic!();
    };

    assert!(!structure.fields.contains_key("type"));

    assert_eq!(variants[1].name, "File");
    assert!(variants[2].value.is_none());
}

#[test]
fn registers_the_type_a_variant_holds() {
    let mut generator = SchemaGenerator::default();
    generator.add::<Source>();

    assert!(generator.schemas.contains_key("GitSource"));
}

mod partial {
    use super::*;

    #[derive(Config)]
    #[serde(tag = "type")]
    pub enum Mode {
        #[setting(default)]
        Off,
        File {
            path: String,
        },
    }

    // The tag is always written, so only the value becomes optional
    #[test]
    fn partializes_only_the_value() {
        let schema = SchemaBuilder::build_root::<PartialMode>();
        let file = variants(&schema)[1];

        let SchemaType::Struct(structure) = &file.value.as_ref().unwrap().ty else {
            panic!();
        };

        assert!(structure.fields["path"].optional);
        assert!(!structure.fields.contains_key("type"));

        let mut expanded = Schema::tagged(file.clone());
        expanded.expand_tagged();

        let SchemaType::Struct(structure) = &expanded.ty else {
            panic!();
        };

        assert!(!structure.fields["type"].optional);
        assert!(!structure.fields["type"].nullable);
    }
}

/// Renders a struct as its keys, and nothing else it needn't.
struct KeysRenderer;

impl SchemaRenderer<String> for KeysRenderer {
    fn is_reference(&self, _name: &str) -> bool {
        false
    }

    fn render_literal(&mut self, _literal: &LiteralType, _schema: &Schema) -> RenderResult {
        Ok("literal".into())
    }

    fn render_string(&mut self, _string: &StringType, _schema: &Schema) -> RenderResult {
        Ok("string".into())
    }

    fn render_struct(&mut self, structure: &StructType, _schema: &Schema) -> RenderResult {
        let mut keys = vec![];

        for (key, field) in structure.sorted_fields() {
            keys.push(format!("{key}: {}", self.render_schema(&field.schema)?));
        }

        Ok(format!("{{ {} }}", keys.join(", ")))
    }

    fn render_union(&mut self, uni: &UnionType, _schema: &Schema) -> RenderResult {
        let mut variants = vec![];

        for variant in &uni.variants_types {
            variants.push(self.render_schema(variant)?);
        }

        Ok(variants.join(" | "))
    }

    fn render(&mut self, schemas: IndexMap<String, Schema>) -> RenderResult {
        self.render_schema_without_reference(&schemas["Source"])
    }
}

// A renderer that doesn't render tagged variants renders the struct serde
// writes for each, without implementing anything for them
#[test]
fn renders_the_struct_serde_writes_by_default() {
    let mut generator = SchemaGenerator::default();
    generator.add::<Source>();

    assert_eq!(
        generator.render(KeysRenderer).unwrap(),
        "{ GitSource: { url: string }, type: literal } | { path: string, type: literal } | { type: literal }"
    );
}
