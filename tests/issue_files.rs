//! `VER-ISSUE-FILES`: issue files round-trip through the core library.

mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

use common::{
    copy_repository_backlog, json_error_code, racing_updates, stderr, temp_dir, write_files, zban,
};
use tempfile::TempDir;
use zban::{
    ErrorCode, Id, Issue, IssueFile, IssueType, Prefix, Priority, ProblemCode, Project, Status,
    Timestamp,
};

const CONFIG: &str = "format_version = 1\nprefix = \"zb\"\n";

fn project(files: &[(&str, &str)]) -> (TempDir, Project) {
    let dir = temp_dir();
    write_files(dir.path(), &[(".zban/config.toml", CONFIG)]);
    write_files(dir.path(), files);
    let project = Project::open(dir.path()).unwrap();
    (dir, project)
}

fn load_one(project: &Project, id: &str) -> IssueFile {
    project
        .load()
        .unwrap()
        .into_iter()
        .find(|file| file.id.as_ref().map(Id::as_str) == Some(id))
        .unwrap_or_else(|| panic!("{id} is loaded"))
}

fn id(value: &str) -> Id {
    Id::parse(&Default::default(), value).unwrap()
}

fn timestamp(value: &str) -> Timestamp {
    Timestamp::parse(value).unwrap()
}

/// Writes every issue back without changes and returns the files that were
/// not byte-identical afterwards.
fn rewrite_all(project: &Project) -> Vec<String> {
    let files = project.load().unwrap();
    assert!(!files.is_empty());
    let mut changed = Vec::new();
    for file in files {
        let issue = file
            .issue
            .as_ref()
            .unwrap_or_else(|| panic!("{}: {:?}", file.path.display(), file.problems));
        let written = project.replace(&file, issue).unwrap();
        assert_eq!(written.issue.as_ref(), Some(issue));
        if fs::read(&file.path).unwrap() != file.bytes {
            changed.push(file.path.display().to_string());
        }
    }
    changed
}

// @mara implements VER-ISSUE-FILES
#[test]
fn repository_backlog_is_written_back_byte_identical() {
    let dir = temp_dir();
    copy_repository_backlog(dir.path());
    let project = Project::open(dir.path()).unwrap();
    let files = project.load().unwrap();
    assert!(files.iter().any(|file| file.archived));
    assert!(files.iter().any(|file| !file.archived));
    for file in &files {
        assert_eq!(file.problems, [], "{}", file.path.display());
    }
    let changed = rewrite_all(&project);
    assert_eq!(
        changed,
        Vec::<String>::new(),
        "this repository's backlog has files that are not in canonical form"
    );
}

const CANONICAL_FULL: &str = "\
---
title: Parse issue files
status: in-progress
type: feature
priority: high
tags:
  - storage
  - yaml-1
parent: zb-9jra34
blocked_by:
  - zb-naex76
  - zb-2b3mf7
related:
  - zb-ramrc1
discovered_from: zb-as3s58
created_at: 2026-11-11T20:00:00Z
estimate:
  low: 1 # days
  high: 3
reviewers: [ana, bo]
---

Load every issue.

- [x] Read frontmatter
- [ ] Preserve unknown keys
";

const CANONICAL_CLOSED: &str = "\
---
title: \"Titles: quoted when needed\"
status: cancelled
type: bug
priority: deferred
created_at: 2026-11-11T20:00:00Z
closed_at: 2026-11-12T08:30:00Z
---

## Reasons for Cancelling

Duplicate.";

