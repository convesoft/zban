//! The operations that the command line and the MCP server adapt
//! (`DES-SHARED-CORE`), and the documents they return, whose JSON form is
//! specified in `DES-CLI`.
//!
//! Requests carry values as given, such as status words and IDs in any case,
//! so that both surfaces share one validation and one set of error codes
//! (`REQ-SURFACE-PARITY`).

use std::fmt;
use std::path::Path;
use std::str::FromStr;

use serde::Serialize;

use crate::error::{Error, ErrorCode, Result};
use crate::id::{Id, Prefix};
use crate::issue::{Issue, IssueType, Priority, Status};
use crate::lifecycle::Checklist;
use crate::relations::{Entry, Graph, IssueRef, NotReady};
use crate::store::{IssueFile, Project};
use crate::timestamp::Timestamp;

/// The result of `init`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Initialized {
    /// The absolute path of the created `.zban/` directory.
    pub path: String,
    pub prefix: String,
}

/// An issue as `list` returns it and as mutations return it.
// @mara implements DES-CLI
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IssueView {
    pub id: Id,
    pub title: String,
    pub status: Status,
    #[serde(rename = "type")]
    pub issue_type: IssueType,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub parent: Option<Id>,
    pub blocked_by: Vec<Id>,
    pub related: Vec<Id>,
    pub discovered_from: Option<Id>,
    pub created_at: Timestamp,
    pub closed_at: Option<Timestamp>,
    /// The absolute path of the issue file.
    pub path: String,
    pub checklist: Checklist,
    pub ready: bool,
    /// Every reason from `DES-READINESS`; empty exactly when `ready`.
    pub not_ready: Vec<NotReady>,
}

/// An issue as `show` returns it: an [`IssueView`] with the body and the
/// reverse relations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IssueDetail {
    #[serde(flatten)]
    pub issue: IssueView,
    pub body: String,
    pub children: Vec<Id>,
    pub blocks: Vec<Id>,
    pub related_from: Vec<Id>,
    pub discovered: Vec<Id>,
}

/// The result of `list` and `show`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Issues<T> {
    pub issues: Vec<T>,
    pub warnings: Vec<Warning>,
}

/// A file that `list` or `show` skipped, with a code from `DES-CHECK`.
// @mara implements REQ-DAMAGED-DATA
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Warning {
    pub code: &'static str,
    /// The absolute path of the skipped file.
    pub path: String,
    pub message: String,
}

/// A `create` request.
#[derive(Debug, Clone, Default)]
pub struct CreateRequest {
    pub title: String,
    pub issue_type: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub tags: Vec<String>,
    pub parent: Option<String>,
    pub blocked_by: Vec<String>,
    pub related: Vec<String>,
    pub discovered_from: Option<String>,
    pub body: Option<String>,
}

/// A `list` request. Repeated values of one filter match any of them, and
/// every given filter must match (`REQ-LIST-FILTERS`).
#[derive(Debug, Clone, Default)]
pub struct ListRequest {
    pub statuses: Vec<String>,
    pub types: Vec<String>,
    pub priorities: Vec<String>,
    pub tags: Vec<String>,
    /// Matches direct children of this issue only.
    pub parent: Option<String>,
    pub ready: bool,
    /// Matched case-insensitively against the title and the body.
    pub search: Option<String>,
    /// Includes closed issues.
    pub all: bool,
}

/// An `update` request. `Some(None)` in `parent` or `discovered_from`
/// removes the value.
#[derive(Debug, Clone, Default)]
pub struct UpdateRequest {
    pub id: String,
    pub title: Option<String>,
    pub status: Option<String>,
    pub issue_type: Option<String>,
    pub priority: Option<String>,
    pub add_tags: Vec<String>,
    pub remove_tags: Vec<String>,
    pub parent: Option<Option<String>>,
    pub add_blocked_by: Vec<String>,
    pub remove_blocked_by: Vec<String>,
    pub add_related: Vec<String>,
    pub remove_related: Vec<String>,
    pub discovered_from: Option<Option<String>>,
    /// Replaces the body. Applied before `replacements` and `append`.
    pub body: Option<String>,
    /// Applied in order, after `body` and before `append`.
    pub replacements: Vec<Replacement>,
    pub append: Option<String>,
    /// Skips the closing rules of `DES-LIFECYCLE`.
    pub force: bool,
}

