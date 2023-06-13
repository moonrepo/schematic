use super::SchemaRenderer;
use miette::IntoDiagnostic;
use schematic_types::*;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// A generator collects [`SchemaType`]s and renders them to a specific file,
/// using a renderer that implements [`SchemaRenderer`].
#[derive(Debug, Default)]
pub struct SchemaGenerator {
    references: HashSet<String>,
    schemas: Vec<SchemaType>,
}

impl SchemaGenerator {
    /// Add a [`SchemaType`] to be rendered, derived from the provided [`Schematic`].
    pub fn add<T: Schematic>(&mut self) {
        let schema = T::generate_schema();
        self.add_schema(&schema);
    }

    /// Add an explicit [`SchemaType`] to be rendered, and recursively add any nested schemas.
    /// Schemas with a name will be considered a reference.
    pub fn add_schema(&mut self, schema: &SchemaType) {
        let mut schema = schema.to_owned();

        // Recursively add any nested schema types
        match &mut schema {
            SchemaType::Array(ArrayType { items_type, .. }) => {
                self.add_schema(&items_type);
            }
            SchemaType::Enum(EnumType { variants, .. }) => {
                if let Some(variants) = variants.as_ref() {
                    for field in variants {
                        self.add_schema(&field.type_of);
                    }
                }
            }
            SchemaType::Object(ObjectType {
                key_type,
                value_type,
                ..
            }) => {
                self.add_schema(&key_type);
                self.add_schema(&value_type);
            }
            SchemaType::Struct(StructType { ref mut fields, .. }) => {
                fields.sort_by(|a, d| a.name.cmp(&d.name));

                for field in fields {
                    self.add_schema(&field.type_of);
                }
            }
            SchemaType::Tuple(TupleType { items_types, .. }) => {
                for item in items_types {
                    self.add_schema(&item);
                }
            }
            SchemaType::Union(UnionType {
                variants_types,
                variants,
                ..
            }) => {
                for variant in variants_types {
                    self.add_schema(&variant);
                }

                if let Some(variants) = variants.as_ref() {
                    for field in variants {
                        self.add_schema(&field.type_of);
                    }
                }
            }
            _ => {}
        };

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

        self.schemas.push(schema);
    }

    /// Generate an output by rendering all collected [`SchemaType`]s using the provided
    /// [`SchemaRenderer`], and finally write to the provided file path.
    pub fn generate<P: AsRef<Path>, O, R: SchemaRenderer<O>>(
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
