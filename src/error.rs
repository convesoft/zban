//! Operation errors with the stable codes of `DES-CLI`.

use std::fmt;
use std::io;
use std::path::Path;

/// A stable error code that agents can branch on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// No project was found above the starting directory.
    ProjectNotFound,
    /// `init` found an existing `.zban/` directory.
    ProjectExists,
    /// The project configuration is malformed or unsupported.
    InvalidConfig,
    /// An ID names a file in both `.zban/` and `.zban/archive/`.
    AmbiguousId,
    /// The operation needs an issue file that cannot be read.
    Unreadable,
    /// A value breaks the issue file format or the project layout.
    InvalidValue,
    /// `done` with unchecked task items, without force.
    UncheckedItems,
    /// `done` or `cancelled` with open children, without force.
    OpenChildren,
    /// The file changed between reading and writing it.
    Conflict,
    /// Any other file-system failure.
    Io,
}

impl ErrorCode {
    /// The code as it appears in JSON output.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProjectNotFound => "project_not_found",
            Self::ProjectExists => "project_exists",
            Self::InvalidConfig => "invalid_config",
            Self::AmbiguousId => "ambiguous_id",
            Self::Unreadable => "unreadable",
            Self::InvalidValue => "invalid_value",
            Self::UncheckedItems => "unchecked_items",
            Self::OpenChildren => "open_children",
            Self::Conflict => "conflict",
            Self::Io => "io",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failed operation: a stable code and a message for humans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
}

impl Error {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn invalid_value(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidValue, message)
    }

    /// An `io` error that names the path it concerns.
    pub fn io(path: &Path, error: &io::Error) -> Self {
        Self::new(ErrorCode::Io, format!("{}: {error}", path.display()))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

pub type Result<T, E = Error> = std::result::Result<T, E>;