/// One exact text replacement in the body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replacement {
    pub old: String,
    pub new: String,
}

/// Creates a project in `directory`.
pub fn init(directory: &Path, prefix: Option<&str>) -> Result<Initialized> {
    let project = Project::init(directory, prefix)?;
    Ok(Initialized {
        path: project.dir().display().to_string(),
        prefix: project.prefix().to_string(),
    })
}

/// Creates an issue with the defaults of `DES-LIFECYCLE`.
pub fn create(project: &Project, request: &CreateRequest) -> Result<IssueView> {
    let prefix = project.prefix();
    let now = Timestamp::now();
    let mut issue = Issue::new(request.title.clone(), now.clone());
    set(&mut issue.status, request.status.as_deref())?;
    set(&mut issue.issue_type, request.issue_type.as_deref())?;
    set(&mut issue.priority, request.priority.as_deref())?;
    issue.tags = distinct(request.tags.clone());
    issue.parent = optional_reference(prefix, "parent", request.parent.as_deref())?;
    issue.blocked_by = references(prefix, "blocked_by", &request.blocked_by)?;
    issue.related = references(prefix, "related", &request.related)?;
    issue.discovered_from = optional_reference(
        prefix,
        "discovered_from",
        request.discovered_from.as_deref(),
    )?;
    issue.body = request.body.as_deref().map(stored).unwrap_or_default();
    if issue.status.is_closed() {
        issue.closed_at = Some(now);
    }
    let files = project.load()?;
    let graph = Graph::new(&files);
    let file = project.create_among(&graph, &issue, || Id::generate(prefix))?;
    view_of(&graph, &file)
}

/// Lists issues in the order of `DES-READINESS`, with a warning for each
/// skipped file.
// @mara implements REQ-LIST-FILTERS
pub fn list(project: &Project, request: &ListRequest) -> Result<Issues<IssueView>> {
    let statuses: Vec<Status> = parse_all(&request.statuses)?;
    let types: Vec<IssueType> = parse_all(&request.types)?;
    let priorities: Vec<Priority> = parse_all(&request.priorities)?;
    // A value that is not an ID names no parent, so it matches nothing.
    let parent = request
        .parent
        .as_deref()
        .map(|parent| Id::parse_input(project.prefix(), parent));
    let search = request.search.as_deref().map(str::to_lowercase);
    let files = project.load()?;
    let graph = Graph::new(&files);
    let matches = |issue: &Issue| {
        let status = if statuses.is_empty() {
            request.all || !issue.status.is_closed()
        } else {
            statuses.contains(&issue.status)
        };
        let parent = match &parent {
            None => true,
            Some(parent) => parent.is_some() && issue.parent == *parent,
        };
        let search = search.as_ref().is_none_or(|text| {
            issue.title.to_lowercase().contains(text) || issue.body.to_lowercase().contains(text)
        });
        status
            && parent
            && search
            && (types.is_empty() || types.contains(&issue.issue_type))
            && (priorities.is_empty() || priorities.contains(&issue.priority))
            && (request.tags.is_empty() || issue.tags.iter().any(|t| request.tags.contains(t)))
    };
    let issues = graph
        .issues()
        .into_iter()
        .filter(|found| matches(found.issue))
        .map(|found| IssueView::of(&graph, found))
        .filter(|view| !request.ready || view.ready)
        .collect();
    Ok(Issues {
        issues,
        warnings: warnings(&files, &graph),
    })
}

/// Shows the named issues in the order requested.
pub fn show(project: &Project, ids: &[String]) -> Result<Issues<IssueDetail>> {
    if ids.is_empty() {
        return Err(Error::usage("name at least one issue to show"));
    }
    let ids = ids
        .iter()
        .map(|id| named(project.prefix(), id))
        .collect::<Result<Vec<_>>>()?;
    let files = project.load()?;
    let graph = Graph::new(&files);
    let issues = ids
        .iter()
        .map(|id| {
            graph
                .resolve(id)
                .map(|found| IssueDetail::of(&graph, found))
        })
        .collect::<Result<_>>()?;
    Ok(Issues {
        issues,
        warnings: warnings(&files, &graph),
    })
}

