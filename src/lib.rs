//! The zban core library: the issue model, the file store and the operations
//! that the `zban` command line and MCP server adapt.

pub mod config;
pub mod error;
pub mod id;
pub mod issue;
pub mod lifecycle;
mod markdown;
pub mod store;
pub mod timestamp;
mod yaml;

pub use error::{Error, ErrorCode, Result};
pub use id::{Id, Prefix};
pub use issue::{Issue, IssueType, Priority, Problem, ProblemCode, Status};
pub use lifecycle::Checklist;
pub use store::{IssueFile, Project};
pub use timestamp::Timestamp;
