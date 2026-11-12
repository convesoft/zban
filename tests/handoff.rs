//! `VER-HANDOFF`: writing, replacing, removing and reading handoff notes
//! through the real `zban` binary.

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use common::mcp::Client;
use common::{snapshot, stderr, stdout, temp_dir, write_files, zban, zban_with_input};
use serde_json::{Value, json};
use zban::Timestamp;

const CONFIG: &str = "format_version = 1\nprefix = \"zb\"\n";

const A: &str = "zb-aaaaaa";
const B: &str = "zb-bbbbbb";
const C: &str = "zb-cccccc";

/// The header of A as zban writes it, so that a rewrite keeps its bytes.
const HEADER: &str = "---\ntitle: Parse files\nstatus: in-progress\ntype: task\n\
                      created_at: 2026-01-01T00:00:01Z\nestimate: 3\n---\n";

/// A project whose issue A has `body`.
fn project(name: &str, body: &str) -> PathBuf {
    let root = temp_dir(&format!("handoff-{name}"));
    write_files(
        &root,
        &[
            (".zban/config.toml", CONFIG),
            (".zban/zb-aaaaaa.md", &format!("{HEADER}\n{body}")),
        ],
    );
    root
}

fn read(root: &Path) -> String {
    fs::read_to_string(root.join(".zban/zb-aaaaaa.md")).unwrap()
}

fn files(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    snapshot(&root.join(".zban"), &[])
}

fn document(args: &[&str], output: &Output) -> Value {
    assert!(output.status.success(), "{args:?}: {}", stdout(output));
    serde_json::from_slice(&output.stdout).unwrap()
}

/// Runs `zban --json <args>`, which must succeed, and returns its document.
fn ok(root: &Path, args: &[&str]) -> Value {
    let args = [&["--json"], args].concat();
    document(&args, &zban(root, &args))
}

/// Runs `zban --json <args>`, which must fail without changing any file,
/// and returns the error code.
fn refused(root: &Path, args: &[&str]) -> String {
    let before = files(root);
    let args = [&["--json"], args].concat();
    let output = zban(root, &args);
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let code = value["error"]["code"].as_str().unwrap().to_owned();
    let status = if code == "usage" { 2 } else { 1 };
    assert_eq!(output.status.code(), Some(status), "{args:?}: {value}");
    assert_eq!(files(root), before, "{args:?} wrote nothing");
    code
}

fn shown(root: &Path, id: &str) -> Value {
    ok(root, &["show", id])["issues"][0].clone()
}

/// The `handoff_at` of each listed issue, by ID.
fn handoff_times(root: &Path) -> Vec<(String, Value)> {
    ok(root, &["list"])["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|issue| {
            let id = issue["id"].as_str().unwrap().to_owned();
            (id, issue["handoff_at"].clone())
        })
        .collect()
}

/// The time in the last `## Handoff (<time>)` heading of `text`, checked to
/// lie between `before` and now. The callers compare the whole text.
fn written_at(text: &str, before: &Timestamp) -> String {
    let (_, rest) = text
        .rsplit_once("\n## Handoff (")
        .expect("a timestamped heading");
    let time = &rest[..rest.find(")\n").unwrap()];
    let parsed = Timestamp::parse(time).expect("an RFC 3339 time");
    assert!(before <= &parsed && parsed <= Timestamp::now(), "{time}");
    time.to_owned()
}

