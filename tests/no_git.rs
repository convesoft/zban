//! `VER-NO-GIT`: zban works without version control and writes only inside
//! `.zban/`.

mod common;

use std::fs;
use std::path::Path;

use common::{snapshot, stderr, stdout, temp_dir, write_files, zban};

/// Runs a command that must succeed, with `--json`, and returns its document.
fn ok(dir: &Path, args: &[&str]) -> serde_json::Value {
    let output = zban(dir, &[&["--json"], args].concat());
    assert!(
        output.status.success(),
        "{args:?}: {} {}",
        stdout(&output),
        stderr(&output)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn id(value: &serde_json::Value) -> String {
    value["id"].as_str().unwrap().to_owned()
}

// @mara implements VER-NO-GIT
#[test]
fn every_command_works_outside_git_and_writes_only_inside_zban() {
    let dir = temp_dir();
    let root = dir.path();
    for dir in root.ancestors() {
        assert!(!dir.join(".git").exists(), "{} is in Git", root.display());
    }
    write_files(
        root,
        &[
            ("README.md", "# Project\n"),
            ("src/lib.rs", "// code\n"),
            (".gitignore", "target/\n"),
        ],
    );
    let outside_zban = || snapshot(root, &[".zban"]);
    let before = outside_zban();

    ok(root, &["init", "--prefix", "np"]);
    let first = id(&ok(root, &["create", "First", "--body", "- [ ] step"]));
    let second = id(&ok(
        root,
        &["create", "Second", "-t", "bug", "--blocked-by", &first],
    ));
    ok(root, &["list"]);
    ok(root, &["list", "--ready", "--all", "--search", "first"]);
    ok(root, &["show", &first, &second]);
    ok(
        root,
        &[
            "update",
            &first,
            "-s",
            "in-progress",
            "--add-related",
            &second,
        ],
    );
    // Close, which archives, then reopen and close again.
    ok(
        root,
        &[
            "update",
            &first,
            "--replace-old",
            "- [ ] step",
            "--replace-new",
            "- [x] step",
            "--append",
            "## Summary of Changes",
            "-s",
            "done",
        ],
    );
    ok(root, &["update", &first, "-s", "todo"]);
    ok(root, &["update", &first, "-s", "cancelled"]);
    ok(
        root,
        &[
            "update",
            &second,
            "--remove-blocked-by",
            &first,
            "--remove-related",
            &first,
        ],
    );
    ok(root, &["delete", &second]);
    // Check, then repair a closed issue moved back out of the archive by hand.
    let clean = serde_json::json!({"findings": []});
    assert_eq!(ok(root, &["check"]), clean);
    let archived = root.join(format!(".zban/archive/{first}.md"));
    fs::rename(&archived, root.join(format!(".zban/{first}.md"))).unwrap();
    assert_eq!(ok(root, &["check", "--fix"]), clean);
    let output = zban(root, &["list", "--all"]);
    assert!(output.status.success());
    assert!(stdout(&output).contains("First"));

    assert_eq!(outside_zban(), before);
    assert!(!root.join(".git").exists());
    assert!(archived.is_file());
}

/// The inspection of `VER-NO-GIT`, automated: no source file starts a process.
// @mara implements VER-NO-GIT
#[test]
fn zban_starts_no_external_processes() {
    fn sources(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                sources(&path, files);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }
    let mut files = Vec::new();
    sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    assert!(files.len() > 5, "{files:?}");
    for file in files {
        let text = fs::read_to_string(&file).unwrap();
        for pattern in [
            "process::Command",
            "Command::new",
            "process::Stdio",
            "exec(",
        ] {
            assert!(!text.contains(pattern), "{} uses {pattern}", file.display());
        }
    }
}