// @mara implements VER-ISSUE-FILES
#[test]
fn canonical_files_are_written_back_byte_identical() {
    let (_dir, project) = project(&[
        (".zban/zb-n938cw.md", CANONICAL_FULL),
        (
            ".zban/zb-aaaaaa.md",
            "---\ntitle: Empty body\nstatus: draft\ntype: epic\ncreated_at: 2026-11-11T20:00:00Z\n---\n",
        ),
        (
            ".zban/zb-bbbbbb.md",
            "---\ntitle: Body with leading blank lines and no final newline\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z\n---\n\n\n\n---\nnot a header  \n\tindented",
        ),
        (".zban/archive/zb-cccccc.md", CANONICAL_CLOSED),
    ]);
    assert_eq!(rewrite_all(&project), Vec::<String>::new());

    let full = load_one(&project, "zb-n938cw");
    let issue = full.issue.unwrap();
    assert_eq!(issue.title, "Parse issue files");
    assert_eq!(issue.status, Status::InProgress);
    assert_eq!(issue.issue_type, IssueType::Feature);
    assert_eq!(issue.priority, Priority::High);
    assert_eq!(issue.tags, ["storage", "yaml-1"]);
    assert_eq!(issue.parent, Some(id("zb-9jra34")));
    assert_eq!(issue.blocked_by, [id("zb-naex76"), id("zb-2b3mf7")]);
    assert_eq!(issue.related, [id("zb-ramrc1")]);
    assert_eq!(issue.discovered_from, Some(id("zb-as3s58")));
    assert_eq!(issue.created_at, timestamp("2026-11-11T20:00:00Z"));
    assert_eq!(issue.closed_at, None);
    assert_eq!(
        issue.unknown,
        [
            "estimate:\n  low: 1 # days\n  high: 3\n",
            "reviewers: [ana, bo]\n"
        ]
    );
    assert_eq!(
        issue.body,
        "Load every issue.\n\n- [x] Read frontmatter\n- [ ] Preserve unknown keys\n"
    );
    let codes: Vec<_> = full.problems.iter().map(|problem| problem.code).collect();
    assert_eq!(codes, [ProblemCode::UnknownKey, ProblemCode::UnknownKey]);

    let closed = load_one(&project, "zb-cccccc");
    assert!(closed.archived);
    let issue = closed.issue.unwrap();
    assert_eq!(issue.title, "Titles: quoted when needed");
    assert_eq!(issue.closed_at, Some(timestamp("2026-11-12T08:30:00Z")));
    assert_eq!(issue.body, "## Reasons for Cancelling\n\nDuplicate.");
}

// @mara implements VER-ISSUE-FILES
#[test]
fn metadata_updates_keep_body_bytes_and_unknown_keys() {
    let body = "Body with ---\n---\ntrailing spaces   \nCRLF line\r\n\n\n```yaml\ntitle: not a header\n```";
    let original = CANONICAL_FULL.replace(
        "\nLoad every issue.\n\n- [x] Read frontmatter\n- [ ] Preserve unknown keys\n",
        &format!("\n{body}"),
    );
    let (_dir, project) = project(&[(".zban/zb-n938cw.md", &original)]);
    let file = load_one(&project, "zb-n938cw");
    let mut issue = file.issue.clone().unwrap();
    issue.title = "Parse and write issue files".into();
    issue.status = Status::Todo;
    issue.priority = Priority::Normal;
    issue.tags.push("format".into());
    issue.parent = None;
    issue.blocked_by.clear();
    issue.related.push(id("zb-yj8xta"));
    project.replace(&file, &issue).unwrap();

    let written = fs::read_to_string(&file.path).unwrap();
    assert_eq!(
        written,
        format!(
            "\
---
title: Parse and write issue files
status: todo
type: feature
tags:
  - storage
  - yaml-1
  - format
related:
  - zb-ramrc1
  - zb-yj8xta
discovered_from: zb-as3s58
created_at: 2026-11-11T20:00:00Z
estimate:
  low: 1 # days
  high: 3
reviewers: [ana, bo]
---

{body}"
        )
    );
    let reloaded = load_one(&project, "zb-n938cw");
    assert_eq!(reloaded.issue, Some(issue));
}

