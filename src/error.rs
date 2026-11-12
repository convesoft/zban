//! Operation errors with the stable codes of `DES-CLI`.

use std::fmt;
use std::io;
use std::path::Path;

/// Declares [`ErrorCode`] with the JSON name of each code, and
/// [`ErrorCode::ALL`], which therefore cannot miss a code.
macro_rules! error_codes {
    ($($(#[$doc:meta])* $code:ident => $name:literal,)*) => {
        /// A stable error code that agents can branch on.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum ErrorCode {
            $($(#[$doc])* $code,)*
        }

        impl ErrorCode {
            /// Every code, in declaration order.
            pub const ALL: &[Self] = &[$(Self::$code),*];

            /// The code as it appears in JSON output.
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$code => $name,)*
                }
            }
        }
    };
}

error_codes! {
    /// Unknown, missing or conflicting arguments. The command line exits
    /// with status 2 for it.
    Usage => "usage",
    /// No project was found above the starting directory.
    ProjectNotFound => "project_not_found",
    /// `init` found an existing `.zban/` directory.
    ProjectExists => "project_exists",
    /// The project configuration is malformed or unsupported.
    InvalidConfig => "invalid_config",
    /// A named issue does not exist.
    NotFound => "not_found",
    /// An ID names a file in both `.zban/` and `.zban/archive/`.
    AmbiguousId => "ambiguous_id",
    /// The operation needs an issue file that cannot be read.
    Unreadable => "unreadable",
    /// A value breaks the issue file format or the project layout.
    InvalidValue => "invalid_value",
    /// A relation names an unknown issue or the issue itself.
    InvalidReference => "invalid_reference",
    /// A relation would create a cycle or a blocker on an ancestor.
    Cycle => "cycle",
    /// `done` with unchecked task items, without force.
    UncheckedItems => "unchecked_items",
    /// `done` or `cancelled` with open children, without force.
    OpenChildren => "open_children",
    /// Deleting an issue that another issue references.
    Referenced => "referenced",
    /// A replacement's old text does not occur in the body.
    TextNotFound => "text_not_found",
    /// A replacement's old text occurs more than once in the body.
    TextAmbiguous => "text_ambiguous",
    /// Handoff text contains a level 1 or 2 heading or an unclosed fence, or
    /// the body has several handoff sections.
    InvalidHandoff => "invalid_handoff",
    /// The file changed between reading and writing it.
    Conflict => "conflict",
    /// Any other file-system failure.
    Io => "io",
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
