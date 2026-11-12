//! `VER-CHECK`: reporting and repairing inconsistencies through the real
//! `zban` binary, with one broken project for each `DES-CHECK` code.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use common::{
    copy_repository_backlog, snapshot, stderr, stdout, temp_dir, temp_dir_outside_projects,
    write_files, zban,
};
use serde_json::{Value, json};
use zban::{Project, Severity};

const CONFIG: &str = "format_version = 1\nprefix = \"zb\"\n";

/// The codes of the `DES-CHECK` table.
const DES_CHECK_CODES: [&str; 12] = [
    "invalid_config",
    "invalid_filename",
    "unparseable",
    "missing_field",
    "invalid_value",
    "unknown_key",
    "duplicate_id",
    "invalid_reference",
    "cycle",
    "related_both_sides",
    "open_under_closed",
    "misplaced",
];

/// A hand-written issue with `status`, extra `header` lines and no body. A
/// closed status gets `closed_at`.
fn issue(status: &str, header: &str) -> String {
    let closed = if matches!(status, "done" | "cancelled") {
        "closed_at: 2026-11-12T08:00:00Z\n"
    } else {
        ""
    };
    format!(
        "---\ntitle: Issue\nstatus: {status}\ntype: task\n{header}created_at: 2026-11-11T20:00:00Z\n{closed}---\n"
    )
}

/// A header that Git left with conflict markers after a merge.
const CONFLICTED: &str = "---\n<<<<<<< HEAD\ntitle: Ours\n=======\ntitle: Theirs\n>>>>>>> topic\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z\n---\n";

/// A project with the default configuration, then `files` written over it.
fn project(name: &str, files: &[(&str, String)]) -> PathBuf {
    let root = temp_dir(&format!("check-{name}"));
    write_files(&root, &[(".zban/config.toml", CONFIG)]);
    for (path, content) in files {
        write_files(&root, &[(path, content)]);
    }
    root
}

/// A finding as `zban check` prints it, without the message:
/// `(path, severity, code)`.
type Reported = (String, String, String);

/// Runs `zban check` with `args` in `root` and returns its exit status and
/// findings. Each line is `<path>: <severity>: <message> [<code>]`.
fn check(root: &Path, args: &[&str]) -> (i32, Vec<Reported>) {
    let output = zban(root, &[&["check"], args].concat());
    assert_eq!(stderr(&output), "", "check {args:?} in {}", root.display());
    let findings = stdout(&output)
        .lines()
        .map(|line| {
            let (path, rest) = line.split_once(": ").expect("a path");
            let (severity, rest) = rest.split_once(": ").expect("a severity");
            let code = rest
                .rsplit_once(" [")
                .and_then(|(_, code)| code.strip_suffix(']'))
                .expect("a code");
            (path.to_owned(), severity.to_owned(), code.to_owned())
        })
        .collect();
    (output.status.code().expect("an exit status"), findings)
}

fn reported(expected: &[(&str, &str, &str)]) -> Vec<Reported> {
    expected
        .iter()
        .map(|&(path, severity, code)| (path.to_owned(), severity.to_owned(), code.to_owned()))
        .collect()
}

/// Runs `zban check --json` with `args` in `root` and returns its exit
/// status and the one JSON document it prints.
fn check_json(root: &Path, args: &[&str]) -> (i32, Value) {
    let output = zban(root, &[&["check", "--json"], args].concat());
    assert_eq!(stderr(&output), "", "check --json {args:?}");
    let document = serde_json::from_slice(&output.stdout).expect("one JSON document");
    (output.status.code().expect("an exit status"), document)
}

/// Every file and directory under `root` with its bytes, by relative path.
fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    snapshot(root, &[]).into_iter().collect()
}

/// Moves the entry `from` to `to` in a [`files`] snapshot.
fn moved(
    mut files: BTreeMap<PathBuf, Vec<u8>>,
    from: &str,
    to: &str,
) -> BTreeMap<PathBuf, Vec<u8>> {
    let bytes = files.remove(Path::new(from)).expect("the file was there");
    files.insert(PathBuf::from(to), bytes);
    files
}

