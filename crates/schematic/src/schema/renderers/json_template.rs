#![allow(clippy::should_implement_trait)]
#![allow(clippy::new_ret_no_self)]

use super::jsonc_template::JsoncTemplateRenderer;
use super::template::TemplateOptions;
use std::mem;

/// Renders JSON config templates without comments.
pub struct JsonTemplateRenderer;

impl JsonTemplateRenderer {
    pub fn default<'gen>() -> JsoncTemplateRenderer<'gen> {
        Self::new(TemplateOptions::default())
    }

    pub fn new<'gen>(mut options: TemplateOptions) -> JsoncTemplateRenderer<'gen> {
        options.comments = false;
        options
            .hide_fields
            .extend(mem::take(&mut options.comment_fields));
        options.newline_between_fields = false;

        JsoncTemplateRenderer::new(options)
    }
}
