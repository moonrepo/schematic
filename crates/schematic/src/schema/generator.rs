use super::SchemaRenderer;
use indexmap::IndexMap;
use miette::IntoDiagnostic;
use schematic_types::*;
use std::fs;
use std::path::Path;

/// A generator collects [`Schema`]s and renders them to a specific file,
/// using a renderer that implements [`SchemaRenderer`].
#[derive(Debug, Default)]
pub struct SchemaGenerator {
    pub schemas: IndexMap<String, Schema>,
}

impl SchemaGenerator {
    /// Add a [`Schema`] to be rendered, derived from the provided [`Schematic`].
    pub fn add<T: Schematic>(&mut self) {
        let schema = SchemaBuilder::build_root::<T>();
        self.add_schema(&schema);
    }

    /// Add an explicit [`Schema`] to be rendered, and recursively add any nested schemas.
    /// Schemas with a name will be considered a reference.
    pub fn add_schema(&mut self, schema: &Schema) {
        let mut schema = schema.to_owned();

        // Recursively add any nested schema types
        match &mut schema.ty {
            SchemaType::Array(inner) => {
                self.add_schema(&inner.items_type);
            }
            SchemaType::Object(inner) => {
                self.add_schema(&inner.key_type);
                self.add_schema(&inner.value_type);
            }
            SchemaType::Struct(inner) => {
                for field in inner.fields.values() {
                    self.add_schema(&field.schema);
                }
            }
            SchemaType::Tuple(inner) => {
                for item in &inner.items_types {
                    self.add_schema(item);
                }
            }
            SchemaType::Union(inner) => {
                for variant in &inner.variants_types {
                    self.add_schema(variant);
                }
            }
            _ => {}
        };

        // Store the name so that we can use it as a reference for other types
        if let Some(name) = &schema.name {
            // Types without a name cannot be rendered at the root
            if !self.schemas.contains_key(name) {
                self.schemas.insert(name.to_owned(), schema);
            }
        }
    }

    /// Generate an output by rendering all collected [`Schema`]s using the provided
    /// [`SchemaRenderer`], and finally write to the provided file path.
    pub fn generate<P: AsRef<Path>, O, R: SchemaRenderer<O>>(
        &self,
        output_file: P,
        mut renderer: R,
    ) -> miette::Result<()> {
        let output_file = output_file.as_ref();

        let mut output = renderer.render(self.schemas.clone())?;
        output.push('\n');

        if let Some(parent) = output_file.parent() {
            fs::create_dir_all(parent).into_diagnostic()?;
        }

        fs::write(output_file, output).into_diagnostic()?;

        Ok(())
    }
}
