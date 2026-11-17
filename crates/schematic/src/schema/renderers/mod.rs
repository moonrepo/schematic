#[cfg(feature = "renderer_api_docs")]
pub mod api_docs;

#[cfg(any(
    feature = "renderer_api_docs",
    feature = "renderer_pkl_schema",
    feature = "renderer_typescript"
))]
mod doc_tags;

#[cfg(feature = "renderer_json_schema")]
pub mod json_schema;

#[cfg(all(feature = "renderer_template", feature = "json"))]
pub mod json_template;

#[cfg(all(feature = "renderer_template", feature = "json"))]
pub mod jsonc_template;

#[cfg(any(
    feature = "renderer_pkl_schema",
    all(feature = "renderer_template", feature = "pkl")
))]
mod pkl;

#[cfg(feature = "renderer_pkl_schema")]
pub mod pkl_schema;

#[cfg(all(feature = "renderer_template", feature = "pkl"))]
pub mod pkl_template;

#[cfg(feature = "renderer_template")]
pub mod template;

#[cfg(all(feature = "renderer_template", feature = "toml"))]
pub mod toml_template;

#[cfg(feature = "renderer_typescript")]
pub mod typescript;

#[cfg(all(feature = "renderer_template", feature = "yaml"))]
pub mod yaml_template;

/// Expand every tagged variant into the struct serde writes for it, for a
/// renderer that reads the shape of the structs it renders, rather than
/// shaping each tag format itself.
#[cfg(any(
    feature = "renderer_api_docs",
    feature = "renderer_pkl_schema",
    all(
        feature = "renderer_template",
        any(feature = "json", feature = "pkl", feature = "toml", feature = "yaml")
    )
))]
fn expand_tagged(
    mut schemas: indexmap::IndexMap<String, schematic_types::Schema>,
) -> indexmap::IndexMap<String, schematic_types::Schema> {
    for schema in schemas.values_mut() {
        schema.expand_tagged();
    }

    schemas
}
