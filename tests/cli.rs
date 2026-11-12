//! `VER-CLI`: the command-line contract, through the real `zban` binary.

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use common::{
    snapshot, stderr, stdout, temp_dir, temp_dir_outside_projects, write_files, zban,
    zban_with_input,
};
use serde_json::Value;

const CONFIG: &str = "format_version = 1\nprefix = \"zb\"\n";

/// The keys of an issue in mutation results.
const ISSUE_KEYS: &[&str] = &[
    "id",
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
    "path",
    "checklist",
    "ready",
    "not_ready",
];

/// The key `list` adds.
const LIST_KEYS: &[&str] = &["handoff_at"];

/// The keys `show` adds.
const DETAIL_KEYS: &[&str] = &[
    "body",
    "handoff",
    "children",
    "blocks",
    "related_from",
    "discovered",
];

// The fixture of `fixture`: one letter repeated, named after the issue.
const E: &str = "zb-eeeeee"; // epic, the parent of A, B and F
const A: &str = "zb-aaaaaa"; // high task, ready, tagged storage
const B: &str = "zb-bbbbbb"; // critical bug in progress, tagged storage and cli
const C: &str = "zb-cccccc"; // low draft feature
const D: &str = "zb-dddddd"; // child of A, blocked by B
const F: &str = "zb-ffffff"; // done child of E, tagged cli
const G: &str = "zb-gggggg"; // cancelled high task

/// A hand-written issue file created `second` seconds into 2026.
fn issue(title: &str, status: &str, kind: &str, header: &str, second: u32, body: &str) -> String {
    let closed = if matches!(status, "done" | "cancelled") {
        "closed_at: 2026-02-01T00:00:00Z\n"
    } else {
        ""
    };
    let body = if body.is_empty() {
        String::new()
    } else {
        format!("\n{body}")
    };
    format!(
        "---\ntitle: {title}\nstatus: {status}\ntype: {kind}\n{header}created_at: 2026-01-01T00:00:{second:02}Z\n{closed}---\n{body}"
    )
}

/// A project holding `files`, given by path relative to the project root.
fn project(name: &str, files: &[(String, String)]) -> PathBuf {
    let root = temp_dir(&format!("cli-{name}"));
    write_files(&root, &[(".zban/config.toml", CONFIG)]);
    for (path, content) in files {
        write_files(&root, &[(path.as_str(), content.as_str())]);
    }
    root
}

fn open(id: &str) -> String {
    format!(".zban/{id}.md")
}

fn archived(id: &str) -> String {
    format!(".zban/archive/{id}.md")
}

fn fixture(name: &str) -> PathBuf {
    project(
        name,
        &[
            (open(E), issue("Storage epic", "todo", "epic", "", 1, "")),
            (
                open(A),
                issue(
                    "Parse files",
                    "todo",
                    "task",
                    &format!("priority: high\ntags:\n  - storage\nparent: {E}\n"),
                    2,
                    "Read the FRONTMATTER.\n\n- [x] one\n- [ ] two\n",
                ),
            ),
            (
                open(B),
                issue(
                    "Crash on empty title",
                    "in-progress",
                    "bug",
                    &format!("priority: critical\ntags:\n  - storage\n  - cli\nparent: {E}\n"),
                    3,
                    "",
                ),
            ),
            (
                open(C),
                issue("Search Me", "draft", "feature", "priority: low\n", 4, ""),
            ),
            (
                open(D),
                issue(
                    "Write files",
                    "todo",
                    "task",
                    &format!("parent: {A}\nblocked_by:\n  - {B}\n"),
                    5,
                    "",
                ),
            ),
            (
                archived(F),
                issue(
                    "Old task",
                    "done",
                    "task",
                    &format!("tags:\n  - cli\nparent: {E}\n"),
                    6,
                    "",
                ),
            ),
            (
                archived(G),
                issue(
                    "Dropped idea",
                    "cancelled",
                    "task",
                    "priority: high\n",
                    7,
                    "",
                ),
            ),
        ],
    )
}

/// Runs zban with `--json` and returns the exit status and the one JSON
/// document it printed.
fn run_json(dir: &Path, args: &[&str]) -> (i32, Value) {
    run_json_with(dir, args, None)
}

fn run_json_with(dir: &Path, args: &[&str], input: Option<&str>) -> (i32, Value) {
    let args = [&["--json"], args].concat();
    let output = match input {
        Some(input) => zban_with_input(dir, &args, input),
        None => zban(dir, &args),
    };
    document(&args, &output)
}