// @mara implements VER-ISSUE-FILES
#[test]
fn hand_written_header_is_rewritten_in_canonical_form() {
    let hand_written = "\
---
# Written by hand.
title: 'Hand-written: header'   # with a comment
status: \"in-progress\"
type: feature
priority: normal
tags: [storage, \"yaml\"]
blocked_by: [ZB-AAAAAA]
related:
- zb-bbbbbb
discovered_from:
created_at: '2026-11-11T20:00:00Z'
custom: value # kept with the key

# A comment between keys.
---
Body right after the header.
";
    let (_dir, project) = project(&[(".zban/zb-n938cw.md", hand_written)]);
    let file = load_one(&project, "zb-n938cw");
    let issue = file.issue.clone().unwrap();
    assert_eq!(issue.title, "Hand-written: header");
    assert_eq!(issue.status, Status::InProgress);
    assert_eq!(issue.priority, Priority::Normal);
    assert_eq!(issue.tags, ["storage", "yaml"]);
    assert_eq!(issue.blocked_by, [id("zb-aaaaaa")]);
    assert_eq!(issue.discovered_from, None);
    assert_eq!(issue.body, "Body right after the header.\n");

    project.replace(&file, &issue).unwrap();
    let canonical = "\
---
title: \"Hand-written: header\"
status: in-progress
type: feature
tags:
  - storage
  - yaml
blocked_by:
  - zb-aaaaaa
related:
  - zb-bbbbbb
created_at: 2026-11-11T20:00:00Z
custom: value # kept with the key
---

Body right after the header.
";
    assert_eq!(fs::read_to_string(&file.path).unwrap(), canonical);
    assert_eq!(rewrite_unchanged(&project, "zb-n938cw"), canonical);
}

fn rewrite_unchanged(project: &Project, id: &str) -> String {
    let file = load_one(project, id);
    project
        .replace(&file, file.issue.as_ref().unwrap())
        .unwrap();
    fs::read_to_string(&file.path).unwrap()
}

/// Unknown entries holding block scalars, in headers written flush left and
/// indented. `zban check` accepts each file, and a status update through the
/// real binary writes every unknown entry back as it was read: the same lines
/// without the header's indentation.
// @mara implements VER-ISSUE-FILES
#[test]
fn unknown_block_scalars_survive_an_update_in_indented_headers() {
    // Unknown entries written before `status:` and after `created_at:`, and
    // the entries zban reads from them.
    let cases: [(&str, &str, &[&str]); 8] = [
        (
            "notes: |\n  line one\n  line two\n",
            "",
            &["notes: |\n  line one\n  line two\n"],
        ),
        (
            "summary: >\n  folded\n  text\n# about the status\n",
            "",
            &["summary: >\n  folded\n  text\n"],
        ),
        (
            "notes: |-\n  stripped\n\n",
            "",
            &["notes: |-\n  stripped\n\n"],
        ),
        ("notes: |+\n  kept\n\n", "", &["notes: |+\n  kept\n\n"]),
        ("empty: |\n", "", &["empty: |\n"]),
        (
            "extra:\n  nested: |\n    text\n",
            "",
            &["extra:\n  nested: |\n    text\n"],
        ),
        ("", "notes: |\n  last\n", &["notes: |\n  last\n"]),
        (
            "",
            "notes: >-\n  folded\nreviewers: [ana, bo]\nestimate: 3\n",
            &[
                "notes: >-\n  folded\n",
                "reviewers: [ana, bo]\n",
                "estimate: 3\n",
            ],
        ),
    ];
    let indent = |text: &str, by: &str| -> String {
        text.lines()
            .map(|line| match line {
                "" => "\n".to_owned(),
                line => format!("{by}{line}\n"),
            })
            .collect()
    };
    let mut files = Vec::new();
    for by in ["", "  "] {
        for (number, (before, after, unknown)) in cases.iter().enumerate() {
            let header = format!(
                "title: Block scalars\n{before}status: todo\ntype: task\ncreated_at: 2026-11-12T00:00:00Z\n{after}"
            );
            let id = format!("zb-{}0000{number}", by.len());
            let text = format!("---\n{}---\n\nBody\n", indent(&header, by));
            files.push((format!(".zban/{id}.md"), text, id, *unknown));
        }
    }
    let (_dir, project) = project(
        &files
            .iter()
            .map(|(path, text, ..)| (path.as_str(), text.as_str()))
            .collect::<Vec<_>>(),
    );
    let output = zban(project.root(), &["check"]);
    assert!(output.status.success(), "{}", stderr(&output));

    for (_, text, id, unknown) in &files {
        let output = zban(project.root(), &["update", id, "--status", "in-progress"]);
        assert!(output.status.success(), "{text}{}", stderr(&output));
        let written = fs::read_to_string(project.dir().join(format!("{id}.md"))).unwrap();
        assert_eq!(
            written,
            format!(
                "---\ntitle: Block scalars\nstatus: in-progress\ntype: task\ncreated_at: 2026-11-12T00:00:00Z\n{}---\n\nBody\n",
                unknown.concat()
            ),
            "{text}"
        );
        assert_eq!(load_one(&project, id).issue.unwrap().unknown, *unknown);
    }
    let output = zban(project.root(), &["check"]);
    assert!(output.status.success(), "{}", stderr(&output));
}

// @mara implements VER-ISSUE-FILES
#[test]
fn titles_that_look_like_other_yaml_survive_a_write_and_a_read() {
    let (_dir, project) = project(&[]);
    let titles = [
        "123",
        "-1",
        "1.5",
        "1e3",
        "0x1F",
        "0o17",
        ".inf",
        ".NaN",
        "true",
        "False",
        "null",
        "~",
        "yes",
        "no",
        "2026-11-11",
        "2026-11-11T20:00:00Z",
        "a: b",
        "key: value: more",
        "a #b",
        "trailing:",
        "- item",
        "? question",
        "[x]",
        "{x: 1}",
        "#hash",
        "&anchor",
        "*alias",
        "!tag",
        "|",
        ">",
        "%directive",
        "@at",
        "`tick",
        "'single'",
        "\"double\"",
        "  padded  ",
        "back\\slash \"quotes\"",
        "tab\tinside",
        "ünïcødé ✓",
        "---",
        "...",
    ];
    for title in titles {
        let created = project
            .create(&Issue::new(title, Timestamp::now()))
            .unwrap();
        let id = created.id.as_ref().unwrap().as_str().to_owned();
        assert_eq!(load_one(&project, &id).issue.unwrap().title, title);
        let mut issue = created.issue.clone().unwrap();
        issue.priority = Priority::Low;
        project.replace(&created, &issue).unwrap();
        assert_eq!(load_one(&project, &id).issue.unwrap().title, title);
    }
    let header_line = |title: &str| {
        let file = project
            .create(&Issue::new(title, Timestamp::now()))
            .unwrap();
        let text = String::from_utf8(file.bytes).unwrap();
        text.lines().nth(1).unwrap().to_owned()
    };
    assert_eq!(header_line("Parse issue files"), "title: Parse issue files");
    assert_eq!(header_line("123"), "title: \"123\"");
    assert_eq!(header_line("a: b"), "title: \"a: b\"");
    assert_eq!(header_line("a #b"), "title: \"a #b\"");
}

// @mara implements VER-ISSUE-FILES
#[test]
fn blank_line_after_the_header_is_not_part_of_the_body() {
    let header = "---\ntitle: T\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z\n---\n";
    let (_dir, project) = project(&[
        (".zban/zb-aaaaaa.md", &format!("{header}\nBody\n")),
        (".zban/zb-bbbbbb.md", &format!("{header}\n\nBody\n")),
        (".zban/zb-cccccc.md", &format!("{header}Body\n")),
        (".zban/zb-dddddd.md", &format!("{header}\n")),
        (".zban/zb-eeeeee.md", header),
        (".zban/zb-ffffff.md", header.trim_end()),
    ]);
    let body = |id: &str| load_one(&project, id).issue.unwrap().body;
    assert_eq!(body("zb-aaaaaa"), "Body\n");
    assert_eq!(body("zb-bbbbbb"), "\nBody\n");
    assert_eq!(body("zb-cccccc"), "Body\n");
    assert_eq!(body("zb-dddddd"), "");
    assert_eq!(body("zb-eeeeee"), "");
    assert_eq!(body("zb-ffffff"), "");
    for id in ["zb-dddddd", "zb-eeeeee", "zb-ffffff"] {
        assert_eq!(rewrite_unchanged(&project, id), header);
    }

    let created = project
        .create(&Issue::new("Empty", Timestamp::now()))
        .unwrap();
    let text = fs::read_to_string(&created.path).unwrap();
    assert!(text.ends_with("Z\n---\n"), "{text:?}");
}

// @mara implements VER-ISSUE-FILES
#[test]
fn values_outside_the_vocabulary_are_rejected() {
    for (result, word) in [
        ("open".parse::<Status>().map(|_| ()), "open"),
        ("Todo".parse::<Status>().map(|_| ()), "Todo"),
        ("story".parse::<IssueType>().map(|_| ()), "story"),
        ("urgent".parse::<Priority>().map(|_| ()), "urgent"),
    ] {
        let error = result.unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidValue);
        assert!(error.message.contains(word), "{}", error.message);
    }
    let error = "open".parse::<Status>().unwrap_err();
    assert!(
        error
            .message
            .contains("draft, todo, in-progress, done, cancelled"),
        "{}",
        error.message
    );
}

