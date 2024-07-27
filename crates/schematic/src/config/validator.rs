use super::path::{Path, PathSegment};
use miette::Diagnostic;
use starbase_styles::{Style, Stylize};
use std::borrow::Borrow;
use thiserror::Error;

/// Error for a single validation failure.
#[derive(Clone, Debug, Diagnostic, Error)]
#[error("{}{} {message}", .path.to_string().style(Style::Id), ":".style(Style::MutedLight))]
pub struct ValidateError {
    /// Failure message.
    pub message: String,

    /// Relative path to the setting that failed validation.
    pub path: Path,
}

impl ValidateError {
    /// Create a new validation error with the provided message.
    pub fn new<T: AsRef<str>>(message: T) -> Self {
        ValidateError {
            message: message.as_ref().to_owned(),
            path: Path::default(),
        }
    }

    /// Create a new validation error for a required setting.
    pub fn required() -> Self {
        ValidateError {
            message: "this setting is required".into(),
            path: Path::default(),
        }
    }

    /// Create a new validation error with the provided message and [`Path`].
    pub fn with_path<T: AsRef<str>>(message: T, path: Path) -> Self {
        ValidateError {
            message: message.as_ref().to_owned(),
            path,
        }
    }

    /// Create a new validation error with the provided message and path [`PathSegment`].
    pub fn with_segment<T: AsRef<str>>(message: T, segment: PathSegment) -> Self {
        Self::with_segments(message, [segment])
    }

    /// Create a new validation error with the provided message and multiple path [`PathSegment`]s.
    pub fn with_segments<T: AsRef<str>, I>(message: T, segments: I) -> Self
    where
        I: IntoIterator<Item = PathSegment>,
    {
        ValidateError {
            message: message.as_ref().to_owned(),
            path: Path::new(segments.into_iter().collect()),
        }
    }

    pub fn prepend_path(&mut self, path: Path) {
        self.path = path.join_path(&self.path);
    }
}

/// Error that contains multiple validation errors, for each setting that failed.
#[derive(Debug, Diagnostic, Error)]
#[error("{}", self.render_errors())]
pub struct ValidatorError {
    /// A list of validation errors for the current path. Includes nested errors.
    pub errors: Vec<ValidateError>,
}

impl ValidatorError {
    fn render_errors(&self) -> String {
        self.errors
            .iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Borrow<dyn Diagnostic> for Box<ValidatorError> {
    fn borrow(&self) -> &(dyn Diagnostic + 'static) {
        self.as_ref()
    }
}
