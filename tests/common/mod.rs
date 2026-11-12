//! Helpers shared by the integration tests.

#![allow(dead_code)]

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tempfile::TempDir;

/// A fresh empty directory with a random name, for one test only: the test
/// builds its fixture project in it. It lies under the system's temporary
/// directory, outside any project, and its path is canonical like the
/// absolute paths zban prints (on macOS the temporary directory is reached
/// through a symbolic link). Dropping it deletes the directory, so a test
/// creates it before it starts any `zban` process there, such as an MCP
/// client, which then ends first.
pub fn temp_dir() -> TempDir {
    let base = fs::canonicalize(std::env::temp_dir()).unwrap();
    tempfile::Builder::new()
        .prefix("zban-test-")
        .tempdir_in(base)
        .unwrap()
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

/// Updates issue `id` from several `zban` processes at once, each appending
/// its own line with `--json update ID --append -`, in rounds until a run
/// fails with `conflict` because another run replaced the file after its
/// read. Returns that round: each run's line and output. Each process waits
/// for its standard input to end, which starts a round's runs together. Every
/// run must succeed or fail with `conflict`.
pub fn racing_updates(dir: &Path, id: &str) -> Vec<(String, Output)> {
    const RUNS: usize = 8;
    const ROUNDS: usize = 50;
    for round in 0..ROUNDS {
        let mut children: Vec<_> = (0..RUNS)
            .map(|_| {
                Command::new(env!("CARGO_BIN_EXE_zban"))
                    .current_dir(dir)
                    .args(["--json", "update", id, "--append", "-"])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("zban runs")
            })
            .collect();
        let lines: Vec<_> = (0..RUNS)
            .map(|run| format!("Round {round}, run {run}."))
            .collect();
        let mut inputs: Vec<_> = children
            .iter_mut()
            .map(|child| child.stdin.take().unwrap())
            .collect();
        for (input, line) in inputs.iter_mut().zip(&lines) {
            input.write_all(line.as_bytes()).unwrap();
        }
        drop(inputs);
        let runs: Vec<_> = lines
            .into_iter()
            .zip(children)
            .map(|(line, child)| (line, child.wait_with_output().unwrap()))
            .collect();
        for (line, output) in &runs {
            let code = json_error_code(output);
            assert!(
                output.status.success() || code.as_deref() == Some("conflict"),
                "{line}: {code:?}"
            );
        }
        if runs
            .iter()
            .any(|(_, output)| json_error_code(output).as_deref() == Some("conflict"))
        {
            return runs;
        }
    }
    panic!("no run failed with conflict in {ROUNDS} rounds");
}

/// The code of the JSON error that `output` prints, if it prints one.
pub fn json_error_code(output: &Output) -> Option<String> {
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    value["error"]["code"].as_str().map(str::to_owned)
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

pub mod mcp;