// @mara implements VER-ISSUE-FILES
#[test]
fn create_and_update_reject_invalid_values_without_writing() {
    let (_dir, project) = project(&[]);
    let valid = Issue::new("Valid", timestamp("2026-11-11T20:00:00Z"));
    let created = project.create(&valid).unwrap();
    let before = fs::read(&created.path).unwrap();

    let mut invalid = Vec::new();
    for title in ["", "two\nlines", "carriage\rreturn"] {
        invalid.push(Issue {
            title: title.into(),
            ..valid.clone()
        });
    }
    for tags in [
        vec!["Storage"],
        vec!["9lives"],
        vec!["a--b"],
        vec!["a-"],
        vec!["a", "a"],
    ] {
        invalid.push(Issue {
            tags: tags.into_iter().map(String::from).collect(),
            ..valid.clone()
        });
    }
    invalid.push(Issue {
        blocked_by: vec![id("zb-aaaaaa"), id("zb-aaaaaa")],
        ..valid.clone()
    });
    invalid.push(Issue {
        related: vec![id("zb-aaaaaa"), id("zb-aaaaaa")],
        ..valid.clone()
    });
    invalid.push(Issue {
        status: Status::Done,
        ..valid.clone()
    });
    invalid.push(Issue {
        closed_at: Some(Timestamp::now()),
        ..valid.clone()
    });
    invalid.push(Issue {
        parent: Some(Id::parse(&Prefix::parse("web").unwrap(), "web-aaaaaa").unwrap()),
        ..valid.clone()
    });
    for unknown in ["title: again\n", "custom: [unclosed\n", "custom: 1"] {
        invalid.push(Issue {
            unknown: vec![unknown.into()],
            ..valid.clone()
        });
    }

    for issue in &invalid {
        let error = project.create(issue).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidValue, "{issue:?}");
        let error = project.replace(&created, issue).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidValue, "{issue:?}");
    }
    assert_eq!(project.load().unwrap(), std::slice::from_ref(&created));
    assert_eq!(fs::read(&created.path).unwrap(), before);
    assert!(!project.archive_dir().exists());
}

