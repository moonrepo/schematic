use super::path::{Path, PathSegment};
use miette::Diagnostic;
use starbase_styles::{Style, Stylize};
use std::borrow::Borrow;
use thiserror::Error;

pub type ValidateResult = std::result::Result<(), ValidateError>;

/// A validator function that receives a setting value to validate, the parent
/// configuration the setting belongs to, the current context, and can return
/// a [`ValidateError`] on failure.
pub type Validator<Val, Data, Ctx> = Box<dyn FnOnce(&Val, &Data, &Ctx, bool) -> ValidateResult>;

/// Error for a single validation failure.
#[derive(Clone, Debug, Diagnostic, Error)]
#[error("{}", self.render())]
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

    // An error at the root, like one from a config's own validator, has no
    // setting to name, so it's only the message
    fn render(&self) -> String {
        if self.path.is_empty() {
            return self.message.clone();
        }

        format!(
            "{}{} {}",
            self.path.to_string().style(Style::Id),
            ":".style(Style::MutedLight),
            self.message
        )
    }

    #[doc(hidden)]
    pub fn prepend_path(self, path: Path) -> Self {
        Self {
            message: self.message,
            path: path.join_path(&self.path),
        }
    }
}

/// The error a validator returns, which may report several failures, such as
/// one for each item of a list. Implemented for a single [`ValidateError`], a
/// list of them, and a garde [`Report`](garde::Report), whose errors are each
/// reported at their path.
pub trait IntoValidateErrors {
    /// Convert into the errors to report, each relative to the setting.
    fn into_validate_errors(self) -> Vec<ValidateError>;
}

impl IntoValidateErrors for ValidateError {
    fn into_validate_errors(self) -> Vec<ValidateError> {
        vec![self]
    }
}

impl IntoValidateErrors for Vec<ValidateError> {
    fn into_validate_errors(self) -> Vec<ValidateError> {
        self
    }
}

// A garde path's components aren't public, so it's kept as it renders, like `a.b[0]`
impl IntoValidateErrors for garde::Report {
    fn into_validate_errors(self) -> Vec<ValidateError> {
        self.into_inner()
            .into_iter()
            .map(|(path, error)| {
                let path = path.to_string();

                if path.is_empty() {
                    ValidateError::new(error.message())
                } else {
                    ValidateError::with_segment(error.message(), PathSegment::Key(path))
                }
            })
            .collect()
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