fn document(args: &[&str], output: &Output) -> (i32, Value) {
    let value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!("{args:?}: {error}: {:?} {}", stdout(output), stderr(output))
    });
    (output.status.code().unwrap(), value)
}

/// Runs a command that must succeed and returns its JSON document.
fn ok(dir: &Path, args: &[&str]) -> Value {
    let (status, value) = run_json(dir, args);
    assert_eq!(status, 0, "{args:?}: {value}");
    value
}

/// Runs a command that must fail and returns its error code, after checking
/// the error object and the exit status.
fn error_code(dir: &Path, args: &[&str]) -> String {
    let (status, value) = run_json(dir, args);
    error_of(args, status, &value)
}

fn error_of(args: &[&str], status: i32, value: &Value) -> String {
    assert_eq!(keys(value), ["error"], "{args:?}: {value}");
    let error = &value["error"];
    assert_eq!(keys(error), ["code", "message"], "{args:?}: {value}");
    assert!(!error["message"].as_str().unwrap().is_empty());
    let code = error["code"].as_str().unwrap().to_owned();
    let expected = if code == "usage" { 2 } else { 1 };
    assert_eq!(status, expected, "{args:?}: {value}");
    code
}

/// The keys of a JSON object, sorted.
fn keys(value: &Value) -> Vec<&str> {
    let mut keys: Vec<_> = value
        .as_object()
        .unwrap_or_else(|| panic!("an object: {value}"))
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    keys
}

