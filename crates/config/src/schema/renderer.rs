use schematic_types::SchemaType;
use std::collections::HashSet;

pub trait SchemaRenderer {
    fn render(
        &self,
        schemas: &[SchemaType],
        references: &HashSet<String>,
    ) -> miette::Result<String>;
}
