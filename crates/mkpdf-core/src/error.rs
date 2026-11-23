use std::fmt;

/// Stable, coarse classification of failures. Bindings map these to status codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// An argument was outside its documented domain.
    InvalidArgument,
    /// A documented resource limit was exceeded.
    LimitExceeded,
}

/// Error returned by fallible core operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// Classification of this error.
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Human-readable description. Not a stable contract; match on [`Error::kind`].
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}
