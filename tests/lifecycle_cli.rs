//! `VER-LIFECYCLE` through the real `zban` binary: closing, archiving and
//! reopening, and how archived issues appear in `show`, search, relations and
//! listings. `tests/lifecycle.rs` checks the closing rules in more depth
//! through the library.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::{snapshot, stdout, temp_dir, write_files, zban};
use serde_json::{Value, json};
use zban::Timestamp;

const CONFIG: &str = "format_version = 1\nprefix = \"zb\"\n";

// The fixture of `fixture`: one letter repeated, named after the issue.
const E: &str = "zb-eeeeee"; // epic, the parent of A
const A: &str = "zb-aaaaaa"; // high task with a checklist, related to C
const B: &str = "zb-bbbbbb"; // blocked by A
const C: &str = "zb-cccccc"; // the other end of A's related link
const D: &str = "zb-dddddd"; // child of A

/// One checked and one unchecked task item, and an example in a fenced code
/// block that is no task item.
const A_BODY: &str =
    "Read the Quasar format.\n\n- [x] Read\n- [ ] Write\n\n```markdown\n- [ ] an example\n```\n";

/// A hand-written open issue file created `second` seconds into 2026.
fn issue(title: &str, kind: &str, header: &str, second: u32, body: &str) -> String {
    let body = if body.is_empty() {
        String::new()
    } else {
        format!("\n{body}")
    };
    format!(
        "---\ntitle: {title}\nstatus: todo\ntype: {kind}\n{header}created_at: 2026-01-01T00:00:{second:02}Z\n---\n{body}"
    )
}

fn open(id: &str) -> String {
    format!(".zban/{id}.md")
}

fn archived(id: &str) -> String {
    format!(".zban/archive/{id}.md")
}

fn fixture(name: &str) -> PathBuf {
    let root = temp_dir(&format!("lifecycle-cli-{name}"));
    write_files(
        &root,
        &[
            (".zban/config.toml", CONFIG),
            (&open(E), &issue("Release epic", "epic", "", 1, "")),
            (
                &open(A),
                &issue(
                    "Parse files",
                    "task",
                    &format!("priority: high\nparent: {E}\nrelated:\n  - {C}\n"),
                    2,
                    A_BODY,
                ),
            ),
            (
                &open(B),
                &issue(
                    "Write files",
                    "task",
                    &format!("blocked_by:\n  - {A}\n"),
                    3,
                    "",
                ),
            ),
            (&open(C), &issue("Document files", "task", "", 4, "")),
            (
                &open(D),
                &issue("Parse headers", "task", &format!("parent: {A}\n"), 5, ""),
            ),
        ],
    );
    root
}

/// Runs zban with `--json` and returns the exit status and the one JSON
/// document it printed.
fn run_json(dir: &Path, args: &[&str]) -> (i32, Value) {
    let args = [&["--json"], args].concat();
    let output = zban(dir, &args);
    let value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{args:?}: {error}: {:?}", stdout(&output)));
    (output.status.code().unwrap(), value)
}

/// Runs a command that must succeed and returns its JSON document.
fn ok(dir: &Path, args: &[&str]) -> Value {
    let (status, value) = run_json(dir, args);
    assert_eq!(status, 0, "{args:?}: {value}");
    value
}

/// Runs a command that must fail with exit status 1 and returns its error
/// object.
fn fails(dir: &Path, args: &[&str]) -> Value {
    let (status, value) = run_json(dir, args);
    assert_eq!(status, 1, "{args:?}: {value}");
    value["error"].clone()
}

