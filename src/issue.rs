//! The issue model and the issue file format (`DES-ISSUE-FILE`).

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};
use crate::id::{Id, Prefix};
use crate::timestamp::Timestamp;
use crate::yaml::{self, Node};

/// Defines one fixed vocabulary (`REQ-FIXED-VOCABULARY`): its words, their
/// order and parsing that rejects every other value.
macro_rules! vocabulary {
    ($(#[$meta:meta])* $name:ident, $what:literal, [$($variant:ident = $word:literal),+ $(,)?]) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            /// Every value, in the documented order.
            pub const ALL: &[Self] = &[$(Self::$variant),+];

            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $word),+
                }
            }
        }

        impl FromStr for $name {
            type Err = Error;

            fn from_str(value: &str) -> Result<Self> {
                match value {
                    $($word => Ok(Self::$variant),)+
                    _ => {
                        let words: Vec<_> = Self::ALL.iter().map(|v| v.as_str()).collect();
                        Err(Error::invalid_value(format!(
                            "invalid {} {value:?}: use one of {}",
                            $what,
                            words.join(", ")
                        )))
                    }
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

vocabulary!(
    /// An issue's status.
    Status, "status", [
        Draft = "draft",
        Todo = "todo",
        InProgress = "in-progress",
        Done = "done",
        Cancelled = "cancelled",
    ]
);

vocabulary!(
    /// An issue's type.
    IssueType, "type", [
        Epic = "epic",
        Feature = "feature",
        Bug = "bug",
        Task = "task",
    ]
);

vocabulary!(
    /// An issue's priority, ordered highest first.
    Priority, "priority", [
        Critical = "critical",
        High = "high",
        Normal = "normal",
        Low = "low",
        Deferred = "deferred",
    ]
);

impl Status {
    /// Whether the status is `done` or `cancelled`.
    pub fn is_closed(self) -> bool {
        matches!(self, Self::Done | Self::Cancelled)
    }
}

/// An issue: its header fields and its body. The ID is not part of the
/// issue, because it lives only in the filename.
// @mara implements DES-ISSUE-FILE
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub title: String,
    pub status: Status,
    pub issue_type: IssueType,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub parent: Option<Id>,
    pub blocked_by: Vec<Id>,
    pub related: Vec<Id>,
    pub discovered_from: Option<Id>,
    pub created_at: Timestamp,
    pub closed_at: Option<Timestamp>,
    /// Header entries with keys zban does not know, verbatim and in order.
    /// Each entry ends with a newline.
    pub unknown: Vec<String>,
    /// Everything after the blank line that follows the header.
    pub body: String,
}

/// A problem found in one issue file, with a code from `DES-CHECK`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub code: ProblemCode,
    pub message: String,
}

/// The `DES-CHECK` codes that concern a single issue file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProblemCode {
    InvalidFilename,
    Unparseable,
    MissingField,
    InvalidValue,
    UnknownKey,
}

impl ProblemCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidFilename => "invalid_filename",
            Self::Unparseable => "unparseable",
            Self::MissingField => "missing_field",
            Self::InvalidValue => "invalid_value",
            Self::UnknownKey => "unknown_key",
        }
    }

    /// Whether the problem is an error. Only `unknown_key` is a warning, and
    /// it leaves the issue readable.
    pub fn is_error(self) -> bool {
        self != Self::UnknownKey
    }
}

