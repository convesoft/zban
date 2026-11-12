//! Status changes, checklists and archiving (`DES-LIFECYCLE`).

use crate::error::{Error, ErrorCode, Result};
use crate::id::Id;
use crate::issue::{Issue, Status};
use crate::markdown;
use crate::store::{IssueFile, Project};
use crate::timestamp::Timestamp;

/// Checklist progress: the task items in a body and how many are checked.
/// Items inside fenced code blocks do not count.
// @mara implements DES-LIFECYCLE
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Checklist {
    pub checked: usize,
    pub total: usize,
}

impl Checklist {
    pub fn of(body: &str) -> Self {
        let mut checklist = Self::default();
        let lines = markdown::lines_outside_fences(body);
        for checked in lines.filter_map(|(_, line)| markdown::task_item(line)) {
            checklist.total += 1;
            checklist.checked += usize::from(checked);
        }
        checklist
    }

    pub fn unchecked(self) -> usize {
        self.total - self.checked
    }
}

impl Issue {
    /// The checklist progress of the body, which listings report.
    pub fn checklist(&self) -> Checklist {
        Checklist::of(&self.body)
    }
}

impl Project {
    /// Writes `issue` as the new state of `file`, one of the loaded `files`,
    /// following `DES-LIFECYCLE`. Nothing is written when the change is
    /// refused.
    ///
    /// - Changing the status to `done` fails with `unchecked_items` while
    ///   `issue` has unchecked task items, and changing it to `done` or
    ///   `cancelled` fails with `open_children` while an issue in `files`
    ///   whose parent is this one is open. `force` skips both rules. Other
    ///   issues are never changed.
    /// - `closed_at` is set when the status changes from open to closed and
    ///   removed when it changes back; otherwise it keeps its value. Any
    ///   `closed_at` in `issue` is ignored.
    /// - The file is rewritten in place. After a change between open and
    ///   closed it is then renamed into the directory its status requires.
    ///
    /// Fails with `unreadable` when `file` holds no readable issue.
    // @mara implements DES-LIFECYCLE
    pub fn save(
        &self,
        files: &[IssueFile],
        file: &IssueFile,
        mut issue: Issue,
        force: bool,
    ) -> Result<IssueFile> {
        let (Some(id), Some(before)) = (&file.id, &file.issue) else {
            return Err(file.unreadable());
        };
        if !force && issue.status != before.status {
            refuse_unfinished(files, id, &issue)?;
        }
        let was_closed = before.status.is_closed();
        let crosses = was_closed != issue.status.is_closed();
        issue.closed_at = match (was_closed, issue.status.is_closed()) {
            (_, false) => None,
            (false, true) => Some(Timestamp::now()),
            (true, true) => before.closed_at.clone(),
        };
        if crosses {
            // Refuse a taken target name before writing anything.
            self.destination(file, issue.status.is_closed())?;
        }
        let written = self.replace(file, &issue)?;
        if crosses {
            self.place(&written)
        } else {
            Ok(written)
        }
    }
}

/// Refuses creating `issue` as `done` with unchecked task items, which
/// cannot be forced (`DES-LIFECYCLE`). A new issue has no children.
// @mara implements DES-LIFECYCLE
pub(crate) fn refuse_unfinished_creation(issue: &Issue) -> Result<()> {
    refuse_unchecked(
        issue,
        "the new issue",
        "create it open and then force setting done",
    )
}

/// Refuses closing `issue` while its work is unfinished: `done` with
/// unchecked task items, or `done` or `cancelled` with open children.
fn refuse_unfinished(files: &[IssueFile], id: &Id, issue: &Issue) -> Result<()> {
    refuse_unchecked(issue, id.as_str(), "force setting done")?;
    if issue.status.is_closed() {
        let children = open_children(files, id);
        if !children.is_empty() {
            let names: Vec<_> = children.iter().map(Id::as_str).collect();
            return Err(Error::new(
                ErrorCode::OpenChildren,
                format!(
                    "{id} has open children: {}; close or cancel each child first, or force setting {}",
                    names.join(", "),
                    issue.status
                ),
            ));
        }
    }
    Ok(())
}

/// Fails with `unchecked_items` when `issue` is `done` with unchecked task
/// items. The message names the issue as `name` and offers `remedy` besides
/// checking the items.
fn refuse_unchecked(issue: &Issue, name: &str, remedy: &str) -> Result<()> {
    if issue.status != Status::Done {
        return Ok(());
    }
    match issue.checklist().unchecked() {
        0 => Ok(()),
        unchecked => {
            let items = if unchecked == 1 { "item" } else { "items" };
            Err(Error::new(
                ErrorCode::UncheckedItems,
                format!(
                    "{name} has {unchecked} unchecked task {items}; check them first, or {remedy}"
                ),
            ))
        }
    }
}

/// The open issues whose parent is `id`, sorted by ID.
fn open_children(files: &[IssueFile], id: &Id) -> Vec<Id> {
    let mut children: Vec<Id> = files
        .iter()
        .filter_map(|file| {
            let child = file.id.as_ref()?;
            let issue = file.issue.as_ref()?;
            (child != id && issue.parent.as_ref() == Some(id) && !issue.status.is_closed())
                .then(|| child.clone())
        })
        .collect();
    children.sort();
    children.dedup();
    children
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checklist_counts_task_items_outside_fences() {
        let body = "\
Intro with - [ ] inline text.

- [x] one
  - [ ] nested
* [X] two
+ [ ] three

```markdown
- [ ] example
```

~~~
- [x] example
~~~
";
        assert_eq!(
            Checklist::of(body),
            Checklist {
                checked: 2,
                total: 4
            }
        );
        assert_eq!(Checklist::of(body).unchecked(), 2);
        assert_eq!(Checklist::of(""), Checklist::default());
    }
}