/// The IDs of the issues in a `list` or `show` document.
fn ids(value: &Value) -> Vec<String> {
    value["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|issue| issue["id"].as_str().unwrap().to_owned())
        .collect()
}

fn listed(dir: &Path, args: &[&str]) -> Vec<String> {
    ids(&ok(dir, &[&["list"], args].concat()))
}

/// The one issue `show` prints for `id`.
fn shown(dir: &Path, id: &str) -> Value {
    let value = ok(dir, &["show", id]);
    assert_eq!(ids(&value), [id]);
    value["issues"][0].clone()
}

/// The issue `id` in a `list` document.
fn in_listing<'a>(listing: &'a Value, id: &str) -> &'a Value {
    listing["issues"]
        .as_array()
        .unwrap()
        .iter()
        .find(|issue| issue["id"] == id)
        .unwrap_or_else(|| panic!("{id} is listed: {listing}"))
}

/// The absolute path zban reports for `path`, relative to the project root.
fn reported(root: &Path, path: &str) -> String {
    fs::canonicalize(root)
        .unwrap()
        .join(path)
        .display()
        .to_string()
}

/// Every file under `.zban/` with its content. Directories are left out,
/// because closing creates `.zban/archive/` when it is missing.
fn files(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let dir = root.join(".zban");
    snapshot(&dir, &[])
        .into_iter()
        .filter(|(path, _)| dir.join(path).is_file())
        .collect()
}

fn read(root: &Path, path: &str) -> String {
    fs::read_to_string(root.join(path)).unwrap()
}

// @mara implements VER-LIFECYCLE
#[test]
fn closing_and_reopening_follow_the_rules_and_move_the_file() {
    let root = fixture("close");
    let original = read(&root, &open(A));
    let child = read(&root, &open(D));
    let before = files(&root);

    // The fenced example does not count, so one item is unchecked.
    let error = fails(&root, &["update", A, "-s", "done"]);
    assert_eq!(error["code"], "unchecked_items");
    let message = error["message"].as_str().unwrap();
    assert!(message.contains("1 unchecked task item;"), "{message}");
    assert_eq!(files(&root), before);

    // Cancelling with unchecked items is allowed, but A has an open child.
    let error = fails(&root, &["update", A, "-s", "cancelled"]);
    assert_eq!(error["code"], "open_children");
    let message = error["message"].as_str().unwrap();
    assert!(
        message.contains(&format!("open children: {D};")),
        "{message}"
    );
    assert_eq!(files(&root), before);

    let start = Timestamp::now();
    let closed = ok(&root, &["update", A, "-s", "done", "--force"]);
    let closed_at = closed["closed_at"].as_str().unwrap().to_owned();
    assert!(
        start.as_str() <= closed_at.as_str() && closed_at.as_str() <= Timestamp::now().as_str()
    );
    assert_eq!(closed["status"], "done");
    assert_eq!(closed["path"], reported(&root, &archived(A)));
    assert!(!root.join(open(A)).exists());
    assert_eq!(
        read(&root, &archived(A)),
        original
            .replace("status: todo\n", "status: done\n")
            .replace("Z\n---\n", &format!("Z\nclosed_at: {closed_at}\n---\n"))
    );
    assert_eq!(read(&root, &open(D)), child);

    // Reopening restores the file exactly as it was.
    let reopened = ok(&root, &["update", A, "-s", "todo"]);
    assert_eq!(reopened["closed_at"], Value::Null);
    assert_eq!(reopened["path"], reported(&root, &open(A)));
    assert_eq!(files(&root), before);

    // Once the child is closed, one request can check the last item, append
    // a summary and close without force.
    ok(&root, &["update", D, "-s", "cancelled"]);
    assert!(root.join(archived(D)).is_file());
    let done = ok(
        &root,
        &[
            "update",
            A,
            "--replace-old",
            "- [ ] Write",
            "--replace-new",
            "- [x] Write",
            "--append",
            "## Summary of Changes\n\nBoth written.",
            "-s",
            "done",
        ],
    );
    assert_eq!(done["checklist"], json!({"checked": 2, "total": 2}));
    assert_eq!(done["path"], reported(&root, &archived(A)));
    assert!(read(&root, &archived(A)).ends_with("## Summary of Changes\n\nBoth written.\n"));
}

