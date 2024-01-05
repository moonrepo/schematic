#![allow(clippy::should_implement_trait)]
#![allow(clippy::new_ret_no_self)]

use super::template::{TemplateOptions, TemplateRenderer};
use crate::format::Format;

/// Renders JSON file templates with comments.
pub struct JsoncTemplateRenderer;

impl JsoncTemplateRenderer {
    pub fn default() -> TemplateRenderer {
        TemplateRenderer::new_format(Format::Json)
    }

    pub fn new(options: TemplateOptions) -> TemplateRenderer {
        TemplateRenderer::new(Format::Json, options)
    }
}