impl Problem {
    pub(crate) fn new(code: ProblemCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

const KNOWN_KEYS: [&str; 11] = [
    "title",
    "status",
    "type",
    "priority",
    "tags",
    "parent",
    "blocked_by",
    "related",
    "discovered_from",
    "created_at",
    "closed_at",
];

impl Issue {
    /// A new issue with the defaults of `DES-LIFECYCLE`: `todo`, `task` and
    /// `normal`.
    pub fn new(title: impl Into<String>, created_at: Timestamp) -> Self {
        Self {
            title: title.into(),
            status: Status::Todo,
            issue_type: IssueType::Task,
            priority: Priority::Normal,
            tags: Vec::new(),
            parent: None,
            blocked_by: Vec::new(),
            related: Vec::new(),
            discovered_from: None,
            created_at,
            closed_at: None,
            unknown: Vec::new(),
            body: String::new(),
        }
    }

    /// Parses an issue file. The issue is `None` when any problem is an
    /// error; the problems list every finding either way.
    pub fn parse(prefix: &Prefix, text: &str) -> (Option<Self>, Vec<Problem>) {
        let Some((header, body)) = split(text) else {
            let problem = Problem::new(
                ProblemCode::Unparseable,
                "no frontmatter: the file must start with a `---` line and close the header with another",
            );
            return (None, vec![problem]);
        };
        // The header text starts on the line after the opening `---`.
        let entries = match yaml::parse_header(header, 2) {
            Ok(entries) => entries,
            Err(message) => {
                return (None, vec![Problem::new(ProblemCode::Unparseable, message)]);
            }
        };
        let mut reader = FieldReader {
            values: HashMap::new(),
            problems: Vec::new(),
        };
        let mut unknown = Vec::new();
        for entry in entries {
            if let Some(known) = KNOWN_KEYS.iter().find(|known| **known == entry.key) {
                reader.values.insert(known, entry.value);
            } else {
                reader.problems.push(Problem::new(
                    ProblemCode::UnknownKey,
                    format!("unknown key `{}`", entry.key),
                ));
                unknown.push(entry.text);
            }
        }

        let title = reader.required("title", |title: &str| {
            valid_title(title)
                .then(|| title.to_owned())
                .ok_or_else(|| "`title` must be a non-empty single line".to_owned())
        });
        let status = reader.required("status", vocabulary_word::<Status>);
        let issue_type = reader.required("type", vocabulary_word::<IssueType>);
        let priority = reader.optional("priority", vocabulary_word::<Priority>);
        let tags = reader.list("tags", |tag: &str| {
            valid_tag(tag)
                .then(|| tag.to_owned())
                .ok_or_else(|| format!("invalid tag {tag:?}: use lowercase letters, digits and single hyphens, starting with a letter"))
        });
        let parent = reader.optional("parent", |id: &str| id_value(prefix, "parent", id));
        let blocked_by = reader.list("blocked_by", |id: &str| id_value(prefix, "blocked_by", id));
        let related = reader.list("related", |id: &str| id_value(prefix, "related", id));
        let discovered_from = reader.optional("discovered_from", |id: &str| {
            id_value(prefix, "discovered_from", id)
        });
        let created_at =
            reader.required("created_at", |value: &str| timestamp("created_at", value));
        let closed_at = reader.optional("closed_at", |value: &str| timestamp("closed_at", value));

        if let Some(status) = status {
            match &closed_at {
                Some(None) if status.is_closed() => reader.problems.push(Problem::new(
                    ProblemCode::MissingField,
                    format!("missing key `closed_at`, required while the status is {status}"),
                )),
                Some(Some(_)) if !status.is_closed() => reader.problems.push(Problem::new(
                    ProblemCode::InvalidValue,
                    format!("`closed_at` is set, but the status {status} is open"),
                )),
                _ => {}
            }
        }

        let issue = (|| {
            Some(Self {
                title: title?,
                status: status?,
                issue_type: issue_type?,
                priority: priority?.unwrap_or(Priority::Normal),
                tags: tags?,
                parent: parent?,
                blocked_by: blocked_by?,
                related: related?,
                discovered_from: discovered_from?,
                created_at: created_at?,
                closed_at: closed_at?,
                unknown,
                body: body.to_owned(),
            })
        })();
        let problems = reader.problems;
        if problems.iter().any(|problem| problem.code.is_error()) {
            return (None, problems);
        }
        (issue, problems)
    }

    /// Checks the rules of `DES-ISSUE-FILE` that the types do not enforce.
    pub fn validate(&self) -> Result<()> {
        if !valid_title(&self.title) {
            return Err(Error::invalid_value(
                "the title must be a non-empty single line",
            ));
        }
        if let Some(tag) = self.tags.iter().find(|tag| !valid_tag(tag)) {
            return Err(Error::invalid_value(format!(
                "invalid tag {tag:?}: use lowercase letters, digits and single hyphens, starting with a letter"
            )));
        }
        distinct("tags", &self.tags)?;
        distinct("blocked_by", &self.blocked_by)?;
        distinct("related", &self.related)?;
        match (self.status.is_closed(), &self.closed_at) {
            (true, None) => Err(Error::invalid_value(format!(
                "closed_at is required while the status is {}",
                self.status
            ))),
            (false, Some(_)) => Err(Error::invalid_value(format!(
                "closed_at must be absent while the status is {}",
                self.status
            ))),
            _ => Ok(()),
        }
    }

    /// Renders the issue file in canonical form.
    pub fn render(&self) -> String {
        let mut text = String::from("---\n");
        push_field(&mut text, "title", &self.title);
        push_field(&mut text, "status", self.status.as_str());
        push_field(&mut text, "type", self.issue_type.as_str());
        if self.priority != Priority::Normal {
            push_field(&mut text, "priority", self.priority.as_str());
        }
        push_list(&mut text, "tags", &self.tags);
        if let Some(parent) = &self.parent {
            push_field(&mut text, "parent", parent.as_str());
        }
        push_list(&mut text, "blocked_by", &self.blocked_by);
        push_list(&mut text, "related", &self.related);
        if let Some(discovered_from) = &self.discovered_from {
            push_field(&mut text, "discovered_from", discovered_from.as_str());
        }
        push_field(&mut text, "created_at", self.created_at.as_str());
        if let Some(closed_at) = &self.closed_at {
            push_field(&mut text, "closed_at", closed_at.as_str());
        }
        for entry in &self.unknown {
            text.push_str(entry);
            if !entry.ends_with('\n') {
                text.push('\n');
            }
        }
        text.push_str("---\n");
        if !self.body.is_empty() {
            text.push('\n');
            text.push_str(&self.body);
        }
        text
    }
}

fn push_field(text: &mut String, key: &str, value: &str) {
    text.push_str(key);
    text.push_str(": ");
    text.push_str(&yaml::scalar(value));
    text.push('\n');
}

fn push_list(text: &mut String, key: &str, items: &[impl AsRef<str>]) {
    if items.is_empty() {
        return;
    }
    text.push_str(key);
    text.push_str(":\n");
    for item in items {
        text.push_str("  - ");
        text.push_str(&yaml::scalar(item.as_ref()));
        text.push('\n');
    }
}

/// Splits a file into header text and body. The blank line after the closing
/// `---` belongs to neither.
fn split(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix("---\n")?;
    let mut start = 0;
    loop {
        let end = rest[start..].find('\n').map(|at| start + at);
        if &rest[start..end.unwrap_or(rest.len())] == "---" {
            let after = end.map_or("", |end| &rest[end + 1..]);
            return Some((&rest[..start], after.strip_prefix('\n').unwrap_or(after)));
        }
        start = end? + 1;
    }
}

fn valid_title(title: &str) -> bool {
    !title.is_empty() && !title.contains(['\n', '\r'])
}

fn valid_tag(tag: &str) -> bool {
    tag.starts_with(|c: char| c.is_ascii_lowercase())
        && !tag.ends_with('-')
        && !tag.contains("--")
        && tag
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn distinct<T: AsRef<str>>(key: &str, items: &[T]) -> Result<()> {
    let mut seen = HashSet::new();
    match items.iter().find(|&item| !seen.insert(item.as_ref())) {
        Some(item) => Err(Error::invalid_value(format!(
            "`{key}` lists {:?} more than once",
            item.as_ref()
        ))),
        None => Ok(()),
    }
}

fn vocabulary_word<T: FromStr<Err = Error>>(value: &str) -> Result<T, String> {
    value.parse().map_err(|error: Error| error.message)
}

fn id_value(prefix: &Prefix, key: &str, value: &str) -> Result<Id, String> {
    Id::parse_input(prefix, value).ok_or_else(|| {
        format!("`{key}` holds {value:?}, which is not an issue ID with prefix `{prefix}`")
    })
}

fn timestamp(key: &str, value: &str) -> Result<Timestamp, String> {
    Timestamp::parse(value).ok_or_else(|| {
        format!("`{key}` holds {value:?}; use RFC 3339 in UTC to whole seconds, such as 2026-11-11T20:00:00Z")
    })
}

/// Reads known header values, collecting a problem for each invalid one.
/// Each accessor returns `None` exactly when it recorded an error.
struct FieldReader {
    values: HashMap<&'static str, Node>,
    problems: Vec<Problem>,
}

impl FieldReader {
    fn invalid(&mut self, message: String) {
        self.problems
            .push(Problem::new(ProblemCode::InvalidValue, message));
    }

    /// The scalar text of a value. Plain scalars are taken as written, so a
    /// hand-written `title: 2027` reads as the title "2027".
    fn scalar<T>(
        &mut self,
        key: &str,
        node: Node,
        convert: impl FnOnce(&str) -> Result<T, String>,
    ) -> Option<T> {
        match node {
            Node::Scalar { text, .. } => match convert(&text) {
                Ok(value) => Some(value),
                Err(message) => {
                    self.invalid(message);
                    None
                }
            },
            _ => {
                self.invalid(format!("`{key}` must be a single value"));
                None
            }
        }
    }

    fn take(&mut self, key: &'static str) -> Option<Node> {
        self.values.remove(key).filter(|node| !node.is_null())
    }

    fn required<T>(
        &mut self,
        key: &'static str,
        convert: impl FnOnce(&str) -> Result<T, String>,
    ) -> Option<T> {
        match self.take(key) {
            Some(node) => self.scalar(key, node, convert),
            None => {
                self.problems.push(Problem::new(
                    ProblemCode::MissingField,
                    format!("missing key `{key}`"),
                ));
                None
            }
        }
    }

    /// An optional value: `Some(None)` when absent, `None` when invalid.
    fn optional<T>(
        &mut self,
        key: &'static str,
        convert: impl FnOnce(&str) -> Result<T, String>,
    ) -> Option<Option<T>> {
        match self.take(key) {
            Some(node) => self.scalar(key, node, convert).map(Some),
            None => Some(None),
        }
    }

    fn list<T: AsRef<str>>(
        &mut self,
        key: &'static str,
        convert: impl Fn(&str) -> Result<T, String>,
    ) -> Option<Vec<T>> {
        let items = match self.take(key) {
            None => return Some(Vec::new()),
            Some(Node::Sequence(items)) => items,
            Some(_) => {
                self.invalid(format!("`{key}` must be a list"));
                return None;
            }
        };
        let mut values = Vec::with_capacity(items.len());
        let mut valid = true;
        for item in items {
            match self.scalar(key, item, &convert) {
                Some(value) => values.push(value),
                None => valid = false,
            }
        }
        if !valid {
            return None;
        }
        if let Err(error) = distinct(key, &values) {
            self.invalid(error.message);
            return None;
        }
        Some(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vocabulary_is_fixed() {
        let words = |all: &[&str]| all.join(" ");
        assert_eq!(
            words(&Status::ALL.iter().map(|v| v.as_str()).collect::<Vec<_>>()),
            "draft todo in-progress done cancelled"
        );
        assert_eq!(
            words(
                &IssueType::ALL
                    .iter()
                    .map(|v| v.as_str())
                    .collect::<Vec<_>>()
            ),
            "epic feature bug task"
        );
        assert_eq!(
            words(&Priority::ALL.iter().map(|v| v.as_str()).collect::<Vec<_>>()),
            "critical high normal low deferred"
        );
        assert!(Priority::Critical < Priority::High && Priority::Low < Priority::Deferred);
    }
}
