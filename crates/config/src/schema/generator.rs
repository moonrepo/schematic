use super::SchemaRenderer;
use miette::IntoDiagnostic;
use schematic_types::*;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// A generator collects [`SchemaType`]s and renders them to a specific file,
/// using a renderer that implements [`SchemaRenderer`].
#[derive(Default)]
pub struct SchemaGenerator {
    references: HashSet<String>,
    schemas: Vec<SchemaType>,
}

impl SchemaGenerator {
    /// Add a [`SchemaType`] to be rendered, derived from the provided [`Schematic`].
    pub fn add<T: Schematic>(&mut self) {
        self.add_schema(T::generate_schema());
    }

    /// Add an explicit [`SchemaType`] to be rendered, and recursively add any nested schemas.
    /// Schemas with a name will be considered a reference.
    pub fn add_schema(&mut self, schema: SchemaType) {
        // Store the name so that we can use it as a reference for other types
        if let Some(name) = schema.get_name() {
            // Type has already been added
            if self.references.contains(name) {
                return;
            }

            self.references.insert(name.to_owned());

        // Types without a name cannot be rendered at the root
        } else {
            return;
        }

        // Recursively add any nested schema types
        match &schema {
            SchemaType::Array(ArrayType { items_type, .. }) => {
                self.add_schema(*(*items_type).clone());
            }
            SchemaType::Object(ObjectType {
                key_type,
                value_type,
                ..
            }) => {
                self.add_schema(*(*key_type).clone());
                self.add_schema(*(*value_type).clone());
            }
            SchemaType::Struct(StructType { fields, .. }) => {
                for field in fields {
                    self.add_schema(field.type_of.clone());
                }
            }
            SchemaType::Tuple(TupleType { items_types, .. }) => {
                for item in items_types {
                    self.add_schema(*(*item).clone());
                }
            }
            SchemaType::Union(UnionType { variants_types, .. }) => {
                for variant in variants_types {
                    self.add_schema(*(*variant).clone());
                }
            }
            _ => {}
        };

        self.schemas.push(schema);
    }

    /// Generate an output by rendering all collected [`SchemaType`]s using the provided
    /// [`SchemaRenderer`], and finally write to the provided file path.
    pub fn generate<P: AsRef<Path>, R: SchemaRenderer>(
        &self,
        output_file: P,
        mut renderer: R,
    ) -> miette::Result<()> {
        let output_file = output_file.as_ref();

        let mut output = renderer.render(&self.schemas, &self.references)?;
        output.push('\n');

        if let Some(parent) = output_file.parent() {
            fs::create_dir_all(parent).into_diagnostic()?;
        }

        fs::write(output_file, output).into_diagnostic()?;

        Ok(())
    }
}