/// One broken project and the findings `zban check` must report for it, in
/// order: `(path, severity, code)`.
struct Case {
    name: &'static str,
    files: Vec<(&'static str, String)>,
    findings: Vec<(&'static str, &'static str, &'static str)>,
}

fn corpus() -> Vec<Case> {
    let broken_issues = || {
        vec![
            (".zban/notes.md", "# Notes\n".to_owned()),
            (".zban/zb-cccccc.md", CONFLICTED.to_owned()),
            (".zban/zb-dddddd.md", issue("done", "")),
        ]
    };
    let invalid_config = |name, config: &str| {
        let mut files = broken_issues();
        files.push((".zban/config.toml", config.to_owned()));
        Case {
            name,
            files,
            findings: vec![(".zban/config.toml", "error", "invalid_config")],
        }
    };
    vec![
        invalid_config(
            "config-unknown-key",
            "format_version = 1\nprefix = \"zb\"\ncolor = \"red\"\n",
        ),
        invalid_config(
            "config-unknown-format",
            "format_version = 2\nprefix = \"zb\"\n",
        ),
        invalid_config("config-invalid-prefix", "format_version = 1\nprefix = \"Zb\"\n"),
        invalid_config("config-unparseable", "format_version = 1\nprefix = zb\n"),
        Case {
            name: "invalid-filename",
            files: vec![
                (".zban/notes.md", issue("todo", "")),
                (".zban/zb-aaaaa.md", issue("todo", "")),
                (".zban/web-aaaaaa.md", issue("todo", "")),
                (".zban/archive/ZB-BBBBBB.md", issue("done", "")),
                (".zban/zb-cccccc.md", issue("todo", "")),
            ],
            findings: vec![
                (".zban/archive/ZB-BBBBBB.md", "error", "invalid_filename"),
                (".zban/notes.md", "error", "invalid_filename"),
                (".zban/web-aaaaaa.md", "error", "invalid_filename"),
                (".zban/zb-aaaaa.md", "error", "invalid_filename"),
            ],
        },
        Case {
            name: "unparseable",
            files: vec![
                (".zban/zb-aaaaaa.md", "# No frontmatter\n".to_owned()),
                (".zban/zb-bbbbbb.md", CONFLICTED.to_owned()),
                (
                    ".zban/zb-cccccc.md",
                    "---\ntitle: &name Anchored\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z\n---\n".to_owned(),
                ),
                (
                    ".zban/zb-dddddd.md",
                    "---\ntitle: [unclosed\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z\n---\n".to_owned(),
                ),
                (".zban/zb-eeeeee.md", "---\ntitle: No closing line\n".to_owned()),
            ],
            findings: vec![
                (".zban/zb-aaaaaa.md", "error", "unparseable"),
                (".zban/zb-bbbbbb.md", "error", "unparseable"),
                (".zban/zb-cccccc.md", "error", "unparseable"),
                (".zban/zb-dddddd.md", "error", "unparseable"),
                (".zban/zb-eeeeee.md", "error", "unparseable"),
            ],
        },
        Case {
            name: "missing-field",
            files: vec![
                (
                    ".zban/zb-aaaaaa.md",
                    "---\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z\n---\n"
                        .to_owned(),
                ),
                (
                    ".zban/archive/zb-bbbbbb.md",
                    "---\ntitle: Closed without closed_at\nstatus: done\ntype: task\ncreated_at: 2026-11-11T20:00:00Z\n---\n".to_owned(),
                ),
                (
                    ".zban/zb-cccccc.md",
                    "---\ntitle: Undated\nstatus: todo\n---\n".to_owned(),
                ),
            ],
            findings: vec![
                (".zban/archive/zb-bbbbbb.md", "error", "missing_field"),
                (".zban/zb-aaaaaa.md", "error", "missing_field"),
                (".zban/zb-cccccc.md", "error", "missing_field"),
                (".zban/zb-cccccc.md", "error", "missing_field"),
            ],
        },
        Case {
            name: "invalid-value",
            files: vec![
                (".zban/zb-aaaaaa.md", issue("open", "")),
                (
                    ".zban/zb-bbbbbb.md",
                    issue("todo", "closed_at: 2026-11-12T08:00:00Z\n"),
                ),
                (
                    ".zban/zb-cccccc.md",
                    issue("todo", "priority: urgent\ntags:\n  - Storage\n"),
                ),
                (".zban/zb-dddddd.md", issue("todo", "parent: zb-nope\n")),
                (
                    ".zban/zb-eeeeee.md",
                    issue("todo", "").replace("type: task", "type: story"),
                ),
            ],
            findings: vec![
                (".zban/zb-aaaaaa.md", "error", "invalid_value"),
                (".zban/zb-bbbbbb.md", "error", "invalid_value"),
                (".zban/zb-cccccc.md", "error", "invalid_value"),
                (".zban/zb-cccccc.md", "error", "invalid_value"),
                (".zban/zb-dddddd.md", "error", "invalid_value"),
                (".zban/zb-eeeeee.md", "error", "invalid_value"),
            ],
        },
        Case {
            name: "unknown-key",
            files: vec![(
                ".zban/zb-aaaaaa.md",
                issue("todo", "assignee: alice\nestimate: 3\n"),
            )],
            findings: vec![
                (".zban/zb-aaaaaa.md", "warning", "unknown_key"),
                (".zban/zb-aaaaaa.md", "warning", "unknown_key"),
            ],
        },
        Case {
            name: "duplicate-id",
            files: vec![
                (".zban/zb-aaaaaa.md", issue("todo", "")),
                (".zban/archive/zb-aaaaaa.md", issue("done", "")),
            ],
            findings: vec![
                (".zban/archive/zb-aaaaaa.md", "error", "duplicate_id"),
                (".zban/zb-aaaaaa.md", "error", "duplicate_id"),
            ],
        },
        Case {
            name: "invalid-reference",
            files: vec![
                (
                    ".zban/zb-aaaaaa.md",
                    issue(
                        "todo",
                        "blocked_by:\n  - zb-000000\ndiscovered_from: zb-111111\n",
                    ),
                ),
                (".zban/zb-bbbbbb.md", issue("todo", "parent: zb-bbbbbb\n")),
                // A reference to an unreadable file is not reported again.
                (
                    ".zban/zb-cccccc.md",
                    issue("todo", "related:\n  - zb-vvvvvv\n"),
                ),
                (".zban/zb-vvvvvv.md", CONFLICTED.to_owned()),
            ],
            findings: vec![
                (".zban/zb-aaaaaa.md", "error", "invalid_reference"),
                (".zban/zb-aaaaaa.md", "error", "invalid_reference"),
                (".zban/zb-bbbbbb.md", "error", "invalid_reference"),
                (".zban/zb-vvvvvv.md", "error", "unparseable"),
            ],
        },
        Case {
            name: "cycle",
            files: vec![
                // B is blocked by its own child A.
                (".zban/zb-aaaaaa.md", issue("todo", "parent: zb-bbbbbb\n")),
                (
                    ".zban/zb-bbbbbb.md",
                    issue("todo", "blocked_by:\n  - zb-aaaaaa\n"),
                ),
                // C is blocked by its own parent D.
                (
                    ".zban/zb-cccccc.md",
                    issue("todo", "parent: zb-dddddd\nblocked_by:\n  - zb-dddddd\n"),
                ),
                (".zban/zb-dddddd.md", issue("todo", "")),
                // P and Q are each other's parent.
                (".zban/zb-pppppp.md", issue("todo", "parent: zb-qqqqqq\n")),
                (".zban/zb-qqqqqq.md", issue("todo", "parent: zb-pppppp\n")),
                // X and Y block each other.
                (
                    ".zban/zb-xxxxxx.md",
                    issue("todo", "blocked_by:\n  - zb-yyyyyy\n"),
                ),
                (
                    ".zban/zb-yyyyyy.md",
                    issue("todo", "blocked_by:\n  - zb-xxxxxx\n"),
                ),
            ],
            findings: vec![
                (".zban/zb-aaaaaa.md", "error", "cycle"),
                (".zban/zb-bbbbbb.md", "error", "cycle"),
                (".zban/zb-cccccc.md", "error", "cycle"),
                (".zban/zb-pppppp.md", "error", "cycle"),
                (".zban/zb-qqqqqq.md", "error", "cycle"),
                (".zban/zb-xxxxxx.md", "error", "cycle"),
                (".zban/zb-yyyyyy.md", "error", "cycle"),
            ],
        },
        Case {
            name: "related-both-sides",
            files: vec![
                (
                    ".zban/zb-aaaaaa.md",
                    issue("todo", "related:\n  - zb-bbbbbb\n"),
                ),
                (
                    ".zban/zb-bbbbbb.md",
                    issue("todo", "related:\n  - zb-aaaaaa\n"),
                ),
            ],
            findings: vec![(".zban/zb-bbbbbb.md", "warning", "related_both_sides")],
        },
        Case {
            name: "open-under-closed",
            files: vec![
                (".zban/archive/zb-pppppp.md", issue("done", "")),
                (".zban/zb-qqqqqq.md", issue("todo", "parent: zb-pppppp\n")),
                (".zban/archive/zb-rrrrrr.md", issue("cancelled", "parent: zb-pppppp\n")),
            ],
            findings: vec![(".zban/zb-qqqqqq.md", "warning", "open_under_closed")],
        },
        Case {
            name: "misplaced",
            files: vec![
                (".zban/zb-aaaaaa.md", issue("done", "")),
                (".zban/archive/zb-bbbbbb.md", issue("todo", "")),
                (".zban/zb-cccccc.md", issue("in-progress", "")),
                (".zban/archive/zb-dddddd.md", issue("cancelled", "")),
            ],
            findings: vec![
                (".zban/archive/zb-bbbbbb.md", "error", "misplaced"),
                (".zban/zb-aaaaaa.md", "error", "misplaced"),
            ],
        },
    ]
}

// @mara implements VER-CHECK
#[test]
fn each_broken_project_reports_its_code_severity_and_path() {
    let mut covered = BTreeSet::new();
    for case in corpus() {
        let root = project(case.name, &case.files);
        let (status, findings) = check(&root, &[]);
        assert_eq!(findings, reported(&case.findings), "{}", case.name);
        let errors = case
            .findings
            .iter()
            .any(|&(_, severity, _)| severity == "error");
        assert_eq!(status, i32::from(errors), "{}", case.name);
        covered.extend(case.findings.iter().map(|&(_, _, code)| code));
    }
    assert_eq!(covered, BTreeSet::from(DES_CHECK_CODES));
}

// @mara implements VER-CHECK
#[test]
fn only_warnings_exit_zero_and_a_clean_project_prints_nothing() {
    let root = project(
        "warnings-only",
        &[
            (".zban/zb-aaaaaa.md", issue("todo", "assignee: alice\n")),
            (
                ".zban/zb-bbbbbb.md",
                issue("todo", "related:\n  - zb-cccccc\n"),
            ),
            (
                ".zban/zb-cccccc.md",
                issue("todo", "related:\n  - zb-bbbbbb\n"),
            ),
            (".zban/archive/zb-dddddd.md", issue("done", "")),
            (".zban/zb-eeeeee.md", issue("draft", "parent: zb-dddddd\n")),
        ],
    );
    let (status, findings) = check(&root, &[]);
    assert_eq!(status, 0);
    assert_eq!(
        findings,
        reported(&[
            (".zban/zb-aaaaaa.md", "warning", "unknown_key"),
            (".zban/zb-cccccc.md", "warning", "related_both_sides"),
            (".zban/zb-eeeeee.md", "warning", "open_under_closed"),
        ])
    );

    let clean = project("clean", &[(".zban/zb-aaaaaa.md", issue("todo", ""))]);
    let output = zban(&clean, &["check"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "");
}

// @mara implements VER-CHECK
#[test]
fn fix_moves_only_misplaced_files() {
    let root = project(
        "fix",
        &[
            (".zban/zb-aaaaaa.md", issue("done", "")),
            (
                ".zban/archive/zb-bbbbbb.md",
                issue("todo", "blocked_by:\n  - zb-000000\nassignee: alice\n"),
            ),
            (".zban/zb-cccccc.md", CONFLICTED.to_owned()),
            (".zban/zb-dddddd.md", issue("todo", "parent: zb-aaaaaa\n")),
            (".zban/archive/zb-eeeeee.md", issue("cancelled", "")),
            (
                ".zban/zb-ffffff.md",
                issue("todo", "parent: zb-gggggg\nblocked_by:\n  - zb-gggggg\n"),
            ),
            (".zban/zb-gggggg.md", issue("todo", "")),
        ],
    );
    let (_, before) = check(&root, &[]);
    assert!(before.iter().any(|(_, _, code)| code == "misplaced"));
    let before_fix = files(&root);

    let (status, fixed) = check(&root, &["--fix"]);
    let remaining = reported(&[
        (".zban/zb-bbbbbb.md", "error", "invalid_reference"),
        (".zban/zb-bbbbbb.md", "warning", "unknown_key"),
        (".zban/zb-cccccc.md", "error", "unparseable"),
        (".zban/zb-dddddd.md", "warning", "open_under_closed"),
        (".zban/zb-ffffff.md", "error", "cycle"),
    ]);
    assert_eq!(fixed, remaining);
    assert_eq!(status, 1);

    // Only the two misplaced files moved, each with its bytes unchanged.
    let expected = moved(
        before_fix,
        ".zban/zb-aaaaaa.md",
        ".zban/archive/zb-aaaaaa.md",
    );
    let expected = moved(expected, ".zban/archive/zb-bbbbbb.md", ".zban/zb-bbbbbb.md");
    assert_eq!(files(&root), expected);

    assert_eq!(check(&root, &[]), (1, remaining));
}

// @mara implements VER-CHECK
#[test]
fn a_misplaced_file_whose_target_is_taken_stays_and_is_reported() {
    let root = project(
        "fix-taken",
        &[
            (".zban/zb-aaaaaa.md", issue("done", "")),
            (".zban/archive/zb-aaaaaa.md", issue("done", "")),
            (".zban/zb-bbbbbb.md", issue("todo", "")),
            (".zban/archive/zb-bbbbbb.md", issue("todo", "")),
            (".zban/archive/zb-cccccc.md", issue("todo", "")),
        ],
    );
    let before_fix = files(&root);
    let (status, findings) = check(&root, &["--fix"]);
    assert_eq!(status, 1);
    assert_eq!(
        findings,
        reported(&[
            (".zban/archive/zb-aaaaaa.md", "error", "duplicate_id"),
            (".zban/archive/zb-bbbbbb.md", "error", "duplicate_id"),
            (".zban/archive/zb-bbbbbb.md", "error", "misplaced"),
            (".zban/zb-aaaaaa.md", "error", "duplicate_id"),
            (".zban/zb-aaaaaa.md", "error", "misplaced"),
            (".zban/zb-bbbbbb.md", "error", "duplicate_id"),
        ])
    );
    let expected = moved(
        before_fix,
        ".zban/archive/zb-cccccc.md",
        ".zban/zb-cccccc.md",
    );
    assert_eq!(files(&root), expected);

    let output = zban(&root, &["check"]);
    assert!(
        stdout(&output).contains(
            ".zban/zb-aaaaaa.md: error: the status done belongs in .zban/archive/, but a file with this name already exists there [misplaced]"
        ),
        "{}",
        stdout(&output)
    );
}

// @mara implements VER-CHECK
#[test]
fn findings_carry_severity_code_absolute_path_and_message() {
    let root = project(
        "library",
        &[
            (
                ".zban/zb-aaaaaa.md",
                issue("done", "assignee: alice\nblocked_by:\n  - zb-000000\n"),
            ),
            (".zban/zb-bbbbbb.md", issue("todo", "parent: zb-cccccc\n")),
            (
                ".zban/zb-cccccc.md",
                issue("todo", "blocked_by:\n  - zb-bbbbbb\n"),
            ),
        ],
    );
    let findings = Project::check(&root, false).unwrap();
    let root = fs::canonicalize(&root).unwrap();
    let summary: Vec<_> = findings
        .iter()
        .map(|finding| {
            assert_eq!(finding.severity, finding.code.severity());
            let path = Path::new(&finding.path)
                .strip_prefix(&root)
                .expect("an absolute path");
            (
                path.to_str().unwrap(),
                finding.severity,
                finding.code.as_str(),
                finding.message.as_str(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            (
                ".zban/zb-aaaaaa.md",
                Severity::Error,
                "invalid_reference",
                "`blocked_by` names zb-000000, which does not exist",
            ),
            (
                ".zban/zb-aaaaaa.md",
                Severity::Error,
                "misplaced",
                "the status done belongs in .zban/archive/; `zban check --fix` moves the file",
            ),
            (
                ".zban/zb-aaaaaa.md",
                Severity::Warning,
                "unknown_key",
                "unknown key `assignee`",
            ),
            (
                ".zban/zb-bbbbbb.md",
                Severity::Error,
                "cycle",
                "on a cycle through `parent` and `blocked_by`: zb-bbbbbb -> zb-cccccc -> zb-bbbbbb",
            ),
            (
                ".zban/zb-cccccc.md",
                Severity::Error,
                "cycle",
                "on a cycle through `parent` and `blocked_by`: zb-cccccc -> zb-bbbbbb -> zb-cccccc",
            ),
        ]
    );
}

// @mara implements VER-CHECK
#[test]
fn json_lists_each_finding_with_its_absolute_path() {
    let root = project(
        "json",
        &[
            (".zban/zb-aaaaaa.md", issue("done", "assignee: alice\n")),
            (
                ".zban/zb-bbbbbb.md",
                issue("todo", "related:\n  - zb-cccccc\n"),
            ),
            (
                ".zban/zb-cccccc.md",
                issue("todo", "related:\n  - zb-bbbbbb\n"),
            ),
        ],
    );
    let canonical = fs::canonicalize(&root).unwrap();
    let path = |relative: &str| canonical.join(relative).display().to_string();
    let (status, document) = check_json(&root, &[]);
    assert_eq!(status, 1);
    assert_eq!(
        document,
        json!({"findings": [
            {
                "severity": "error",
                "code": "misplaced",
                "path": path(".zban/zb-aaaaaa.md"),
                "message": "the status done belongs in .zban/archive/; `zban check --fix` moves the file",
            },
            {
                "severity": "warning",
                "code": "unknown_key",
                "path": path(".zban/zb-aaaaaa.md"),
                "message": "unknown key `assignee`",
            },
            {
                "severity": "warning",
                "code": "related_both_sides",
                "path": path(".zban/zb-cccccc.md"),
                "message": "zb-bbbbbb stores the same `related` link to this issue; keep it on one of the two",
            },
        ]})
    );

    // Paths stay absolute when zban runs elsewhere with `-C`.
    let outside = temp_dir_outside_projects("check-json-outside");
    let output = zban(&outside, &["-C", root.to_str().unwrap(), "check", "--json"]);
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        document
    );
    fs::remove_dir_all(outside).unwrap();

    // Fixed, the misplaced file is no longer reported, and warnings alone exit 0.
    let (status, document) = check_json(&root, &["--fix"]);
    assert_eq!(status, 0);
    let findings = document["findings"].as_array().unwrap();
    let summary: Vec<_> = findings
        .iter()
        .map(|finding| {
            (
                finding["severity"].as_str().unwrap(),
                finding["code"].as_str().unwrap(),
                finding["path"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            ("warning", "unknown_key", path(".zban/archive/zb-aaaaaa.md")),
            ("warning", "related_both_sides", path(".zban/zb-cccccc.md")),
        ]
    );

    let clean = project("json-clean", &[(".zban/zb-aaaaaa.md", issue("todo", ""))]);
    assert_eq!(check_json(&clean, &[]), (0, json!({"findings": []})));

    let broken = project(
        "json-config",
        &[(
            ".zban/config.toml",
            "format_version = 2\nprefix = \"zb\"\n".to_owned(),
        )],
    );
    let (status, document) = check_json(&broken, &[]);
    assert_eq!(status, 1);
    let config = fs::canonicalize(&broken).unwrap().join(".zban/config.toml");
    assert_eq!(
        document,
        json!({"findings": [{
            "severity": "error",
            "code": "invalid_config",
            "path": config.display().to_string(),
            "message": "unsupported format_version 2; this zban reads format 1",
        }]})
    );
}

// @mara implements VER-CHECK
#[test]
fn this_repository_backlog_is_clean() {
    let root = temp_dir("check-repository");
    copy_repository_backlog(&root);
    assert_eq!(check(&root, &[]), (0, Vec::new()));
}

#[test]
fn check_finds_its_project_like_every_command() {
    let outside = temp_dir_outside_projects("check-outside");
    let output = zban(&outside, &["check"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("[project_not_found]"),
        "{}",
        stderr(&output)
    );
    let output = zban(&outside, &["check", "--json"]);
    assert_eq!(output.status.code(), Some(1));
    let document: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(document["error"]["code"], "project_not_found");

    // With `-C`, paths outside the working directory are shown in full.
    let root = project(
        "directory-option",
        &[(".zban/zb-aaaaaa.md", issue("done", ""))],
    );
    let output = zban(
        &outside,
        &["-C", root.join(".zban").to_str().unwrap(), "check"],
    );
    assert_eq!(output.status.code(), Some(1));
    let path = fs::canonicalize(&root).unwrap().join(".zban/zb-aaaaaa.md");
    assert!(
        stdout(&output).starts_with(&format!("{}: error: ", path.display())),
        "{}",
        stdout(&output)
    );
    fs::remove_dir_all(outside).unwrap();
}