// @mara implements VER-ISSUE-FILES
#[test]
fn loading_reports_each_broken_file_and_keeps_the_others() {
    let valid = "---\ntitle: T\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z\n---\n";
    let conflict = "---\n<<<<<<< HEAD\ntitle: Ours\n=======\ntitle: Theirs\n>>>>>>> branch\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z\n---\n";
    let header = |lines: &str| format!("---\n{lines}\n---\n");
    let cases: Vec<(&str, String, ProblemCode, &str)> = vec![
        (
            "zb-000001",
            "No header".into(),
            ProblemCode::Unparseable,
            "frontmatter",
        ),
        (
            "zb-000002",
            "---\ntitle: T\n".into(),
            ProblemCode::Unparseable,
            "frontmatter",
        ),
        (
            "zb-000003",
            conflict.into(),
            ProblemCode::Unparseable,
            "YAML",
        ),
        (
            "zb-000004",
            header("title: &a T\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z"),
            ProblemCode::Unparseable,
            "anchors",
        ),
        (
            "zb-000005",
            header("title: !!str T\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z"),
            ProblemCode::Unparseable,
            "tags",
        ),
        (
            "zb-000006",
            header(
                "title: T\ntitle: U\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z",
            ),
            ProblemCode::Unparseable,
            "duplicate key",
        ),
        (
            "zb-000007",
            header("status: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z"),
            ProblemCode::MissingField,
            "title",
        ),
        (
            "zb-000008",
            header("title: T\nstatus: done\ntype: task\ncreated_at: 2026-11-11T20:00:00Z"),
            ProblemCode::MissingField,
            "closed_at",
        ),
        (
            "zb-000009",
            header("title: T\nstatus: open\ntype: task\ncreated_at: 2026-11-11T20:00:00Z"),
            ProblemCode::InvalidValue,
            "status",
        ),
        (
            "zb-00000a",
            header("title: T\nstatus: todo\ntype: story\ncreated_at: 2026-11-11T20:00:00Z"),
            ProblemCode::InvalidValue,
            "type",
        ),
        (
            "zb-00000b",
            header(
                "title: T\nstatus: todo\ntype: task\npriority: p1\ncreated_at: 2026-11-11T20:00:00Z",
            ),
            ProblemCode::InvalidValue,
            "priority",
        ),
        (
            "zb-00000c",
            header(
                "title: T\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z\nclosed_at: 2026-11-12T20:00:00Z",
            ),
            ProblemCode::InvalidValue,
            "closed_at",
        ),
        (
            "zb-00000d",
            header("title: T\nstatus: todo\ntype: task\ncreated_at: yesterday"),
            ProblemCode::InvalidValue,
            "created_at",
        ),
        (
            "zb-00000e",
            header(
                "title: T\nstatus: todo\ntype: task\nparent: 42\ncreated_at: 2026-11-11T20:00:00Z",
            ),
            ProblemCode::InvalidValue,
            "parent",
        ),
        (
            "zb-00000f",
            header(
                "title: T\nstatus: todo\ntype: task\nblocked_by: zb-aaaaaa\ncreated_at: 2026-11-11T20:00:00Z",
            ),
            ProblemCode::InvalidValue,
            "list",
        ),
        (
            "zb-00000g",
            header(
                "title: T\nstatus: todo\ntype: task\ntags: [a, a]\ncreated_at: 2026-11-11T20:00:00Z",
            ),
            ProblemCode::InvalidValue,
            "more than once",
        ),
        (
            "zb-00000h",
            header("title: [T]\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z"),
            ProblemCode::InvalidValue,
            "single value",
        ),
        (
            "zb-00000j",
            header(
                "title: |\n  two\n  lines\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z",
            ),
            ProblemCode::InvalidValue,
            "single line",
        ),
    ];
    let mut files: Vec<(String, String)> = cases
        .iter()
        .map(|(id, content, _, _)| (format!(".zban/{id}.md"), content.clone()))
        .collect();
    files.push((".zban/zb-vvvvvv.md".into(), valid.into()));
    files.push((".zban/notes.md".into(), valid.into()));
    files.push((
        ".zban/zb-kkkkkk.md".into(),
        "---\ntitle: T\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z\ncustom: 1\n---\n".into(),
    ));
    let borrowed: Vec<(&str, &str)> = files
        .iter()
        .map(|(path, content)| (path.as_str(), content.as_str()))
        .collect();
    let (_dir, project) = project(&borrowed);
    fs::write(
        project.dir().join("zb-nnnnnn.md"),
        b"---\ntitle: \xff\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z\n---\n",
    )
    .unwrap();

    let loaded = project.load().unwrap();
    let find = |name: &str| {
        loaded
            .iter()
            .find(|file| file.path.file_name().unwrap() == name)
            .unwrap_or_else(|| panic!("{name} is loaded"))
    };
    for (id, _, code, reason) in &cases {
        let file = find(&format!("{id}.md"));
        assert_eq!(file.issue, None, "{id}");
        assert!(
            file.problems
                .iter()
                .any(|problem| problem.code == *code && problem.message.contains(reason)),
            "{id}: {:?}",
            file.problems
        );
    }
    assert!(find("zb-vvvvvv.md").issue.is_some());
    let unknown = find("zb-kkkkkk.md");
    assert!(unknown.issue.is_some());
    assert_eq!(unknown.problems[0].code, ProblemCode::UnknownKey);
    let misnamed = find("notes.md");
    assert_eq!(misnamed.id, None);
    assert_eq!(misnamed.issue, None);
    assert_eq!(misnamed.problems[0].code, ProblemCode::InvalidFilename);
    assert_eq!(
        find("zb-nnnnnn.md").problems[0].code,
        ProblemCode::Unparseable
    );
}

