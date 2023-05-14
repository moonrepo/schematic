use std::fmt::{self, Display};

#[derive(Clone, Debug)]
pub enum Segment {
    Index(usize),
    Key(String),
    Variant(String),
    Unknown,
}

#[derive(Clone, Debug, Default)]
pub struct SettingPath {
    segments: Vec<Segment>,
}

impl SettingPath {
    pub fn new(segments: Vec<Segment>) -> Self {
        Self { segments }
    }

    pub fn join(&self, segment: Segment) -> Self {
        let mut path = self.clone();
        path.segments.push(segment);
        path
    }

    pub fn join_key(&self, key: &str) -> Self {
        self.join(Segment::Key(key.to_owned()))
    }

    pub fn join_path(&self, other: &Self) -> Self {
        let mut path = self.clone();
        path.segments.extend(other.segments.clone());
        path
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

#[derive(Clone, Debug)]
pub struct ValidateError {
    pub message: String,
    pub path: Option<SettingPath>,
}

impl ValidateError {
    pub fn new<T: AsRef<str>>(message: T) -> Self {
        ValidateError {
            message: message.as_ref().to_owned(),
            path: None,
        }
    }

    pub fn with_path<T: AsRef<str>>(message: T, path: SettingPath) -> Self {
        ValidateError {
            message: message.as_ref().to_owned(),
            path: Some(path),
        }
    }

    pub fn with_segments<T: AsRef<str>>(message: T, segments: Vec<Segment>) -> Self {
        ValidateError {
            message: message.as_ref().to_owned(),
            path: Some(SettingPath::new(segments)),
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
        path: SettingPath,
        error: ValidatorError,
    },
}

impl ValidateErrorType {
    pub fn setting(path: SettingPath, error: ValidateError) -> Self {
        ValidateErrorType::Setting { path, error }
    }

    pub fn nested(path: SettingPath, error: ValidatorError) -> Self {
        ValidateErrorType::Nested { path, error }
    }

    pub fn to_error_list(&self) -> Vec<String> {
        let mut list = vec![];

        match self {
            ValidateErrorType::Setting { path, error } => {
                let path = match &error.path {
                    Some(child_path) => path.join_path(&child_path),
                    None => path.clone(),
                };

                list.push(format!("`{}` - {}", path, error.message));
            }
            ValidateErrorType::Nested { path, error } => {
                // let path = match parent_path {
                //     Some(parent_path) => parent_path.join_path(path),
                //     None => path.clone(),
                // };

                // for error_type in &error.errors {
                //     list.extend(error_type.to_error_list(Some(&path)));
                // }
            }
        }

        list
    }
}

#[derive(Clone, Debug)]
pub struct ValidatorError {
    pub errors: Vec<ValidateErrorType>,
}

impl std::error::Error for ValidatorError {}

impl Display for ValidatorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to validate:")?;

        for error_type in &self.errors {
            for error in error_type.to_error_list() {
                write!(f, "\n  {}", error)?;
            }
        }

        Ok(())
    }
}
