mod generator;
mod renderer;
mod renderers;

pub use generator::*;
pub use indexmap;
pub use renderer::*;
pub use schematic_types::*;

/// Renders JSON schemas.
#[cfg(feature = "renderer_json_schema")]
pub use renderers::json_schema::{self, *};

/// Renders JSON config templates.
#[cfg(all(feature = "renderer_template", feature = "json"))]
pub use renderers::json_template::*;

/// Renders JSONC config templates.
#[cfg(all(feature = "renderer_template", feature = "json"))]
pub use renderers::jsonc_template::*;

/// Renders Pkl config templates.
#[cfg(all(feature = "renderer_template", feature = "pkl"))]
pub use renderers::pkl_template::*;

/// Helpers for config templates.
#[cfg(feature = "renderer_template")]
pub use renderers::template::TemplateOptions;

/// Renders TOML config templates.
#[cfg(all(feature = "renderer_template", feature = "toml"))]
pub use renderers::toml_template::*;

/// Renders TypeScript types.
#[cfg(feature = "renderer_typescript")]
pub use renderers::typescript::{self, *};

/// Renders YAML config templates.
#[cfg(all(feature = "renderer_template", any(feature = "yaml", feature = "yml")))]
pub use renderers::yaml_template::*;
