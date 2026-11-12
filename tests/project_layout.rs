//! `DES-PROJECT-LAYOUT`: creating and finding a project, and which files in it
//! are issues.

mod common;

use std::fs;

use common::{stderr, stdout, temp_dir, write_files, zban};
use zban::{ErrorCode, ProblemCode, Project};

const VALID: &str =
    "---\ntitle: T\nstatus: todo\ntype: task\ncreated_at: 2026-11-11T20:00:00Z\n---\n";

#[test]
fn init_creates_the_project_once() {
    let dir = temp_dir();
    let root = dir.path();
    let output = zban(root, &["init"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("ID prefix zb"));
    assert!(root.join(".zban/config.toml").is_file());
    assert!(!root.join(".zban/archive").exists());
    assert_eq!(fs::read_dir(root.join(".zban")).unwrap().count(), 1);

    let output = zban(root, &["init", "--prefix", "web"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("project_exists"));
    assert_eq!(
        fs::read_to_string(root.join(".zban/config.toml")).unwrap(),
        "format_version = 1\nprefix = \"zb\"\n"
    );
}

#[test]
fn init_uses_the_directory_option() {
    let dir = temp_dir();
    let root = dir.path();
    fs::create_dir_all(root.join("a/b")).unwrap();
    let output = zban(&root.join("a"), &["-C", "b", "init"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(root.join("a/b/.zban/config.toml").is_file());

    let output = zban(
        root,
        &["init", "--directory", root.join("a").to_str().unwrap()],
    );
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(root.join("a/.zban/config.toml").is_file());
    assert!(!root.join(".zban").exists());

    let output = zban(root, &["-C", "missing", "init"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("[io]"));
}

#[test]
fn discovery_walks_up_to_the_nearest_project() {
    let dir = temp_dir();
    let root = dir.path();
    Project::init(root, None).unwrap();
    fs::create_dir_all(root.join("a/b/c")).unwrap();
    let found = Project::discover(&root.join("a/b/c")).unwrap();
    assert_eq!(found.root(), fs::canonicalize(root).unwrap());

    Project::init(&root.join("a"), Some("inner")).unwrap();
    // A `.zban/` without a configuration is not a project.
    fs::create_dir_all(root.join("a/b/.zban")).unwrap();
    let found = Project::discover(&root.join("a/b/c")).unwrap();
    assert_eq!(found.root(), fs::canonicalize(root.join("a")).unwrap());
    assert_eq!(found.prefix().as_str(), "inner");
    let found = Project::discover(&root.join("a/b/c/../../..")).unwrap();
    assert_eq!(found.root(), fs::canonicalize(root).unwrap());
}

#[test]
fn discovery_outside_a_project_fails() {
    let dir = temp_dir();
    let error = Project::discover(dir.path()).unwrap_err();
    assert_eq!(error.code, ErrorCode::ProjectNotFound);
}

#[test]
fn invalid_configuration_is_reported() {
    for config in [
        "format_version = 1\nprefix = \"zb\"\nextra = 1\n",
        "format_version = 2\nprefix = \"zb\"\n",
        "format_version = 1\nprefix = \"Zb\"\n",
        "format_version = \n",
    ] {
        let dir = temp_dir();
        let root = dir.path();
        write_files(root, &[(".zban/config.toml", config)]);
        let error = Project::discover(root).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidConfig, "{config:?}");
        assert!(error.message.contains("config.toml"), "{}", error.message);
    }
}

#[test]
fn only_markdown_files_in_the_two_directories_are_issues() {
    let dir = temp_dir();
    let root = dir.path();
    write_files(
        root,
        &[
            (".zban/config.toml", "format_version = 1\nprefix = \"zb\"\n"),
            (".zban/zb-aaaaaa.md", VALID),
            (".zban/archive/zb-bbbbbb.md", VALID),
            (".zban/README.md", VALID),
            (".zban/notes.txt", VALID),
            (".zban/zb-cccccc.md.orig", VALID),
            (".zban/.zb-dddddd.md", VALID),
            (".zban/drafts/zb-eeeeee.md", VALID),
            (".zban/archive/old/zb-ffffff.md", VALID),
            ("zb-gggggg.md", VALID),
        ],
    );
    fs::create_dir(root.join(".zban/zb-hhhhhh.md")).unwrap();
    let project = Project::open(root).unwrap();
    let loaded = project.load().unwrap();
    let names: Vec<_> = loaded
        .iter()
        .map(|file| {
            file.path
                .strip_prefix(project.dir())
                .unwrap()
                .to_str()
                .unwrap()
        })
        .collect();
    assert_eq!(
        names,
        [
            ".zb-dddddd.md",
            "README.md",
            "archive/zb-bbbbbb.md",
            "zb-aaaaaa.md"
        ]
    );
    for misnamed in &loaded[..2] {
        assert_eq!(misnamed.problems[0].code, ProblemCode::InvalidFilename);
    }
    assert!(loaded[2].archived && !loaded[3].archived);
}
