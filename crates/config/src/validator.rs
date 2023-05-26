use miette::Diagnostic;
use starbase_styles::color;
use starbase_styles::{Style, Stylize};
use std::fmt::{self, Display};
use thiserror::Error;

#[derive(Clone, Debug)]
pub enum Segment {
    /// List index: [0]
    Index(usize),
    /// Map key: name.
    Key(String),
    /// Enum variant: name.
    Variant(String),
    /// Unknown segment: ?
    Unknown,
}

#[derive(Clone, Debug, Default)]
pub struct SettingPath {
    /// List of path segments.
    segments: Vec<Segment>,
}

impl SettingPath {
    /// Create a new instance with the provided [Segment]s.
    pub fn new(segments: Vec<Segment>) -> Self {
        Self { segments }
    }

    /// Create a new instance and append the provided [Segment]
    /// to the end of the current path.
    pub fn join(&self, segment: Segment) -> Self {
        let mut path = self.clone();
        path.segments.push(segment);
        path
    }

    /// Create a new instance and append an `Index` [Segment]
    /// to the end of the current path.
    pub fn join_index(&self, index: usize) -> Self {
        self.join(Segment::Index(index))
    }

    /// Create a new instance and append an `Key` [Segment]
    /// to the end of the current path.
    pub fn join_key(&self, key: &str) -> Self {
        self.join(Segment::Key(key.to_owned()))
    }

    /// Create a new instance and append another [SettingPath]
    /// to the end of the current path.
    pub fn join_path(&self, other: &Self) -> Self {
        let mut path = self.clone();
        path.segments.extend(other.segments.clone());
        path
    }

    /// Create a new instance and append an `Variant` [Segment]
    /// to the end of the current path.
    pub fn join_variant(&self, variant: &str) -> Self {
        self.join(Segment::Variant(variant.to_owned()))
    }
}

impl Display for SettingPath {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.segments.is_empty() {
            return formatter.write_str(".");
        }

        let mut separator = "";

        for segment in &self.segments {
            match segment {
                Segment::Index(index) => {
                    write!(formatter, "[{}]", index)?;
                }
                Segment::Key(key) | Segment::Variant(key) => {
                    write!(formatter, "{}{}", separator, key)?;
                }
                Segment::Unknown => {
                    write!(formatter, "{}?", separator)?;
                }
            }

            separator = ".";
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Diagnostic, Error)]
#[error("{}{} {message}", .path.to_string().style(Style::Id), ":".style(Style::MutedLight))]
pub struct ValidateError {
    pub message: String,
    pub path: SettingPath,
}

impl ValidateError {
    /// Create a new validation error with the provided message.
    pub fn new<T: AsRef<str>>(message: T) -> Self {
        ValidateError {
            message: message.as_ref().to_owned(),
            path: SettingPath::default(),
        }
    }

    /// Create a new validation error with the provided message and [SettingPath].
    pub fn with_path<T: AsRef<str>>(message: T, path: SettingPath) -> Self {
        ValidateError {
            message: message.as_ref().to_owned(),
            path,
        }
    }

    /// Create a new validation error with the provided message and path [Segment].
    pub fn with_segment<T: AsRef<str>>(message: T, segment: Segment) -> Self {
        Self::with_segments(message, [segment])
    }

    /// Create a new validation error with the provided message and multiple path [Segment]s.
    pub fn with_segments<T: AsRef<str>, I>(message: T, segments: I) -> Self
    where
        I: IntoIterator<Item = Segment>,
    {
        ValidateError {
            message: message.as_ref().to_owned(),
            path: SettingPath::new(segments.into_iter().collect()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ValidateErrorType {
    Setting {
        path: SettingPath,
        error: ValidateError,
    },

    Nested {
        error: ValidatorError,
    },
}

impl ValidateErrorType {
    pub fn setting(path: SettingPath, error: ValidateError) -> Self {
        ValidateErrorType::Setting { path, error }
    }

    pub fn nested(error: ValidatorError) -> Self {
        ValidateErrorType::Nested { error }
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

#[derive(Clone, Debug, Diagnostic, Error)]
pub struct ValidatorError {
    /// When nested, the path to the setting that contains the nested error.
    pub path: SettingPath,

    /// A list of validation errors for the current path. Includes nested errors.
    pub errors: Vec<ValidateErrorType>,
}

impl ValidatorError {
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

        Ok(())
    }
}