// @mara implements VER-HANDOFF
#[test]
fn writes_replaces_and_removes_the_note() {
    let root = project("write", "Intro.\n\n- [x] Step\n");
    let before = Timestamp::now();

    // Added to a body without one, after a blank line.
    let updated = ok(
        &root,
        &[
            "update",
            A,
            "--handoff",
            "- Done: reader\n- Next: writer\n\n",
        ],
    );
    assert_eq!(updated["id"], A);
    let text = read(&root);
    let first = written_at(&text, &before);
    assert_eq!(
        text,
        format!(
            "{HEADER}\nIntro.\n\n- [x] Step\n\n## Handoff ({first})\n\n- Done: reader\n- Next: writer\n"
        )
    );

    // Replaced in place, from standard input, with a section after it.
    ok(&root, &["update", A, "--append", "## Notes\n\nKeep these."]);
    let old = read(&root);
    let start = old.find("## Handoff (").unwrap();
    let end = old.find("## Notes").unwrap();
    let note = "- Next: tests\n\n### Detail\n\n```markdown\n## Not a heading\n```\n";
    let output = zban_with_input(&root, &["--json", "update", A, "--handoff", "-"], note);
    document(&["update", "--handoff", "-"], &output);
    let new = read(&root);
    let second = written_at(&new, &before);
    assert!(new.starts_with(&old[..start]), "{new}");
    assert!(new.ends_with(&old[end..]), "{new}");
    assert_eq!(
        &new[start..new.len() - (old.len() - end)],
        format!("## Handoff ({second})\n\n{note}\n")
    );

    // Read back by `show` and `list`.
    let issue = shown(&root, A);
    assert_eq!(
        issue["handoff"],
        json!({"written_at": second, "text": note})
    );
    assert_eq!(issue["body"], new[HEADER.len() + 1..]);
    assert_eq!(handoff_times(&root), [(A.to_owned(), json!(second))]);

    // Removed, leaving every byte outside the section.
    ok(&root, &["update", A, "--no-handoff"]);
    assert_eq!(
        read(&root),
        format!("{HEADER}\nIntro.\n\n- [x] Step\n\n## Notes\n\nKeep these.\n")
    );
    assert_eq!(shown(&root, A)["handoff"], Value::Null);
    assert_eq!(handoff_times(&root), [(A.to_owned(), Value::Null)]);

    // A note at the end goes with the blank line before it.
    ok(&root, &["update", A, "--handoff", "Last"]);
    ok(&root, &["update", A, "--no-handoff"]);
    assert_eq!(
        read(&root),
        format!("{HEADER}\nIntro.\n\n- [x] Step\n\n## Notes\n\nKeep these.\n")
    );
}

// @mara implements VER-HANDOFF
#[test]
fn rejects_headings_that_would_end_the_note() {
    let root = project("headings", "Intro.\n");
    for text in [
        "- Next: x\n## Next steps\n- y",
        "# Title",
        "```\nunclosed fence\n## Swallowed",
    ] {
        assert_eq!(
            refused(&root, &["update", A, "--handoff", text]),
            "invalid_handoff",
            "{text:?}"
        );
    }
    assert_eq!(refused(&root, &["update", A, "--handoff", "\n"]), "usage");
    assert_eq!(
        refused(&root, &["update", A, "--handoff", "x", "--no-handoff"]),
        "usage"
    );
    let output = zban_with_input(
        &root,
        &["update", A, "--append", "-", "--handoff", "-"],
        "x",
    );
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
}

// @mara implements VER-HANDOFF
#[test]
fn a_handoff_heading_in_a_fenced_code_block_is_ignored() {
    let example = "Example:\n\n```markdown\n## Handoff (2026-01-01T00:00:00Z)\n\nOld\n```\n";
    let root = project("fenced", example);
    assert_eq!(shown(&root, A)["handoff"], Value::Null);
    assert_eq!(handoff_times(&root), [(A.to_owned(), Value::Null)]);

    let before = Timestamp::now();
    ok(&root, &["update", A, "--handoff", "New"]);
    let text = read(&root);
    let time = written_at(&text, &before);
    assert_eq!(
        text,
        format!("{HEADER}\n{example}\n## Handoff ({time})\n\nNew\n")
    );

    ok(&root, &["update", A, "--no-handoff"]);
    assert_eq!(read(&root), format!("{HEADER}\n{example}"));
}

