//! Output for humans, printed without `--json`.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use zban::operations::{Initialized, IssueDetail, IssueView};
use zban::{Finding, Id};

pub fn initialized(initialized: &Initialized) -> String {
    format!(
        "Created {} with ID prefix {}\n",
        initialized.path, initialized.prefix
    )
}

/// The line printed after a mutation, such as `Created zb-a1b2c3: Title`.
pub fn changed(verb: &str, issue: &IssueView) -> String {
    format!("{verb} {}: {}\n", issue.id, issue.title)
}

/// One line per issue: ID, status, priority, type, title and checklist
/// progress.
pub fn list(issues: &[IssueView]) -> String {
    if issues.is_empty() {
        return "No issues.\n".to_owned();
    }
    let mut text = String::new();
    for issue in issues {
        let _ = write!(
            text,
            "{}  {:<11}  {:<8}  {:<7}  {}",
            issue.id,
            issue.status.as_str(),
            issue.priority.as_str(),
            issue.issue_type.as_str(),
            issue.title
        );
        if issue.checklist.total > 0 {
            let _ = write!(
                text,
                "  [{}/{}]",
                issue.checklist.checked, issue.checklist.total
            );
        }
        text.push('\n');
    }
    text
}

/// Each issue's fields, one per line, followed by its body. Issues are
/// separated by a blank line.
pub fn show(issues: &[IssueDetail]) -> String {
    let shown: Vec<_> = issues.iter().map(detail).collect();
    shown.join("\n")
}

fn detail(detail: &IssueDetail) -> String {
    let issue = &detail.issue;
    let mut text = format!("{}: {}\n", issue.id, issue.title);
    let mut field = |key: &str, value: &str| {
        if !value.is_empty() {
            let _ = writeln!(text, "{key}: {value}");
        }
    };
    let ids = |ids: &[Id]| ids.iter().map(Id::as_str).collect::<Vec<_>>().join(", ");
    field("status", issue.status.as_str());
    field("type", issue.issue_type.as_str());
    field("priority", issue.priority.as_str());
    field("tags", &issue.tags.join(", "));
    field("parent", issue.parent.as_ref().map_or("", Id::as_str));
    field("blocked_by", &ids(&issue.blocked_by));
    field("related", &ids(&issue.related));
    field(
        "discovered_from",
        issue.discovered_from.as_ref().map_or("", Id::as_str),
    );
    field("children", &ids(&detail.children));
    field("blocks", &ids(&detail.blocks));
    field("related_from", &ids(&detail.related_from));
    field("discovered", &ids(&detail.discovered));
    field("created_at", issue.created_at.as_str());
    field(
        "closed_at",
        issue.closed_at.as_ref().map_or("", |t| t.as_str()),
    );
    if issue.checklist.total > 0 {
        let progress = format!(
            "{}/{} checked",
            issue.checklist.checked, issue.checklist.total
        );
        field("checklist", &progress);
    }
    let reasons: Vec<_> = issue.not_ready.iter().map(ToString::to_string).collect();
    let ready = if issue.ready {
        "yes".to_owned()
    } else {
        format!("no ({})", reasons.join(", "))
    };
    field("ready", &ready);
    field("path", &issue.path);
    if !detail.body.is_empty() {
        text.push('\n');
        text.push_str(&detail.body);
        if !text.ends_with('\n') {
            text.push('\n');
        }
    }
    text
}

/// One line per finding: `<path>: <severity>: <message> [<code>]`, with the
/// path relative to the working directory when it lies below it. A clean
/// check prints nothing.
pub fn findings(findings: &[Finding]) -> String {
    // Finding paths are canonical, so compare them with the canonical
    // working directory.
    let cwd = std::env::current_dir().and_then(fs::canonicalize).ok();
    let mut text = String::new();
    for finding in findings {
        let path = Path::new(&finding.path);
        let shown = cwd
            .as_deref()
            .and_then(|cwd| path.strip_prefix(cwd).ok())
            .unwrap_or(path);
        let _ = writeln!(
            text,
            "{}: {}: {} [{}]",
            shown.display(),
            finding.severity,
            finding.message,
            finding.code
        );
    }
    text
}