// @mara implements VER-ISSUE-FILES
#[test]
fn concurrent_change_fails_with_conflict_and_survives() {
    let (_dir, project) = project(&[]);
    let created = project
        .create(&Issue::new("Original", Timestamp::now()))
        .unwrap();
    let mut issue = created.issue.clone().unwrap();
    issue.status = Status::InProgress;

    let other_change = String::from_utf8(created.bytes.clone())
        .unwrap()
        .replace("Original", "Edited elsewhere");
    fs::write(&created.path, &other_change).unwrap();
    let error = project.replace(&created, &issue).unwrap_err();
    assert_eq!(error.code, ErrorCode::Conflict);
    assert_eq!(fs::read_to_string(&created.path).unwrap(), other_change);

    fs::remove_file(&created.path).unwrap();
    let error = project.replace(&created, &issue).unwrap_err();
    assert_eq!(error.code, ErrorCode::Conflict);
    assert!(!created.path.exists());

    let fresh = project
        .create(&Issue::new("Fresh", Timestamp::now()))
        .unwrap();
    let updated = project.replace(&fresh, &issue).unwrap();
    let error = project.replace(&fresh, &issue).unwrap_err();
    assert_eq!(error.code, ErrorCode::Conflict, "a stale read is rejected");
    project.replace(&updated, &issue).unwrap();
}