// @mara implements VER-HANDOFF
#[test]
fn several_handoff_sections_make_both_edits_fail() {
    let body = "## Handoff\n\nOne\n\n## Handoff (2026-01-01T00:00:00Z)\n\nTwo\n";
    let root = project("several", body);
    for args in [&["--handoff", "Three"][..], &["--no-handoff"]] {
        let args = [&["update", A], args].concat();
        assert_eq!(refused(&root, &args), "invalid_handoff", "{args:?}");
    }
    let issue = shown(&root, A);
    assert_eq!(
        issue["handoff"],
        json!({"written_at": null, "text": "One\n"}),
        "reading uses the first"
    );
}

// @mara implements VER-HANDOFF
#[test]
fn show_and_list_report_the_note_and_its_time() {
    let root = project("read", "## Handoff\n\n- Next: undated\n");
    let issue = |title: &str, body: &str| {
        format!(
            "---\ntitle: {title}\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:02Z\n---\n\n{body}"
        )
    };
    write_files(
        &root,
        &[
            (
                ".zban/zb-bbbbbb.md",
                &issue(
                    "Dated",
                    "Plan.\n\n## Handoff (2026-02-03T04:05:06Z)\n\n- Next: dated\n",
                ),
            ),
            (".zban/zb-cccccc.md", &issue("Without", "Plan.\n")),
        ],
    );

    let shown = ok(&root, &["show", A, B, C])["issues"].clone();
    assert_eq!(
        shown[0]["handoff"],
        json!({"written_at": null, "text": "- Next: undated\n"})
    );
    assert_eq!(
        shown[1]["handoff"],
        json!({"written_at": "2026-02-03T04:05:06Z", "text": "- Next: dated\n"})
    );
    assert_eq!(shown[2]["handoff"], Value::Null);
    assert_eq!(
        handoff_times(&root),
        [
            (A.to_owned(), Value::Null),
            (B.to_owned(), json!("2026-02-03T04:05:06Z")),
            (C.to_owned(), Value::Null),
        ]
    );

    // Human output shows the age of the note.
    let human = stdout(&zban(&root, &["show", A, B, C]));
    assert!(human.contains("\nhandoff: time unknown\n"), "{human}");
    assert!(
        human.contains("\nhandoff: 2026-02-03T04:05:06Z, ") && human.contains(" days ago\n"),
        "{human}"
    );
    assert_eq!(human.matches("\nhandoff: ").count(), 2, "{human}");
    let listed = stdout(&zban(&root, &["list"]));
    let line = |id: &str| listed.lines().find(|line| line.starts_with(id)).unwrap();
    assert!(line(B).ends_with(" days ago"), "{listed}");
    assert!(!line(A).contains("handoff"), "{listed}");
    assert!(!line(C).contains("handoff"), "{listed}");
}

// @mara implements VER-HANDOFF
#[test]
fn closing_leaves_the_handoff_untouched() {
    let root = project("close", "Intro.\n");
    ok(
        &root,
        &["update", A, "--handoff", "- Rejected: a generic YAML value"],
    );
    let body = shown(&root, A)["body"].clone();
    let handoff = shown(&root, A)["handoff"].clone();

    ok(&root, &["update", A, "--status", "done"]);
    assert!(!root.join(".zban/zb-aaaaaa.md").exists());
    let closed = shown(&root, A);
    assert!(closed["path"].as_str().unwrap().contains("/archive/"));
    assert_eq!(closed["body"], body);
    assert_eq!(closed["handoff"], handoff);
}

// @mara implements VER-HANDOFF
#[test]
fn prime_and_the_mcp_instructions_carry_no_handoff_notes() {
    let marker = "Unique handoff marker 7f3a";
    let root = project("prime", "Intro.\n");
    ok(&root, &["update", A, "--handoff", marker]);
    assert!(read(&root).contains(marker));

    let snapshot =
        |path: &str| fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
    let prime = zban(&root, &["prime"]);
    assert!(prime.status.success());
    assert_eq!(stdout(&prime), snapshot("tests/snapshots/prime.md"));

    let client = Client::start(&root, &[]);
    let instructions = client.instructions().to_owned();
    assert!(client.finish().success());
    assert_eq!(instructions, snapshot("tests/snapshots/mcp.md"));
    assert!(!instructions.contains(marker));
}
