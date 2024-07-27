use miette::{Diagnostic, NamedSource, SourceSpan};
use starbase_styles::{Style, Stylize};
use std::borrow::Borrow;
use thiserror::Error;

/// Error for a single parse failure. Can be used with [`TryFrom`], [`FromStr`], or serde.
#[derive(Clone, Debug, Diagnostic, Error)]
#[error("{message}")]
pub struct ParseError {
    /// Failure message.
    pub message: String,
}

impl ParseError {
    /// Create a new parse error with the provided message.
    pub fn new<T: AsRef<str>>(message: T) -> Self {
        ParseError {
            message: message.as_ref().to_owned(),
        }
    }
}

/// Error related to serde parsing.
#[derive(Error, Debug, Diagnostic)]
#[error("{}{} {message}", .path.style(Style::Id), ":".style(Style::MutedLight))]
#[diagnostic(severity(Error))]
pub struct ParserError {
    #[source_code]
    pub content: NamedSource<String>,

    pub message: String,

    pub path: String,

    #[label("Fix this")]
    pub span: Option<SourceSpan>,
}

impl Borrow<dyn Diagnostic> for Box<ParserError> {
    fn borrow(&self) -> &(dyn Diagnostic + 'static) {
        self.as_ref()
    }
}
