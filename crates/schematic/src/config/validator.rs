use crate::config::path::{Path, PathSegment};
use miette::Diagnostic;
use starbase_styles::color;
use starbase_styles::{Style, Stylize};
use std::fmt::{self, Display};
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
}

/// Either contains a single or multiple validation errors.
#[derive(Clone, Debug)]
pub enum ValidateErrorType {
    Setting { path: Path, error: ValidateError },

    Nested { error: ValidatorError },
}

impl ValidateErrorType {
    pub fn setting(path: Path, error: ValidateError) -> Self {
        ValidateErrorType::Setting { path, error }
    }

    pub fn nested(error: ValidatorError) -> Self {
        ValidateErrorType::Nested { error }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            ValidateErrorType::Setting { .. } => false,
            ValidateErrorType::Nested { error } => error.is_empty(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            ValidateErrorType::Setting { .. } => 1,
            ValidateErrorType::Nested { error } => error.len(),
        }
    }

    pub fn to_error_list(&self) -> Vec<String> {
        let mut list = vec![];

        match self {
            ValidateErrorType::Setting { path, error } => {
                let mut error = error.clone();
                error.path = path.join_path(&error.path);

                list.push(error.to_string());
            }
            ValidateErrorType::Nested {
                error: nested_error,
            } => {
                for error in &nested_error.errors {
                    list.extend(error.to_error_list());
                }
            }
        }

        list
    }
}

/// Error that contains multiple validation errors, for each setting that failed.
#[derive(Clone, Debug, Diagnostic, Error)]
pub struct ValidatorError {
    /// When nested, the path to the setting that contains the nested error.
    pub path: Path,

    /// A list of validation errors for the current path. Includes nested errors.
    pub errors: Vec<ValidateErrorType>,
}

impl ValidatorError {
    /// Return true if there are no validation errors.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Return a count of all recursive validation errors.
    pub fn len(&self) -> usize {
        self.errors.iter().map(|e| e.len()).sum()
    }

    /// Return a string of all recursive validation errors, joined with newlines.
    pub fn to_full_string(&self) -> String {
        let mut message = String::new();

        for error_type in &self.errors {
            for error in error_type.to_error_list() {
                message.push_str(format!("\n  {}", error).as_str());
            }
        }

        message
    }
}

impl Display for ValidatorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        let dot = color::failure("·");

        for error_type in &self.errors {
            for error in error_type.to_error_list() {
                if first {
                    write!(f, "{} {}", dot, error)?;
                    first = false;
                } else {
                    write!(f, "\n{} {}", dot, error)?;
                }
            }
        }

        writeln!(f)?;

        Ok(())
    }
}
