#[cfg(feature = "json_schema")]
pub mod json_schema;

#[cfg(all(feature = "template", feature = "json"))]
pub mod json_template;

#[cfg(all(feature = "template", feature = "json"))]
pub mod jsonc_template;

#[cfg(feature = "template")]
pub mod template;

#[cfg(all(feature = "template", feature = "toml"))]
pub mod toml_template;

#[cfg(feature = "typescript")]
pub mod typescript;

#[cfg(all(feature = "template", feature = "yaml"))]
pub mod yaml_template;
