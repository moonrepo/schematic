#![allow(clippy::should_implement_trait)]
#![allow(clippy::new_ret_no_self)]

use super::template::{TemplateOptions, TemplateRenderer};
use crate::format::Format;

/// Renders YAML file templates.
pub struct YamlTemplateRenderer;

impl YamlTemplateRenderer {
    pub fn default() -> TemplateRenderer {
        TemplateRenderer::new_format(Format::Yaml)
    }

    pub fn new(options: TemplateOptions) -> TemplateRenderer {
        TemplateRenderer::new(Format::Yaml, options)
    }
}