/// Runs of `zban update` racing on one issue: a run whose file another run
/// replaced after its read fails with `conflict` and writes nothing, so the
/// other change survives.
// @mara implements VER-ISSUE-FILES
#[test]
fn update_fails_with_conflict_when_another_run_replaced_the_file() {
    let (_dir, project) = project(&[]);
    let created = project
        .create(&Issue::new("Raced", Timestamp::now()))
        .unwrap();
    let id = created.id.unwrap();

    let runs = racing_updates(project.root(), id.as_str());
    let body = load_one(&project, id.as_str()).issue.unwrap().body;
    let failed: Vec<_> = runs
        .iter()
        .filter(|(_, output)| json_error_code(output).as_deref() == Some("conflict"))
        .collect();
    assert!(!failed.is_empty());
    for (line, output) in failed {
        assert_eq!(output.status.code(), Some(1));
        assert!(!body.contains(line.as_str()), "{line} in {body:?}");
    }
    // The file holds what the last successful run wrote.
    assert!(
        runs.iter()
            .any(|(line, output)| output.status.success() && body.contains(line.as_str())),
        "{body:?}"
    );
    assert_no_temporary_files(&project.dir());
}

/// An update writes a new file and renames it over the issue, so the issue's
/// path names another inode afterwards and the original file is never
/// written in place: a second name for it keeps the old bytes. That is what
/// keeps an interrupted update from leaving a partial issue.
// @mara implements VER-ISSUE-FILES
#[test]
fn update_renames_a_new_file_over_the_issue() {
    let (_dir, project) = project(&[]);
    let created = project
        .create(&Issue::new("Original", Timestamp::now()))
        .unwrap();
    let id = created.id.unwrap();
    let id = id.as_str();
    let original = project.root().join("original.md");

    for args in [
        &["update", id, "--title", "Renamed"][..],
        &["update", id, "--append", "More text."],
        &["update", id, "--status", "done"],
    ] {
        let before = load_one(&project, id);
        let _ = fs::remove_file(&original);
        fs::hard_link(&before.path, &original).unwrap();

        let output = zban(project.root(), args);
        assert!(output.status.success(), "{args:?}: {}", stderr(&output));

        assert_ne!(load_one(&project, id).bytes, before.bytes, "{args:?}");
        assert_eq!(
            fs::read_to_string(&original).unwrap(),
            String::from_utf8_lossy(&before.bytes),
            "{args:?}"
        );
    }
    assert_eq!(load_one(&project, id).issue.unwrap().status, Status::Done);
    assert_no_temporary_files(&project.dir());
    assert_no_temporary_files(&project.archive_dir());
}

