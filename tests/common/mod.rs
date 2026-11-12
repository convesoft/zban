//! Helpers shared by the integration tests.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::Mutex;

/// An empty directory under Cargo's temporary directory, private to this
/// call.
pub fn temp_dir(name: &str) -> PathBuf {
    new_dir(Path::new(env!("CARGO_TARGET_TMPDIR")).join(unique(name)))
}

/// An empty directory outside any project, under the system's temporary
/// directory, private to this call. Cargo's temporary directory lies inside
/// this repository, whose own `.zban/` would be found from there.
pub fn temp_dir_outside_projects(name: &str) -> PathBuf {
    let name = format!("zban-test-{}-{}", unique(name), std::process::id());
    new_dir(std::env::temp_dir().join(name))
}

/// `name` with a number that differs on each call with the same name. Tests
/// run in parallel, and a directory that two of them share can be deleted
/// while one is still using it.
fn unique(name: &str) -> String {
    static CALLS: Mutex<BTreeMap<String, usize>> = Mutex::new(BTreeMap::new());
    let mut calls = CALLS.lock().unwrap();
    let call = calls.entry(name.to_owned()).or_default();
    *call += 1;
    format!("{name}-{call}")
}

/// Empties or creates `dir`.
fn new_dir(dir: PathBuf) -> PathBuf {
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Copies this repository's `.zban/` into `root`.
pub fn copy_repository_backlog(root: &Path) {
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join(".zban"),
        &root.join(".zban"),
    );
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// Writes `files`, given as paths relative to `root` and their content.
pub fn write_files(root: &Path, files: &[(&str, &str)]) {
    for (path, content) in files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
}

/// Runs the real `zban` binary in `dir`.
pub fn zban(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zban"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("zban runs")
}

/// Runs the real `zban` binary in `dir` with `input` on standard input.
pub fn zban_with_input(dir: &Path, args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zban"))
        .current_dir(dir)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("zban runs");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

/// Every file under `dir` with its content, by path relative to `dir`,
/// leaving out the directories named in `skip`.
pub fn snapshot(dir: &Path, skip: &[&str]) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(base: &Path, dir: &Path, skip: &[&str], files: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let relative = path.strip_prefix(base).unwrap().to_path_buf();
            if entry.file_type().unwrap().is_dir() {
                if !skip.iter().any(|skipped| relative == Path::new(skipped)) {
                    files.push((relative, Vec::new()));
                    walk(base, &path, skip, files);
                }
            } else {
                files.push((relative, fs::read(&path).unwrap()));
            }
        }
    }
    let mut files = Vec::new();
    walk(dir, dir, skip, &mut files);
    files.sort();
    files
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

pub mod mcp;
