//! Operation errors with the stable codes of `DES-CLI`.

use std::fmt;
use std::io;
use std::path::Path;

/// A stable error code that agents can branch on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// Unknown, missing or conflicting arguments. The command line exits
    /// with status 2 for it.
    Usage,
    /// No project was found above the starting directory.
    ProjectNotFound,
    /// `init` found an existing `.zban/` directory.
    ProjectExists,
    /// The project configuration is malformed or unsupported.
    InvalidConfig,
    /// A named issue does not exist.
    NotFound,
    /// An ID names a file in both `.zban/` and `.zban/archive/`.
    AmbiguousId,
    /// The operation needs an issue file that cannot be read.
    Unreadable,
    /// A value breaks the issue file format or the project layout.
    InvalidValue,
    /// A relation names an unknown issue or the issue itself.
    InvalidReference,
    /// A relation would create a cycle or a blocker on an ancestor.
    Cycle,
    /// `done` with unchecked task items, without force.
    UncheckedItems,
    /// `done` or `cancelled` with open children, without force.
    OpenChildren,
    /// Deleting an issue that another issue references.
    Referenced,
    /// A replacement's old text does not occur in the body.
    TextNotFound,
    /// A replacement's old text occurs more than once in the body.
    TextAmbiguous,
    /// The file changed between reading and writing it.
    Conflict,
    /// Any other file-system failure.
    Io,
}

impl ErrorCode {
    /// The code as it appears in JSON output.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Usage => "usage",
            Self::ProjectNotFound => "project_not_found",
            Self::ProjectExists => "project_exists",
            Self::InvalidConfig => "invalid_config",
            Self::NotFound => "not_found",
            Self::AmbiguousId => "ambiguous_id",
            Self::Unreadable => "unreadable",
            Self::InvalidValue => "invalid_value",
            Self::InvalidReference => "invalid_reference",
            Self::Cycle => "cycle",
            Self::UncheckedItems => "unchecked_items",
            Self::OpenChildren => "open_children",
            Self::Referenced => "referenced",
            Self::TextNotFound => "text_not_found",
            Self::TextAmbiguous => "text_ambiguous",
            Self::Conflict => "conflict",
            Self::Io => "io",
        }
    }
}

impl serde::Serialize for ErrorCode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failed operation: a stable code and a message for humans. In JSON it is
/// the inner object of `{"error": {"code": ..., "message": ...}}`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
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

    pub fn usage(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Usage, message)
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