// @mara implements VER-ISSUE-FILES
#[test]
fn create_never_replaces_an_existing_file() {
    let (_dir, project) = project(&[]);
    let existing = project
        .create(&Issue::new("Existing", Timestamp::now()))
        .unwrap();
    let existing_id = existing.id.clone().unwrap();

    let mut draws = vec![existing_id.clone(), id("zb-fresh1")].into_iter();
    let created = project
        .create_with(&Issue::new("New", Timestamp::now()), || {
            Ok(draws.next().unwrap())
        })
        .unwrap();
    assert_eq!(created.id, Some(id("zb-fresh1")));
    assert_eq!(fs::read(&existing.path).unwrap(), existing.bytes);

    let error = project
        .create_with(&Issue::new("Never written", Timestamp::now()), || {
            Ok(existing_id.clone())
        })
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::Io);
    assert_eq!(fs::read(&existing.path).unwrap(), existing.bytes);
    assert_eq!(project.load().unwrap().len(), 2);
    assert_no_temporary_files(&project.dir());
}

// @mara implements VER-ISSUE-FILES
#[test]
fn init_that_cannot_write_its_configuration_leaves_no_zban() {
    let dir = temp_dir();
    let root = dir.path();
    // A file size limit of zero makes writing the configuration fail with
    // `EFBIG` on any file system and for any user, hard links or not. The
    // shell ignores the signal that such a write raises, and zban inherits
    // that.
    let output = Command::new("sh")
        .current_dir(root)
        .args([
            "-c",
            r#"trap '' XFSZ; ulimit -f 0; exec "$0" --json init"#,
            env!("CARGO_BIN_EXE_zban"),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert_eq!(json_error_code(&output).as_deref(), Some("io"));
    assert_eq!(fs::read_dir(root).unwrap().count(), 0);

    let output = zban(root, &["init"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        fs::read_to_string(root.join(".zban/config.toml")).unwrap(),
        CONFIG
    );
}

fn assert_no_temporary_files(dir: &Path) {
    for entry in fs::read_dir(dir).unwrap() {
        let name = entry.unwrap().file_name();
        assert!(!name.to_string_lossy().ends_with(".tmp"), "{name:?}");
    }
}