// @mara implements VER-LIFECYCLE
#[test]
fn creating_done_with_an_unchecked_item_fails_and_cannot_be_forced() {
    let root = fixture("create-done");
    let before = files(&root);

    // The fenced example does not count, so one item is unchecked.
    let create_done = ["create", "Ship it", "-s", "done", "--body", A_BODY];
    let error = fails(&root, &create_done);
    assert_eq!(error["code"], "unchecked_items");
    let message = error["message"].as_str().unwrap();
    assert!(
        message.starts_with("the new issue has 1 unchecked task item;"),
        "{message}"
    );
    assert_eq!(files(&root), before);

    // `create` has no force.
    let (status, value) = run_json(&root, &[&create_done[..], &["--force"]].concat());
    assert_eq!(status, 2, "{value}");
    assert_eq!(value["error"]["code"], "usage");
    assert_eq!(files(&root), before);

    // Checked items, cancelling and other statuses are not restricted.
    let checked = A_BODY.replace("- [ ] Write", "- [x] Write");
    let done = ok(
        &root,
        &["create", "Shipped", "-s", "done", "--body", &checked],
    );
    assert_eq!(done["checklist"], json!({"checked": 2, "total": 2}));
    assert!(done["closed_at"].is_string());
    let id = done["id"].as_str().unwrap();
    assert_eq!(done["path"], reported(&root, &archived(id)));
    let cancelled = ok(
        &root,
        &["create", "Dropped", "-s", "cancelled", "--body", A_BODY],
    );
    assert_eq!(cancelled["status"], "cancelled");
    let id = cancelled["id"].as_str().unwrap();
    assert_eq!(cancelled["path"], reported(&root, &archived(id)));
    let started = ok(
        &root,
        &["create", "Started", "-s", "in-progress", "--body", A_BODY],
    );
    assert_eq!(started["checklist"], json!({"checked": 1, "total": 2}));

    // To record such an issue, create it open and force the close.
    let created = ok(&root, &["create", "Ship it", "--body", A_BODY]);
    let id = created["id"].as_str().unwrap();
    let closed = ok(&root, &["update", id, "-s", "done", "--force"]);
    assert_eq!(closed["path"], reported(&root, &archived(id)));
    assert_eq!(closed["checklist"], json!({"checked": 1, "total": 2}));
}