/// Updates one issue. The whole request is validated before anything is
/// written, and an issue the request leaves unchanged is not rewritten.
// @mara implements REQ-BODY-EDITS
pub fn update(project: &Project, request: &UpdateRequest) -> Result<IssueView> {
    let prefix = project.prefix();
    let id = named(prefix, &request.id)?;
    let add_blocked_by = references(prefix, "blocked_by", &request.add_blocked_by)?;
    let remove_blocked_by = references(prefix, "blocked_by", &request.remove_blocked_by)?;
    let add_related = references(prefix, "related", &request.add_related)?;
    let remove_related = references(prefix, "related", &request.remove_related)?;
    both("tag", &request.add_tags, &request.remove_tags)?;
    both("blocker", &add_blocked_by, &remove_blocked_by)?;
    both("related issue", &add_related, &remove_related)?;

    let files = project.load()?;
    let graph = Graph::new(&files);
    let found = graph.resolve(&id)?;
    let mut issue = found.issue.clone();
    if let Some(title) = &request.title {
        issue.title.clone_from(title);
    }
    set(&mut issue.status, request.status.as_deref())?;
    set(&mut issue.issue_type, request.issue_type.as_deref())?;
    set(&mut issue.priority, request.priority.as_deref())?;
    issue.tags.retain(|tag| !request.remove_tags.contains(tag));
    for tag in &request.add_tags {
        if !issue.tags.contains(tag) {
            issue.tags.push(tag.clone());
        }
    }
    if let Some(parent) = &request.parent {
        issue.parent = optional_reference(prefix, "parent", parent.as_deref())?;
    }
    issue
        .blocked_by
        .retain(|blocker| !remove_blocked_by.contains(blocker));
    for blocker in add_blocked_by {
        if !issue.blocked_by.contains(&blocker) {
            issue.blocked_by.push(blocker);
        }
    }
    let mut others = Vec::new();
    for other in &remove_related {
        others.extend(graph.remove_related(&id, &mut issue, other));
    }
    for other in &add_related {
        graph.add_related(&id, &mut issue, other);
    }
    if let Some(discovered_from) = &request.discovered_from {
        issue.discovered_from =
            optional_reference(prefix, "discovered_from", discovered_from.as_deref())?;
    }
    edit_body(&mut issue.body, request)?;
    graph.check(Some(&id), Some(found.issue), &issue)?;

    // The issue first: `save` refuses an unfinished close before it writes.
    let file = if issue == *found.issue {
        found.file.clone()
    } else {
        project.save(&files, found.file, issue, request.force)?
    };
    for (other, changed) in others {
        project.save(&files, other.file, changed, false)?;
    }
    view_of(&graph, &file)
}

/// Deletes an issue and returns it as it was.
pub fn delete(project: &Project, id: &str) -> Result<IssueView> {
    let id = named(project.prefix(), id)?;
    let files = project.load()?;
    let graph = Graph::new(&files);
    let view = IssueView::of(&graph, graph.resolve(&id)?);
    project.delete_among(&graph, &id)?;
    Ok(view)
}

impl IssueView {
    fn new(graph: &Graph, id: &Id, issue: &Issue, path: &Path) -> Self {
        let not_ready = graph.not_ready(id, issue);
        Self {
            id: id.clone(),
            title: issue.title.clone(),
            status: issue.status,
            issue_type: issue.issue_type,
            priority: issue.priority,
            tags: issue.tags.clone(),
            parent: issue.parent.clone(),
            blocked_by: issue.blocked_by.clone(),
            related: issue.related.clone(),
            discovered_from: issue.discovered_from.clone(),
            created_at: issue.created_at.clone(),
            closed_at: issue.closed_at.clone(),
            path: path.display().to_string(),
            checklist: issue.checklist(),
            ready: not_ready.is_empty(),
            not_ready,
        }
    }

