#![allow(clippy::should_implement_trait)]
#![allow(clippy::new_ret_no_self)]

use super::template::{TemplateOptions, TemplateRenderer};
use crate::format::Format;
use std::mem;

/// Renders JSON file templates without comments.
pub struct JsonTemplateRenderer;

impl JsonTemplateRenderer {
    pub fn default() -> TemplateRenderer {
        Self::new(TemplateOptions::default())
    }

    pub fn new(mut options: TemplateOptions) -> TemplateRenderer {
        options.comments = false;
        options
            .hide_fields
            .extend(mem::take(&mut options.comment_fields));

        TemplateRenderer::new(Format::Json, options)
    }
}