fn sorted(keys: &[&[&str]]) -> Vec<String> {
    let mut keys: Vec<_> = keys.concat().into_iter().map(String::from).collect();
    keys.sort();
    keys
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

fn read(root: &Path, path: &str) -> String {
    fs::read_to_string(root.join(path)).unwrap()
}

/// The body of an issue file: everything after the header and the blank line.
fn body(root: &Path, path: &str) -> String {
    let text = read(root, path);
    let header_end = text[4..].find("\n---\n").unwrap() + 4 + 5;
    text[header_end..]
        .strip_prefix('\n')
        .unwrap_or(&text[header_end..])
        .to_owned()
}

#[test]
fn version_prints_the_package_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_zban"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        stdout(&output),
        format!("zban {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn no_arguments_prints_help_as_invalid_usage() {
    let dir = temp_dir("cli-no-arguments");
    let output = zban(&dir, &[]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("Usage: zban"));
}

// @mara implements VER-CLI
#[test]
fn every_command_prints_one_json_document() {
    let root = temp_dir("cli-json-documents");
    let issue_keys = sorted(&[ISSUE_KEYS]);
    let list_keys = sorted(&[ISSUE_KEYS, LIST_KEYS]);
    let detail_keys = sorted(&[ISSUE_KEYS, DETAIL_KEYS]);

    let init = ok(&root, &["init"]);
    assert_eq!(keys(&init), ["path", "prefix"]);
    assert_eq!(init["prefix"], "zb");
    assert_eq!(
        init["path"].as_str().unwrap(),
        fs::canonicalize(&root)
            .unwrap()
            .join(".zban")
            .to_str()
            .unwrap()
    );

    let parent = ok(&root, &["create", "Parent", "-t", "epic"]);
    assert_eq!(keys(&parent), issue_keys);
    let parent_id = parent["id"].as_str().unwrap();
    let created = ok(
        &root,
        &[
            "create",
            "Child",
            "-t",
            "feature",
            "-p",
            "high",
            "--tag",
            "storage",
            "--parent",
            parent_id,
            "--body",
            "- [x] one\n- [ ] two",
        ],
    );
    assert_eq!(keys(&created), issue_keys);
    let id = created["id"].as_str().unwrap();
    assert!(id.starts_with("zb-") && id.len() == 9, "{id}");
    assert_eq!(created["title"], "Child");
    assert_eq!(created["status"], "todo");
    assert_eq!(created["type"], "feature");
    assert_eq!(created["priority"], "high");
    assert_eq!(created["tags"], serde_json::json!(["storage"]));
    assert_eq!(created["parent"], parent_id);
    assert_eq!(created["blocked_by"], serde_json::json!([]));
    assert_eq!(created["discovered_from"], Value::Null);
    assert_eq!(created["closed_at"], Value::Null);
    assert_eq!(
        created["checklist"],
        serde_json::json!({"checked": 1, "total": 2})
    );
    assert_eq!(created["ready"], true);
    assert_eq!(created["not_ready"], serde_json::json!([]));
    let path = PathBuf::from(created["path"].as_str().unwrap());
    assert!(path.is_absolute() && path.is_file(), "{path:?}");
    assert_eq!(
        path,
        fs::canonicalize(&root)
            .unwrap()
            .join(format!(".zban/{id}.md"))
    );

    let listing = ok(&root, &["list"]);
    assert_eq!(keys(&listing), ["issues", "warnings"]);
    assert_eq!(listing["warnings"], serde_json::json!([]));
    for issue in listing["issues"].as_array().unwrap() {
        assert_eq!(keys(issue), list_keys);
        assert_eq!(issue["handoff_at"], Value::Null);
    }
    assert_eq!(
        listing["issues"][1]["not_ready"],
        serde_json::json!(["epic"])
    );
    assert_eq!(listing["issues"][1]["ready"], false);

    // `--json` also works after the command.
    let output = zban(&root, &["show", id, parent_id, "--json"]);
    let (status, shown) = document(&["show"], &output);
    assert_eq!(status, 0);
    assert_eq!(keys(&shown), ["issues", "warnings"]);
    assert_eq!(ids(&shown), [id, parent_id]);
    for issue in shown["issues"].as_array().unwrap() {
        assert_eq!(keys(issue), detail_keys);
        assert_eq!(issue["handoff"], Value::Null);
    }
    assert_eq!(shown["issues"][0]["body"], "- [x] one\n- [ ] two\n");
    assert_eq!(shown["issues"][1]["children"], serde_json::json!([id]));

    let updated = ok(&root, &["update", id, "-s", "in-progress"]);
    assert_eq!(keys(&updated), issue_keys);
    assert_eq!(updated["status"], "in-progress");
    assert_eq!(updated["not_ready"], serde_json::json!(["status"]));

    // Closing the parent leaves the child open under it: a warning, exit 0.
    ok(&root, &["update", parent_id, "-s", "done", "--force"]);
    let checked = ok(&root, &["check"]);
    assert_eq!(keys(&checked), ["findings"]);
    let findings = checked["findings"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{checked}");
    let finding = &findings[0];
    assert_eq!(keys(finding), ["code", "message", "path", "severity"]);
    assert_eq!(finding["severity"], "warning");
    assert_eq!(finding["code"], "open_under_closed");
    assert_eq!(finding["path"], created["path"]);

    let deleted = ok(&root, &["delete", id]);
    assert_eq!(keys(&deleted), issue_keys);
    assert_eq!(deleted["id"], id);
    assert!(!path.exists());
}

// @mara implements VER-CLI
#[test]
fn failures_print_an_error_object_and_exit_with_status_1() {
    let root = fixture("cli-failures");
    let before = snapshot(&root.join(".zban"), &[]);
    assert_eq!(error_code(&root, &["show", "zb-000000"]), "not_found");

    let output = zban(&root, &["show", "zb-000000"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output).starts_with("error: ") && stderr(&output).ends_with("[not_found]\n"),
        "{}",
        stderr(&output)
    );
    assert_eq!(snapshot(&root.join(".zban"), &[]), before);
}

// @mara implements VER-CLI
#[test]
fn invalid_usage_exits_with_status_2() {
    let root = fixture("cli-usage");
    let before = snapshot(&root.join(".zban"), &[]);
    for args in [
        &["--no-such-option"][..],
        &[],
        &["create"],
        &["show"],
        &["update", A, "--parent", E, "--no-parent"],
        &["update", A, "--discovered-from", E, "--no-discovered-from"],
        &["update", A, "--replace-old", "one"],
        &[
            "update",
            A,
            "--replace-old",
            "one",
            "--replace-old",
            "two",
            "--replace-new",
            "x",
        ],
        &["update", A, "--replace-old", "", "--replace-new", "x"],
        &["update", A, "--body", "-", "--append", "-"],
        &["update", A, "--add-tag", "x", "--remove-tag", "x"],
        &["update", A, "--add-blocked-by", B, "--remove-blocked-by", B],
        &[
            "update",
            A,
            "--add-related",
            C,
            "--remove-related",
            "ZB-CCCCCC",
        ],
    ] {
        let (status, value) = run_json_with(&root, args, Some(""));
        assert_eq!(error_of(args, status, &value), "usage", "{args:?}");

        let output = zban_with_input(&root, args, "");
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert_eq!(stdout(&output), "", "{args:?}");
        assert!(!stderr(&output).is_empty(), "{args:?}");
    }
    assert_eq!(snapshot(&root.join(".zban"), &[]), before);
}

/// Every error code of `DES-CLI` except `conflict`, which needs a change
/// between reading and writing and is produced by
/// `tests/issue_files.rs::concurrent_change_fails_with_conflict_and_survives`,
/// and `invalid_handoff`, which comes with the handoff options.
// @mara implements VER-CLI
#[test]
fn each_error_code_is_produced() {
    let root = fixture("cli-error-codes");
    write_files(
        &root,
        &[
            (
                ".zban/zb-hhhhhh.md",
                &issue("Unfinished", "todo", "task", "", 8, "- [ ] open item\n"),
            ),
            (
                ".zban/zb-kkkkkk.md",
                "---\n<<<<<<< HEAD\ntitle: Ours\n=======\ntitle: Theirs\n>>>>>>> branch\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z\n---\n",
            ),
            (
                ".zban/zb-wwwwww.md",
                &issue("Twin", "todo", "task", "", 9, ""),
            ),
            (
                ".zban/archive/zb-wwwwww.md",
                &issue("Twin", "done", "task", "", 9, ""),
            ),
        ],
    );
    let bad_config = project("cli-error-codes-config", &[]);
    write_files(
        &bad_config,
        &[(".zban/config.toml", "format_version = 2\n")],
    );
    let outside = temp_dir_outside_projects("error-codes");
    let before = snapshot(&root.join(".zban"), &[]);

    let cases: &[(&str, &Path, &[&str])] = &[
        (
            "usage",
            &root,
            &["update", A, "--add-tag", "x", "--remove-tag", "x"],
        ),
        ("project_not_found", &outside, &["list"]),
        ("project_exists", &root, &["init"]),
        ("invalid_config", &bad_config, &["list"]),
        ("not_found", &root, &["show", "zb-000000"]),
        ("not_found", &root, &["update", "nonsense", "-s", "todo"]),
        ("ambiguous_id", &root, &["show", "zb-wwwwww"]),
        ("unreadable", &root, &["show", "zb-kkkkkk"]),
        ("invalid_value", &root, &["create", "New", "-s", "blocked"]),
        ("invalid_value", &root, &["update", A, "-t", "story"]),
        (
            "invalid_value",
            &root,
            &["update", A, "--title", "Two\nlines"],
        ),
        (
            "invalid_value",
            &root,
            &["update", A, "--add-tag", "Bad Tag"],
        ),
        ("invalid_value", &root, &["list", "--priority", "urgent"]),
        (
            "invalid_reference",
            &root,
            &["create", "New", "--parent", "zb-000000"],
        ),
        (
            "invalid_reference",
            &root,
            &["update", A, "--add-blocked-by", "nonsense"],
        ),
        (
            "invalid_reference",
            &root,
            &["update", A, "--add-related", A],
        ),
        ("cycle", &root, &["update", E, "--parent", D]),
        ("cycle", &root, &["update", D, "--add-blocked-by", E]),
        (
            "unchecked_items",
            &root,
            &["update", "zb-hhhhhh", "-s", "done"],
        ),
        ("open_children", &root, &["update", E, "-s", "cancelled"]),
        ("referenced", &root, &["delete", B]),
        (
            "text_not_found",
            &root,
            &[
                "update",
                A,
                "--replace-old",
                "nowhere",
                "--replace-new",
                "x",
            ],
        ),
        (
            "text_ambiguous",
            &root,
            &["update", A, "--replace-old", "- [", "--replace-new", "x"],
        ),
        ("io", &root, &["-C", "missing", "list"]),
    ];
    for (expected, dir, args) in cases {
        assert_eq!(error_code(dir, args), *expected, "{args:?}");
    }
    assert_eq!(snapshot(&root.join(".zban"), &[]), before);

    let produced: Vec<_> = cases.iter().map(|(code, _, _)| *code).collect();
    for code in [
        "usage",
        "project_not_found",
        "project_exists",
        "invalid_config",
        "not_found",
        "ambiguous_id",
        "unreadable",
        "invalid_value",
        "invalid_reference",
        "cycle",
        "unchecked_items",
        "open_children",
        "referenced",
        "text_not_found",
        "text_ambiguous",
        "io",
    ] {
        assert!(produced.contains(&code), "{code}");
    }
    fs::remove_dir_all(outside).unwrap();
}

// @mara implements VER-CLI
#[test]
fn list_filters_select_issues_in_listing_order() {
    let root = fixture("cli-list-filters");
    for (args, expected) in [
        (&[][..], &[B, A, E, D, C][..]),
        (&["--all"], &[B, A, G, E, D, F, C]),
        (&["--status", "todo"], &[A, E, D]),
        (&["-s", "todo", "--status", "draft"], &[A, E, D, C]),
        (&["--status", "done"], &[F]),
        (&["--status", "done", "--status", "cancelled"], &[G, F]),
        (&["--type", "task"], &[A, D]),
        (&["--type", "task", "-t", "bug"], &[B, A, D]),
        (&["--type", "task", "--all"], &[A, G, D, F]),
        (&["--priority", "high"], &[A]),
        (&["--priority", "high", "--all"], &[A, G]),
        (&["--priority", "normal", "-p", "low"], &[E, D, C]),
        (&["--tag", "storage"], &[B, A]),
        (&["--tag", "cli"], &[B]),
        (&["--tag", "cli", "--all"], &[B, F]),
        (&["--tag", "storage", "--tag", "cli"], &[B, A]),
        (&["--parent", E], &[B, A]),
        (&["--parent", E, "--all"], &[B, A, F]),
        (&["--parent", "ZB-AAAAAA"], &[D]),
        (&["--parent", "nonsense"], &[]),
        (&["--ready"], &[A]),
        (&["--search", "frontmatter"], &[A]),
        (&["--search", "SEARCH"], &[C]),
        (&["--search", "storage"], &[E]),
        (&["--search", "old"], &[]),
        (&["--search", "old", "--all"], &[F]),
        (&["--type", "task", "--tag", "storage"], &[A]),
        (&["--status", "todo", "--priority", "normal"], &[E, D]),
        (&["--parent", E, "--type", "bug"], &[B]),
        (&["--ready", "--tag", "cli"], &[]),
        (&["--ready", "--all", "--search", "files"], &[A]),
    ] {
        assert_eq!(listed(&root, args), expected, "{args:?}");
    }

    let listing = ok(&root, &["list", "--all"]);
    let reasons = |id: &str| {
        let issue = listing["issues"]
            .as_array()
            .unwrap()
            .iter()
            .find(|issue| issue["id"] == id)
            .unwrap();
        issue["not_ready"].clone()
    };
    assert_eq!(reasons(A), serde_json::json!([]));
    assert_eq!(reasons(D), serde_json::json!([format!("blocked_by:{B}")]));
    assert_eq!(reasons(C), serde_json::json!(["status"]));
    assert_eq!(reasons(E), serde_json::json!(["epic"]));
}

// @mara implements VER-CLI
#[test]
fn commands_outside_a_project_fail_with_project_not_found() {
    let dir = temp_dir_outside_projects("cli-outside");
    for args in [
        &["list"][..],
        &["show", A],
        &["create", "New"],
        &["update", A, "-s", "done"],
        &["delete", A],
    ] {
        assert_eq!(error_code(&dir, args), "project_not_found", "{args:?}");
        let output = zban(&dir, args);
        assert!(stderr(&output).contains("[project_not_found]"), "{args:?}");
    }
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
    fs::remove_dir_all(dir).unwrap();
}

// @mara implements VER-CLI
#[test]
fn directory_option_behaves_like_starting_there() {
    let root = fixture("cli-directory");
    fs::create_dir_all(root.join("src/deep")).unwrap();
    let outside = temp_dir_outside_projects("cli-directory");
    let expected = ok(&root, &["list", "--all"]);
    for dir in [root.clone(), root.join("src/deep")] {
        let dir = dir.to_str().unwrap();
        assert_eq!(ok(&outside, &["-C", dir, "list", "--all"]), expected);
        assert_eq!(
            ok(&outside, &["list", "--all", "--directory", dir]),
            expected
        );
        assert_eq!(ok(&root.join("src"), &["list", "--all"]), expected);
    }
    let deep = root.join("src/deep");
    let created = ok(
        &outside,
        &["-C", deep.to_str().unwrap(), "create", "From deep"],
    );
    let id = created["id"].as_str().unwrap();
    assert!(root.join(open(id)).is_file());
    assert!(!deep.join(".zban").exists());
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
    fs::remove_dir_all(outside).unwrap();
}

// @mara implements VER-CLI
#[test]
fn failed_replacements_write_nothing() {
    let root = fixture("cli-replacements");
    let before = read(&root, &open(A));
    for (old, code) in [("nowhere", "text_not_found"), ("- [", "text_ambiguous")] {
        let args = [
            "update",
            A,
            "--replace-old",
            "- [ ] two",
            "--replace-new",
            "- [x] two",
            "--replace-old",
            old,
            "--replace-new",
            "x",
        ];
        assert_eq!(error_code(&root, &args), code);
        assert_eq!(read(&root, &open(A)), before);
    }

    // Replacements apply in order, each to the result of the one before.
    let updated = ok(
        &root,
        &[
            "update",
            A,
            "--replace-old",
            "- [ ] two",
            "--replace-new",
            "- [x] two",
            "--replace-old",
            "- [x] one\n- [x] two",
            "--replace-new",
            "- [x] both",
        ],
    );
    assert_eq!(
        updated["checklist"],
        serde_json::json!({"checked": 1, "total": 1})
    );
    assert_eq!(
        body(&root, &open(A)),
        "Read the FRONTMATTER.\n\n- [x] both\n"
    );
}

// @mara implements VER-CLI
#[test]
fn append_joins_after_exactly_one_blank_line() {
    let root = project(
        "cli-append",
        &[
            (
                open(A),
                issue("Trailing", "todo", "task", "", 1, "Intro\n\n\n"),
            ),
            (open(B), issue("No newline", "todo", "task", "", 2, "Intro")),
            (open(C), issue("Empty", "todo", "task", "", 3, "")),
        ],
    );
    for id in [A, B] {
        ok(&root, &["update", id, "--append", "## Notes\n\nMore."]);
        assert_eq!(
            body(&root, &open(id)),
            "Intro\n\n## Notes\n\nMore.\n",
            "{id}"
        );
    }
    ok(&root, &["update", C, "--append", "First\n\n"]);
    assert_eq!(body(&root, &open(C)), "First\n");
    ok(&root, &["update", C, "--append", "Second"]);
    assert_eq!(body(&root, &open(C)), "First\n\nSecond\n");
}

// @mara implements VER-CLI
#[test]
fn an_update_is_all_or_nothing() {
    let root = fixture("cli-all-or-nothing");
    // A's only child moves away, so that A can close.
    ok(&root, &["update", D, "--no-parent"]);
    let path = open(A);
    let before = read(&root, &path);

    // A failing replacement after a good one, with an append and metadata.
    let args = [
        "update",
        A,
        "--title",
        "Renamed",
        "-p",
        "low",
        "--add-tag",
        "parser",
        "--replace-old",
        "- [ ] two",
        "--replace-new",
        "- [x] two",
        "--replace-old",
        "absent",
        "--replace-new",
        "x",
        "--append",
        "## Summary of Changes",
        "-s",
        "in-progress",
    ];
    assert_eq!(error_code(&root, &args), "text_not_found");
    assert_eq!(read(&root, &path), before);

    // A close refused because the request leaves an unchecked item.
    let args = [
        "update",
        A,
        "--title",
        "Renamed",
        "--append",
        "## Summary of Changes\n\n- [ ] left over",
        "-s",
        "done",
    ];
    assert_eq!(error_code(&root, &args), "unchecked_items");
    assert_eq!(read(&root, &path), before);

    // One update checks the last item, appends the summary and closes.
    let updated = ok(
        &root,
        &[
            "update",
            A,
            "--title",
            "Renamed",
            "--add-tag",
            "parser",
            "--replace-old",
            "- [ ] two",
            "--replace-new",
            "- [x] two",
            "--append",
            "## Summary of Changes\n\nDone.",
            "-s",
            "done",
        ],
    );
    assert_eq!(updated["status"], "done");
    assert_eq!(updated["title"], "Renamed");
    assert_eq!(updated["tags"], serde_json::json!(["storage", "parser"]));
    assert!(updated["closed_at"].is_string());
    assert!(!root.join(&path).exists());
    assert_eq!(
        body(&root, &archived(A)),
        "Read the FRONTMATTER.\n\n- [x] one\n- [x] two\n\n## Summary of Changes\n\nDone.\n"
    );
    assert!(
        updated["path"]
            .as_str()
            .unwrap()
            .ends_with(&format!(".zban/archive/{A}.md"))
    );
}

// @mara implements VER-CLI
#[test]
fn body_text_can_come_from_standard_input() {
    let root = fixture("cli-stdin");
    let (status, created) = run_json_with(
        &root,
        &["create", "From stdin", "--body", "-"],
        Some("- [ ] read\n- [ ] write\n\n\n"),
    );
    assert_eq!(status, 0, "{created}");
    let id = created["id"].as_str().unwrap();
    assert_eq!(body(&root, &open(id)), "- [ ] read\n- [ ] write\n");

    let (status, _) = run_json_with(
        &root,
        &["update", id, "--append", "-"],
        Some("## Notes\n\n- multi\n- line\n"),
    );
    assert_eq!(status, 0);
    assert_eq!(
        body(&root, &open(id)),
        "- [ ] read\n- [ ] write\n\n## Notes\n\n- multi\n- line\n"
    );

    let (status, _) = run_json_with(&root, &["update", id, "--body", "-"], Some("Replaced"));
    assert_eq!(status, 0);
    assert_eq!(body(&root, &open(id)), "Replaced\n");

    // A body that is only `-` text from an option, not standard input.
    ok(&root, &["update", id, "--body", "- item"]);
    assert_eq!(body(&root, &open(id)), "- item\n");
}

// @mara implements VER-CLI
#[test]
fn relations_change_through_update() {
    let root = fixture("cli-relations");
    let updated = ok(
        &root,
        &[
            "update",
            D,
            "--no-parent",
            "--remove-blocked-by",
            B,
            "--add-blocked-by",
            "ZB-CCCCCC",
            "--add-related",
            A,
            "--discovered-from",
            E,
        ],
    );
    assert_eq!(updated["parent"], Value::Null);
    assert_eq!(updated["blocked_by"], serde_json::json!([C]));
    assert_eq!(updated["related"], serde_json::json!([A]));
    assert_eq!(updated["discovered_from"], E);
    assert_eq!(
        updated["not_ready"],
        serde_json::json!([format!("blocked_by:{C}")])
    );

    let shown = ok(&root, &["show", A, C, E]);
    assert_eq!(shown["issues"][0]["related_from"], serde_json::json!([D]));
    assert_eq!(shown["issues"][0]["children"], serde_json::json!([]));
    assert_eq!(shown["issues"][1]["blocks"], serde_json::json!([D]));
    assert_eq!(shown["issues"][2]["discovered"], serde_json::json!([D]));

    // Removing the link from the other side edits the issue that stores it.
    let before = read(&root, &open(A));
    ok(&root, &["update", A, "--remove-related", D]);
    assert_eq!(read(&root, &open(A)), before);
    let shown = ok(&root, &["show", D]);
    assert_eq!(shown["issues"][0]["related"], serde_json::json!([]));

    let updated = ok(&root, &["update", D, "--parent", A, "--no-discovered-from"]);
    assert_eq!(updated["parent"], A);
    assert_eq!(updated["discovered_from"], Value::Null);
}

// @mara implements VER-CLI
#[test]
fn an_update_that_changes_nothing_leaves_the_file_alone() {
    let root = project(
        "cli-unchanged",
        &[(
            open(A),
            "---\n# hand-written\ntitle: 'Quoted'\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:01Z\n---\n"
                .to_owned(),
        )],
    );
    let before = read(&root, &open(A));
    ok(&root, &["update", A]);
    ok(
        &root,
        &["update", A, "-s", "todo", "--remove-tag", "absent"],
    );
    assert_eq!(read(&root, &open(A)), before);
}

// @mara implements VER-CLI
#[test]
fn create_reopen_and_delete_place_files_by_status() {
    let root = fixture("cli-create-closed");
    let created = ok(&root, &["create", "Already done", "-s", "done"]);
    let id = created["id"].as_str().unwrap();
    assert!(created["closed_at"].is_string());
    assert!(root.join(archived(id)).is_file());

    let reopened = ok(&root, &["update", id, "-s", "todo"]);
    assert_eq!(reopened["closed_at"], Value::Null);
    assert!(root.join(open(id)).is_file());
    assert!(!root.join(archived(id)).exists());

    let deleted = ok(&root, &["delete", &id.to_uppercase()]);
    assert_eq!(deleted["title"], "Already done");
    assert!(!root.join(open(id)).exists());
    assert_eq!(error_code(&root, &["show", id]), "not_found");
}

// @mara implements VER-CLI
#[test]
fn broken_files_do_not_stop_work_on_other_issues() {
    let root = project(
        "cli-damaged",
        &[
            (
                open("zb-cccccc"),
                "---\n<<<<<<< HEAD\ntitle: Ours\n=======\ntitle: Theirs\n>>>>>>> branch\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z\n---\n"
                    .to_owned(),
            ),
            (open("zb-vvvvvv"), issue("Invalid", "blocked", "task", "", 1, "")),
            (
                open(D),
                issue("Dangling", "todo", "task", "blocked_by:\n  - zb-000000\n", 2, ""),
            ),
            (open("zb-wwwwww"), issue("Twin", "todo", "task", "", 3, "")),
            (archived("zb-wwwwww"), issue("Twin", "done", "task", "", 3, "")),
            (open(G), issue("Good", "todo", "task", "", 4, "")),
        ],
    );
    let canonical = fs::canonicalize(&root).unwrap();
    let expected_warnings = [
        ("duplicate_id", archived("zb-wwwwww")),
        ("unparseable", open("zb-cccccc")),
        ("invalid_value", open("zb-vvvvvv")),
        ("duplicate_id", open("zb-wwwwww")),
    ];
    let warnings = |value: &Value| -> Vec<(String, String)> {
        value["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|warning| {
                assert_eq!(keys(warning), ["code", "message", "path"]);
                let path = Path::new(warning["path"].as_str().unwrap());
                (
                    warning["code"].as_str().unwrap().to_owned(),
                    path.strip_prefix(&canonical)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned(),
                )
            })
            .collect()
    };
    let expected: Vec<_> = expected_warnings
        .iter()
        .map(|(code, path)| ((*code).to_owned(), path.clone()))
        .collect();

    let listing = ok(&root, &["list"]);
    assert_eq!(ids(&listing), [D, G]);
    assert_eq!(warnings(&listing), expected);
    assert_eq!(
        listing["issues"][0]["not_ready"],
        serde_json::json!(["blocked_by:zb-000000"])
    );
    let shown = ok(&root, &["show", G]);
    assert_eq!(ids(&shown), [G]);
    assert_eq!(warnings(&shown), expected);

    let output = zban(&root, &["list"]);
    assert!(output.status.success());
    let lines: Vec<_> = stderr(&output).lines().map(String::from).collect();
    assert_eq!(lines.len(), 4, "{lines:?}");
    assert!(lines[1].starts_with("warning: skipped ") && lines[1].ends_with("[unparseable]"));

    let updated = ok(&root, &["update", D, "-s", "in-progress"]);
    assert_eq!(updated["status"], "in-progress");
    assert_eq!(updated["blocked_by"], serde_json::json!(["zb-000000"]));

    let before = snapshot(&root.join(".zban"), &[]);
    for id in ["zb-cccccc", "zb-vvvvvv"] {
        assert_eq!(error_code(&root, &["show", id]), "unreadable");
        assert_eq!(
            error_code(&root, &["update", id, "-s", "in-progress"]),
            "unreadable"
        );
    }
    assert_eq!(error_code(&root, &["show", "zb-wwwwww"]), "ambiguous_id");
    assert_eq!(
        error_code(&root, &["update", "zb-wwwwww", "-s", "in-progress"]),
        "ambiguous_id"
    );
    assert_eq!(
        error_code(&root, &["update", G, "--parent", "zb-cccccc"]),
        "unreadable"
    );
    assert_eq!(snapshot(&root.join(".zban"), &[]), before);
}

#[test]
fn human_output_is_text() {
    let root = fixture("cli-human");
    let output = zban(&root, &["list"]);
    assert!(output.status.success());
    let text = stdout(&output);
    let first = text.lines().next().unwrap();
    assert!(
        first.starts_with(B)
            && first.contains("in-progress")
            && first.contains("Crash on empty title")
    );
    assert!(
        text.contains(&format!("{A}  todo")) && text.contains("[1/2]"),
        "{text}"
    );
    assert_eq!(stderr(&output), "");

    let output = zban(&root, &["list", "--tag", "nothing"]);
    assert_eq!(stdout(&output), "No issues.\n");

    let output = zban(&root, &["show", A, E]);
    let text = stdout(&output);
    assert!(text.starts_with(&format!("{A}: Parse files\n")), "{text}");
    assert!(text.contains("checklist: 1/2 checked\n") && text.contains("ready: yes\n"));
    assert!(text.contains("\nRead the FRONTMATTER.\n"));
    assert!(text.contains(&format!("\n\n{E}: Storage epic\n")));
    assert!(text.contains("ready: no (epic)\n"));

    let output = zban(&root, &["create", "Human"]);
    assert!(stdout(&output).starts_with("Created zb-"));
    assert!(stdout(&output).ends_with(": Human\n"));
}