    fn of(graph: &Graph, found: IssueRef) -> Self {
        Self::new(graph, found.id, found.issue, &found.file.path)
    }
}

impl IssueDetail {
    fn of(graph: &Graph, found: IssueRef) -> Self {
        let reverse = graph.reverse(found.id);
        let ids = |issues: Vec<IssueRef>| issues.into_iter().map(|r| r.id.clone()).collect();
        Self {
            issue: IssueView::of(graph, found),
            body: found.issue.body.clone(),
            children: ids(reverse.children),
            blocks: ids(reverse.blocks),
            related_from: ids(reverse.related_from),
            discovered: ids(reverse.discovered),
        }
    }
}

/// The view of a file that an operation has just written.
fn view_of(graph: &Graph, file: &IssueFile) -> Result<IssueView> {
    match (&file.id, &file.issue) {
        (Some(id), Some(issue)) => Ok(IssueView::new(graph, id, issue, &file.path)),
        _ => Err(file.unreadable()),
    }
}

/// A warning for each file that listings skip: unreadable files, and both
/// files of an ID present in `.zban/` and `.zban/archive/`.
// @mara implements REQ-DAMAGED-DATA
fn warnings(files: &[IssueFile], graph: &Graph) -> Vec<Warning> {
    let mut warnings = Vec::new();
    for file in files {
        let path = file.path.display().to_string();
        if file.issue.is_none() {
            let problem = file.problems.iter().find(|problem| problem.code.is_error());
            warnings.push(Warning {
                code: problem.map_or("unparseable", |problem| problem.code.as_str()),
                path: path.clone(),
                message: problem.map_or_else(
                    || "the file cannot be read".to_owned(),
                    |problem| problem.message.clone(),
                ),
            });
        }
        if let Some(id) = &file.id
            && matches!(graph.get(id), Some(Entry::Ambiguous))
        {
            warnings.push(Warning {
                code: "duplicate_id",
                path,
                message: format!("{id} names a file in both .zban/ and .zban/archive/"),
            });
        }
    }
    warnings
}

/// Applies the body edits of `request` in order: the new body, the
/// replacements, then the appended text (`DES-CLI`).
fn edit_body(body: &mut String, request: &UpdateRequest) -> Result<()> {
    if let Some(text) = &request.body {
        *body = stored(text);
    }
    for replacement in &request.replacements {
        *body = replace_once(body, replacement)?;
    }
    if let Some(text) = &request.append {
        let text = stored(text);
        if !text.is_empty() {
            let kept = body.trim_end();
            *body = if kept.is_empty() {
                text
            } else {
                format!("{kept}\n\n{text}")
            };
        }
    }
    Ok(())
}

/// Text as stored in a body: ending with exactly one LF, or empty
/// (`DES-ISSUE-FILE`).
fn stored(text: &str) -> String {
    let text = text.trim_end_matches(['\n', '\r']);
    if text.is_empty() {
        String::new()
    } else {
        format!("{text}\n")
    }
}

/// Replaces the one occurrence of `replacement.old` in `body`. Fails with
/// `text_not_found` or `text_ambiguous` otherwise; overlapping occurrences
/// count separately.
fn replace_once(body: &str, replacement: &Replacement) -> Result<String> {
    let old = replacement.old.as_str();
    let Some(first) = old.chars().next() else {
        return Err(Error::usage("the text to replace must not be empty"));
    };
    let Some(at) = body.find(old) else {
        return Err(Error::new(
            ErrorCode::TextNotFound,
            format!("the body does not contain {old:?}"),
        ));
    };
    if body[at + first.len_utf8()..].contains(old) {
        return Err(Error::new(
            ErrorCode::TextAmbiguous,
            format!(
                "the body contains {old:?} more than once; include more of the surrounding text"
            ),
        ));
    }
    Ok(format!(
        "{}{}{}",
        &body[..at],
        replacement.new,
        &body[at + old.len()..]
    ))
}

