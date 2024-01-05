mod generator;
mod renderer;
mod renderers;

pub use generator::*;
pub use indexmap::*;
pub use renderer::*;
pub use schematic_types::*;

/// Renders JSON schemas.
#[cfg(feature = "json_schema")]
pub use renderers::json_schema::{self, *};

/// Renders JSON file templates.
#[cfg(all(feature = "template", feature = "json"))]
pub use renderers::json_template::*;

/// Renders JSONC file templates.
#[cfg(all(feature = "template", feature = "json"))]
pub use renderers::jsonc_template::*;

/// Helpers for file templates.
#[cfg(feature = "template")]
pub use renderers::template::{self, TemplateOptions, TemplateRenderer};

/// Renders TOML file templates.
#[cfg(all(feature = "template", feature = "toml"))]
pub use renderers::toml_template::*;

/// Renders TypeScript types.
#[cfg(feature = "typescript")]
pub use renderers::typescript::{self, *};

/// Renders YAML file templates.
#[cfg(all(feature = "template", feature = "yaml"))]
pub use renderers::yaml_template::*;