// @mara implements VER-LIFECYCLE
#[test]
fn archived_issues_appear_everywhere_but_the_default_listing() {
    let root = fixture("visible");
    ok(&root, &["update", A, "-s", "cancelled", "--force"]);
    assert!(root.join(archived(A)).is_file());

    // Listings.
    for (args, expected) in [
        (&[][..], &[E, B, C, D][..]),
        (&["--all"], &[A, E, B, C, D]),
        (&["--status", "cancelled"], &[A]),
        (
            &["--status", "cancelled", "--status", "todo"],
            &[A, E, B, C, D],
        ),
        (&["--priority", "high"], &[]),
        (&["--priority", "high", "--all"], &[A]),
        (&["--parent", E], &[]),
        (&["--parent", E, "--all"], &[A]),
        (&["--parent", A], &[D]),
        // A closed blocker no longer blocks, and a closed parent holds its
        // child back.
        (&["--ready"], &[B, C]),
    ] {
        assert_eq!(listed(&root, args), expected, "list {args:?}");
    }
    let listing = ok(&root, &["list", "--all"]);
    let a = in_listing(&listing, A);
    assert_eq!(a["status"], "cancelled");
    assert_eq!(a["path"], reported(&root, &archived(A)));

    // Search, in the title and in the body.
    for (args, expected) in [
        (&["--search", "quasar"][..], &[][..]),
        (&["--search", "QUASAR", "--all"], &[A]),
        (&["--search", "quasar", "--status", "cancelled"], &[A]),
        (&["--search", "parse", "--all"], &[A, D]),
        (&["--search", "parse"], &[D]),
    ] {
        assert_eq!(listed(&root, args), expected, "list {args:?}");
    }

    // Show, alone and among open issues in the order requested.
    let a = shown(&root, A);
    assert_eq!(a["status"], "cancelled");
    assert!(a["closed_at"].is_string());
    assert_eq!(a["path"], reported(&root, &archived(A)));
    assert_eq!(a["body"], A_BODY);
    assert_eq!(a["parent"], E);
    assert_eq!(a["related"], json!([C]));
    assert_eq!(a["children"], json!([D]));
    assert_eq!(a["blocks"], json!([B]));
    assert_eq!(
        ids(&ok(&root, &["show", D, &A.to_uppercase(), E])),
        [D, A, E]
    );
    let output = zban(&root, &["show", A]);
    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("status: cancelled\n"), "{text}");
    assert!(
        text.contains(&format!("path: {}\n", reported(&root, &archived(A)))),
        "{text}"
    );

    // Relations, read from the open issues on the other end.
    assert_eq!(shown(&root, E)["children"], json!([A]));
    let b = shown(&root, B);
    assert_eq!(b["blocked_by"], json!([A]));
    assert_eq!(b["ready"], true);
    assert_eq!(shown(&root, C)["related_from"], json!([A]));
    let d = shown(&root, D);
    assert_eq!(d["parent"], A);
    assert_eq!(d["not_ready"], json!([format!("ancestor_closed:{A}")]));

    // Relations to an archived issue can be added and removed, and an edit
    // stored on the archived issue leaves it in the archive.
    let created = ok(
        &root,
        &[
            "create",
            "Follow-up",
            "--discovered-from",
            A,
            "--blocked-by",
            A,
        ],
    );
    let follow_up = created["id"].as_str().unwrap();
    assert_eq!(created["ready"], true);
    let a = shown(&root, A);
    assert_eq!(a["discovered"], json!([follow_up]));
    assert_eq!(a["blocks"], json!([B, follow_up]));
    ok(&root, &["update", C, "--remove-related", A]);
    assert_eq!(shown(&root, A)["related"], json!([]));
    assert!(root.join(archived(A)).is_file());
    assert!(!root.join(open(A)).exists());
    assert_eq!(fails(&root, &["delete", A])["code"], "referenced");
}

// @mara implements VER-LIFECYCLE
#[test]
fn listings_report_checklist_progress() {
    let root = fixture("progress");
    let progress = |args: &[&str], id: &str| {
        in_listing(&ok(&root, &[&["list"], args].concat()), id)["checklist"].clone()
    };
    assert_eq!(progress(&[], A), json!({"checked": 1, "total": 2}));
    assert_eq!(progress(&[], E), json!({"checked": 0, "total": 0}));

    let line = |args: &[&str], id: &str| {
        let output = zban(&root, &[&["list"], args].concat());
        assert!(output.status.success());
        stdout(&output)
            .lines()
            .find(|line| line.starts_with(id))
            .unwrap_or_else(|| panic!("{id} is listed"))
            .to_owned()
    };
    assert!(line(&[], A).ends_with("  [1/2]"), "{}", line(&[], A));
    assert!(!line(&[], E).contains('['), "{}", line(&[], E));

    // Progress follows the body, also for archived issues.
    ok(
        &root,
        &[
            "update",
            A,
            "--replace-old",
            "- [ ] Write",
            "--replace-new",
            "- [x] Write",
        ],
    );
    assert_eq!(progress(&[], A), json!({"checked": 2, "total": 2}));
    ok(
        &root,
        &[
            "update",
            A,
            "--append",
            "- [ ] Review",
            "-s",
            "cancelled",
            "--force",
        ],
    );
    assert_eq!(progress(&["--all"], A), json!({"checked": 2, "total": 3}));
    assert!(
        line(&["--all"], A).ends_with("  [2/3]"),
        "{}",
        line(&["--all"], A)
    );
    assert_eq!(
        shown(&root, A)["checklist"],
        json!({"checked": 2, "total": 3})
    );
}
