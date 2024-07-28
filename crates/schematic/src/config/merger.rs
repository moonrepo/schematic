use miette::Diagnostic;
use std::fmt::Display;
use thiserror::Error;

pub type MergeResult<T> = std::result::Result<Option<T>, MergeError>;

/// A merger function that receives the previous and next values, the current
/// context, and can return a [`MergeError`] on failure.
pub type Merger<Val, Ctx> = Box<dyn FnOnce(Val, Val, &Ctx) -> MergeResult<Val>>;

/// Error for merge failures.
#[derive(Error, Debug, Diagnostic)]
#[error("{0}")]
pub struct MergeError(pub String);

impl MergeError {
    pub fn new<T: Display>(message: T) -> Self {
        Self(message.to_string())
    }
}
