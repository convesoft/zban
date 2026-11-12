//! `VER-LIFECYCLE`: closing, archiving and reopening through the core library.

mod common;

use std::fs;
use std::path::PathBuf;

use common::{temp_dir, write_files};
use tempfile::TempDir;
use zban::{Checklist, ErrorCode, Id, Issue, IssueFile, Project, Status, Timestamp};

const CONFIG: &str = "format_version = 1\nprefix = \"zb\"\n";

/// A project holding `files`, in a directory that lasts as long as the
/// returned [`TempDir`].
fn fixture(files: &[(&str, &str)]) -> (TempDir, Project) {
    let dir = temp_dir();
    write_files(dir.path(), &[(".zban/config.toml", CONFIG)]);
    write_files(dir.path(), files);
    let project = Project::open(dir.path()).unwrap();
    (dir, project)
}

/// An issue file with `status`, extra header lines and `body`.
fn file_text(status: &str, header: &str, body: &str) -> String {
    let closed = if matches!(status, "done" | "cancelled") {
        "closed_at: 2026-11-12T08:00:00Z\n"
    } else {
        ""
    };
    let body = if body.is_empty() {
        String::new()
    } else {
        format!("\n{body}")
    };
    format!(
        "---\ntitle: Issue\nstatus: {status}\ntype: task\n{header}created_at: 2026-11-11T20:00:00Z\n{closed}---\n{body}"
    )
}

fn load(project: &Project) -> Vec<IssueFile> {
    project.load().unwrap()
}

fn find<'a>(files: &'a [IssueFile], id: &str) -> &'a IssueFile {
    files
        .iter()
        .find(|file| file.id.as_ref().map(Id::as_str) == Some(id))
        .unwrap_or_else(|| panic!("{id} is loaded"))
}

/// Saves `id` with its issue changed by `change`.
fn save(
    project: &Project,
    id: &str,
    force: bool,
    change: impl FnOnce(&mut Issue),
) -> zban::Result<IssueFile> {
    let files = load(project);
    let file = find(&files, id);
    let mut issue = file
        .issue
        .clone()
        .unwrap_or_else(|| panic!("{id}: {:?}", file.problems));
    change(&mut issue);
    project.save(&files, file, issue, force)
}

fn set_status(project: &Project, id: &str, status: Status, force: bool) -> zban::Result<IssueFile> {
    save(project, id, force, |issue| issue.status = status)
}

fn open_path(project: &Project, id: &str) -> PathBuf {
    project.dir().join(format!("{id}.md"))
}

fn archive_path(project: &Project, id: &str) -> PathBuf {
    project.archive_dir().join(format!("{id}.md"))
}

/// Every file under `.zban/` with its content.
fn snapshot(project: &Project) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files = Vec::new();
    for dir in [project.dir(), project.archive_dir()] {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_file() {
                files.push((path.clone(), fs::read(&path).unwrap()));
            }
        }
    }
    files.sort();
    files
}

// @mara implements VER-LIFECYCLE
#[test]
fn done_with_an_unchecked_task_item_fails_and_changes_nothing_unless_forced() {
    let body = "- [x] Read frontmatter\n- [ ] Preserve unknown keys\n";
    let (_dir, project) = fixture(&[(".zban/zb-aaaaaa.md", &file_text("in-progress", "", body))]);
    let before = snapshot(&project);

    let error = set_status(&project, "zb-aaaaaa", Status::Done, false).unwrap_err();
    assert_eq!(error.code, ErrorCode::UncheckedItems);
    assert!(error.message.contains("zb-aaaaaa"), "{}", error.message);
    assert!(error.message.contains("1 unchecked"), "{}", error.message);
    assert_eq!(snapshot(&project), before);

    let closed = set_status(&project, "zb-aaaaaa", Status::Done, true).unwrap();
    assert_eq!(closed.path, archive_path(&project, "zb-aaaaaa"));
    let issue = closed.issue.unwrap();
    assert_eq!(issue.status, Status::Done);
    assert_eq!(issue.body, body);
    assert!(!open_path(&project, "zb-aaaaaa").exists());
}