/// Fails with `usage` when a request both adds and removes one value.
fn both<T: PartialEq + fmt::Display>(what: &str, add: &[T], remove: &[T]) -> Result<()> {
    match add.iter().find(|value| remove.contains(value)) {
        Some(value) => Err(Error::usage(format!(
            "the {what} {value} is both added and removed"
        ))),
        None => Ok(()),
    }
}

/// Parses a vocabulary word into `slot` when one is given.
fn set<T: FromStr<Err = Error>>(slot: &mut T, value: Option<&str>) -> Result<()> {
    if let Some(value) = value {
        *slot = value.parse()?;
    }
    Ok(())
}

fn parse_all<T: FromStr<Err = Error>>(values: &[String]) -> Result<Vec<T>> {
    values.iter().map(|value| value.parse()).collect()
}

/// The issue an operation acts on. Fails with `not_found` for a value that
/// cannot be an ID.
fn named(prefix: &Prefix, value: &str) -> Result<Id> {
    Id::parse_input(prefix, value).ok_or_else(|| {
        Error::new(
            ErrorCode::NotFound,
            format!("issue {value:?} does not exist: IDs look like {prefix}-a1b2c3"),
        )
    })
}

/// A relation value. Fails with `invalid_reference` for a value that cannot
/// be an ID; [`Graph::check`] rejects IDs of unknown issues.
fn reference(prefix: &Prefix, key: &str, value: &str) -> Result<Id> {
    Id::parse_input(prefix, value).ok_or_else(|| {
        Error::new(
            ErrorCode::InvalidReference,
            format!("`{key}` names {value:?}, which is not an issue ID with prefix `{prefix}`"),
        )
    })
}

fn optional_reference(prefix: &Prefix, key: &str, value: Option<&str>) -> Result<Option<Id>> {
    value.map(|value| reference(prefix, key, value)).transpose()
}

/// Distinct relation values, in the order first given.
fn references(prefix: &Prefix, key: &str, values: &[String]) -> Result<Vec<Id>> {
    let ids = values
        .iter()
        .map(|value| reference(prefix, key, value))
        .collect::<Result<Vec<_>>>()?;
    Ok(distinct(ids))
}

fn distinct<T: PartialEq>(values: Vec<T>) -> Vec<T> {
    let mut kept = Vec::with_capacity(values.len());
    for value in values {
        if !kept.contains(&value) {
            kept.push(value);
        }
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replace(body: &str, old: &str, new: &str) -> Result<String> {
        let replacement = Replacement {
            old: old.into(),
            new: new.into(),
        };
        replace_once(body, &replacement)
    }

    #[test]
    fn stored_text_ends_with_one_line_feed() {
        assert_eq!(stored("a"), "a\n");
        assert_eq!(stored("a\n\n\n"), "a\n");
        assert_eq!(stored("a\r\n"), "a\n");
        assert_eq!(stored("\n\n"), "");
        assert_eq!(stored(""), "");
    }

    #[test]
    fn replacements_need_exactly_one_occurrence() {
        assert_eq!(replace("- [ ] a\n", "[ ]", "[x]").unwrap(), "- [x] a\n");
        assert_eq!(replace("é é\n", "é ", "e ").unwrap(), "e é\n");
        let code = |old: &str| replace("aaa b b\n", old, "x").unwrap_err().code;
        assert_eq!(code("c"), ErrorCode::TextNotFound);
        assert_eq!(code("b"), ErrorCode::TextAmbiguous);
        assert_eq!(code("aa"), ErrorCode::TextAmbiguous, "overlapping");
        assert_eq!(code(""), ErrorCode::Usage);
    }

    #[test]
    fn appending_joins_after_one_blank_line() {
        let append = |body: &str, text: &str| {
            let mut body = body.to_owned();
            let request = UpdateRequest {
                append: Some(text.into()),
                ..Default::default()
            };
            edit_body(&mut body, &request).unwrap();
            body
        };
        assert_eq!(append("a\n", "b"), "a\n\nb\n");
        assert_eq!(append("a", "b\n\n"), "a\n\nb\n");
        assert_eq!(append("a\n\n\n", "b"), "a\n\nb\n");
        assert_eq!(append("", "b"), "b\n");
        assert_eq!(append("a\n", ""), "a\n");
    }
}
