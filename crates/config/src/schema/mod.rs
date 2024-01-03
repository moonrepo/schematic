mod generator;
mod renderer;
mod renderers;

pub use generator::*;
pub use indexmap::*;
pub use renderer::*;
pub use schematic_types::*;

/// Renders JSON schemas.
#[cfg(feature = "json_schema")]
pub use renderers::json_schema;

/// Renders file templates.
#[cfg(feature = "template")]
pub use renderers::template;

/// Renders TypeScript types.
#[cfg(feature = "typescript")]
pub use renderers::typescript;