// @mara implements VER-LIFECYCLE
#[test]
fn task_items_inside_fenced_code_blocks_do_not_count() {
    let body = "\
- [x] Parse fences
  ```markdown
  - [ ] an example inside a list item
  ```

~~~~text
- [ ] an example
~~~
* [ ] still inside: the closing fence is too short
~~~~

```
+ [ ] an unclosed fence runs to the end of the body
";
    let (_dir, project) = fixture(&[(".zban/zb-aaaaaa.md", &file_text("todo", "", body))]);
    let files = load(&project);
    assert_eq!(
        find(&files, "zb-aaaaaa")
            .issue
            .as_ref()
            .unwrap()
            .checklist(),
        Checklist {
            checked: 1,
            total: 1
        }
    );
    let closed = set_status(&project, "zb-aaaaaa", Status::Done, false).unwrap();
    assert_eq!(closed.issue.unwrap().status, Status::Done);
}

// @mara implements VER-LIFECYCLE
#[test]
fn fences_open_blocks_only_where_markdown_opens_them() {
    let body = "\
Indented four columns outside a list item, a fence line opens no block:

    ```
    cargo run -- parse

- [x] Nested example
  - [x] Two list items deep, a fence may be indented to their content
    ```markdown
    - [ ] an example
    ```
- [ ] Four columns more than the item's content open no block:
      ```
- [ ] Release
";
    let (_dir, project) = fixture(&[(".zban/zb-aaaaaa.md", &file_text("todo", "", body))]);
    let files = load(&project);
    assert_eq!(
        find(&files, "zb-aaaaaa")
            .issue
            .as_ref()
            .unwrap()
            .checklist(),
        Checklist {
            checked: 2,
            total: 4
        }
    );
    let before = snapshot(&project);
    let error = set_status(&project, "zb-aaaaaa", Status::Done, false).unwrap_err();
    assert_eq!(error.code, ErrorCode::UncheckedItems);
    assert_eq!(snapshot(&project), before);
}

// @mara implements VER-LIFECYCLE
#[test]
fn every_task_item_form_counts() {
    for item in [
        "- [ ] dash",
        "* [ ] star",
        "+ [ ] plus",
        "    - [ ] indented",
        "\t- [ ] tab",
    ] {
        let body = format!("- [x] Checked\n{item}\n");
        let (_dir, project) = fixture(&[(".zban/zb-aaaaaa.md", &file_text("todo", "", &body))]);
        let error = set_status(&project, "zb-aaaaaa", Status::Done, false).unwrap_err();
        assert_eq!(error.code, ErrorCode::UncheckedItems, "{item:?}");
    }
    let body = "- [x] lower\n* [X] upper\n1. [ ] ordered lists are not task items\n";
    let (_dir, project) = fixture(&[(".zban/zb-aaaaaa.md", &file_text("todo", "", body))]);
    set_status(&project, "zb-aaaaaa", Status::Done, false).unwrap();
}

// @mara implements VER-LIFECYCLE
#[test]
fn closing_with_an_open_child_fails_names_it_and_changes_nothing() {
    let (_dir, project) = fixture(&[
        (".zban/zb-aaaaaa.md", &file_text("todo", "", "")),
        (
            ".zban/zb-bbbbbb.md",
            &file_text("todo", "parent: zb-aaaaaa\n", ""),
        ),
        (
            ".zban/zb-cccccc.md",
            &file_text("draft", "parent: zb-aaaaaa\n", ""),
        ),
        (
            ".zban/archive/zb-dddddd.md",
            &file_text("done", "parent: zb-aaaaaa\n", ""),
        ),
        (".zban/zb-eeeeee.md", &file_text("in-progress", "", "")),
    ]);
    let before = snapshot(&project);
    for status in [Status::Done, Status::Cancelled] {
        let error = set_status(&project, "zb-aaaaaa", status, false).unwrap_err();
        assert_eq!(error.code, ErrorCode::OpenChildren, "{status}");
        assert!(
            error.message.contains("zb-bbbbbb, zb-cccccc"),
            "{}",
            error.message
        );
        assert!(!error.message.contains("zb-dddddd"), "{}", error.message);
        assert!(!error.message.contains("zb-eeeeee"), "{}", error.message);
        assert_eq!(snapshot(&project), before);
    }

    // Closing the children first lets the parent close without force.
    for child in ["zb-bbbbbb", "zb-cccccc"] {
        set_status(&project, child, Status::Cancelled, false).unwrap();
    }
    set_status(&project, "zb-aaaaaa", Status::Done, false).unwrap();
}

// @mara implements VER-LIFECYCLE
#[test]
fn forced_close_leaves_the_open_child_unchanged() {
    let child = file_text("in-progress", "parent: zb-aaaaaa\n", "- [ ] Child work\n");
    let (_dir, project) = fixture(&[
        (
            ".zban/zb-aaaaaa.md",
            &file_text("todo", "", "- [ ] Parent work\n"),
        ),
        (".zban/zb-bbbbbb.md", &child),
    ]);
    let closed = set_status(&project, "zb-aaaaaa", Status::Cancelled, true).unwrap();
    assert_eq!(closed.path, archive_path(&project, "zb-aaaaaa"));
    assert_eq!(
        fs::read_to_string(open_path(&project, "zb-bbbbbb")).unwrap(),
        child
    );

    // Cancelling with unchecked task items needs no force; only done does.
    let (_dir, project) = fixture(&[(
        ".zban/zb-aaaaaa.md",
        &file_text("todo", "", "- [ ] Never\n"),
    )]);
    set_status(&project, "zb-aaaaaa", Status::Cancelled, false).unwrap();
}

// @mara implements VER-LIFECYCLE
#[test]
fn closing_archives_and_reopening_moves_back() {
    let original = "---\ntitle: Issue\nstatus: in-progress\ntype: task\ncreated_at: 2026-11-11T20:00:00Z\ncustom: kept\n---\n\n- [x] Done\n\nBody bytes   \r\nstay as they are";
    let (_dir, project) = fixture(&[(".zban/zb-aaaaaa.md", original)]);
    let start = Timestamp::now();

    let closed = set_status(&project, "zb-aaaaaa", Status::Done, false).unwrap();
    assert_eq!(closed.path, archive_path(&project, "zb-aaaaaa"));
    assert!(closed.archived);
    assert!(!open_path(&project, "zb-aaaaaa").exists());
    let closed_at = closed.issue.as_ref().unwrap().closed_at.clone().unwrap();
    assert!(start <= closed_at && closed_at <= Timestamp::now());
    assert_eq!(
        fs::read_to_string(&closed.path).unwrap(),
        original
            .replace("status: in-progress", "status: done")
            .replace(
                "Z\ncustom",
                &format!("Z\nclosed_at: {}\ncustom", closed_at.as_str())
            )
    );

    let files = load(&project);
    let archived = find(&files, "zb-aaaaaa");
    assert!(archived.archived);
    assert_eq!(archived.issue, closed.issue);

    let reopened = set_status(&project, "zb-aaaaaa", Status::Todo, false).unwrap();
    assert_eq!(reopened.path, open_path(&project, "zb-aaaaaa"));
    assert!(!reopened.archived);
    assert!(!archive_path(&project, "zb-aaaaaa").exists());
    assert_eq!(reopened.issue.as_ref().unwrap().closed_at, None);
    assert_eq!(
        fs::read_to_string(&reopened.path).unwrap(),
        original.replace("status: in-progress\n", "status: todo\n")
    );
}

// @mara implements VER-LIFECYCLE
#[test]
fn one_update_can_check_the_last_item_append_a_summary_and_close() {
    let (_dir, project) = fixture(&[(
        ".zban/zb-aaaaaa.md",
        &file_text("in-progress", "", "- [x] One\n- [ ] Two\n"),
    )]);
    let closed = save(&project, "zb-aaaaaa", false, |issue| {
        issue.body = issue.body.replace("- [ ] Two", "- [x] Two");
        issue
            .body
            .push_str("\n## Summary of Changes\n\nBoth done.\n");
        issue.status = Status::Done;
    })
    .unwrap();
    let issue = closed.issue.unwrap();
    assert_eq!(
        issue.checklist(),
        Checklist {
            checked: 2,
            total: 2
        }
    );
    assert_eq!(
        issue.body,
        "- [x] One\n- [x] Two\n\n## Summary of Changes\n\nBoth done.\n"
    );
    assert!(closed.archived);

    // The rules look at the issue as the request leaves it, so unchecking an
    // item in the same request that sets done is refused.
    let (_dir, project) = fixture(&[(".zban/zb-aaaaaa.md", &file_text("todo", "", "- [x] One\n"))]);
    let error = save(&project, "zb-aaaaaa", false, |issue| {
        issue.body = "- [ ] One\n".into();
        issue.status = Status::Done;
    })
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::UncheckedItems);
}

// @mara implements VER-LIFECYCLE
#[test]
fn changes_among_closed_statuses_keep_closed_at_and_the_archive() {
    let (_dir, project) = fixture(&[(
        ".zban/archive/zb-aaaaaa.md",
        &file_text("done", "", "- [ ] Left open by a forced close\n"),
    )]);
    let cancelled = set_status(&project, "zb-aaaaaa", Status::Cancelled, false).unwrap();
    assert_eq!(cancelled.path, archive_path(&project, "zb-aaaaaa"));
    let closed_at = Timestamp::parse("2026-11-12T08:00:00Z");
    assert_eq!(cancelled.issue.unwrap().closed_at, closed_at);

    // Changing the status to done again checks the task items again.
    let error = set_status(&project, "zb-aaaaaa", Status::Done, false).unwrap_err();
    assert_eq!(error.code, ErrorCode::UncheckedItems);

    // Edits that keep the status apply no closing rule, and a closed_at given
    // with the request is ignored.
    let edited = save(&project, "zb-aaaaaa", false, |issue| {
        issue.title = "Renamed".into();
        issue.closed_at = None;
    })
    .unwrap();
    assert_eq!(edited.path, archive_path(&project, "zb-aaaaaa"));
    assert_eq!(edited.issue.unwrap().closed_at, closed_at);
}

// @mara implements VER-LIFECYCLE
#[test]
fn archived_issues_stay_loaded_and_closed_children_do_not_block() {
    let (_dir, project) = fixture(&[
        (".zban/zb-aaaaaa.md", &file_text("todo", "", "")),
        (
            ".zban/zb-bbbbbb.md",
            &file_text(
                "todo",
                "parent: zb-aaaaaa\nblocked_by:\n  - zb-cccccc\n",
                "- [x] One\n- [ ] Two\n",
            ),
        ),
        (".zban/zb-cccccc.md", &file_text("in-progress", "", "")),
    ]);
    set_status(&project, "zb-bbbbbb", Status::Cancelled, false).unwrap();
    set_status(&project, "zb-aaaaaa", Status::Done, false).unwrap();

    let files = load(&project);
    let child = find(&files, "zb-bbbbbb");
    assert!(child.archived);
    let issue = child.issue.as_ref().unwrap();
    assert_eq!(issue.parent.as_ref().map(Id::as_str), Some("zb-aaaaaa"));
    assert_eq!(
        issue.blocked_by.iter().map(Id::as_str).collect::<Vec<_>>(),
        ["zb-cccccc"]
    );
    assert_eq!(
        issue.checklist(),
        Checklist {
            checked: 1,
            total: 2
        }
    );
    assert!(find(&files, "zb-aaaaaa").archived);
    assert!(!find(&files, "zb-cccccc").archived);

    // A reopened child is open again, so its parent cannot be reopened and
    // closed again without force.
    set_status(&project, "zb-aaaaaa", Status::InProgress, false).unwrap();
    set_status(&project, "zb-bbbbbb", Status::Todo, false).unwrap();
    let error = set_status(&project, "zb-aaaaaa", Status::Done, false).unwrap_err();
    assert_eq!(error.code, ErrorCode::OpenChildren);
}

// @mara implements VER-LIFECYCLE
#[test]
fn a_taken_target_name_fails_with_ambiguous_id_and_changes_nothing() {
    let (_dir, project) = fixture(&[
        (".zban/zb-aaaaaa.md", &file_text("todo", "", "")),
        (".zban/archive/zb-aaaaaa.md", &file_text("done", "", "")),
    ]);
    let before = snapshot(&project);
    let files = load(&project);
    let open = files.iter().find(|file| !file.archived).unwrap();
    let mut issue = open.issue.clone().unwrap();
    issue.status = Status::Done;
    let error = project.save(&files, open, issue, false).unwrap_err();
    assert_eq!(error.code, ErrorCode::AmbiguousId);
    assert_eq!(snapshot(&project), before);
}

// @mara implements VER-LIFECYCLE
#[test]
fn a_misplaced_file_ends_where_its_new_status_belongs() {
    // A close whose move was interrupted leaves a done issue in `.zban/`.
    // Reopening it needs no move, and closing it again archives it.
    let (_dir, project) = fixture(&[(".zban/zb-aaaaaa.md", &file_text("done", "", ""))]);
    let reopened = set_status(&project, "zb-aaaaaa", Status::Todo, false).unwrap();
    assert_eq!(reopened.path, open_path(&project, "zb-aaaaaa"));
    let closed = set_status(&project, "zb-aaaaaa", Status::Done, false).unwrap();
    assert_eq!(closed.path, archive_path(&project, "zb-aaaaaa"));

    // Placement on its own moves a misplaced file, as `check --fix` will.
    write_files(
        project.root(),
        &[(".zban/archive/zb-bbbbbb.md", &file_text("todo", "", ""))],
    );
    let files = load(&project);
    let placed = project.place(find(&files, "zb-bbbbbb")).unwrap();
    assert_eq!(placed.path, open_path(&project, "zb-bbbbbb"));
    assert_eq!(load(&project).len(), 2);
}

// @mara implements VER-LIFECYCLE
#[test]
fn an_unreadable_file_fails_with_unreadable() {
    let (_dir, project) = fixture(&[(
        ".zban/zb-aaaaaa.md",
        "---\ntitle: T\nstatus: open\ntype: task\ncreated_at: 2026-11-11T20:00:00Z\n---\n",
    )]);
    let files = load(&project);
    let file = find(&files, "zb-aaaaaa");
    let issue = Issue::new("T", Timestamp::now());
    let error = project.save(&files, file, issue, false).unwrap_err();
    assert_eq!(error.code, ErrorCode::Unreadable);
    assert!(error.message.contains("status"), "{}", error.message);
}
