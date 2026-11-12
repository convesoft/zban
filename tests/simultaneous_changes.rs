//! `VER-ISSUE-FILES`: simultaneous changes to one issue, through the real
//! binary.

mod common;

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

use common::{stdout, temp_dir, zban};
use serde_json::Value;

/// Processes that update the issue at the same time.
const WORKERS: usize = 8;
/// Successful updates each worker makes.
const UPDATES: usize = 25;
/// Attempts after which one update counts as starved, which only a broken
/// conflict check can cause.
const MAX_ATTEMPTS: usize = 1000;
/// How long a write must keep waiting while the lock is held. An unlocked
/// write finishes in a few milliseconds.
const LOCK_WAIT: Duration = Duration::from_millis(500);

/// Runs `zban --json` in `dir` and returns its document, or the error code
/// when it fails.
fn run(dir: &Path, args: &[&str]) -> Result<Value, String> {
    let output = zban(dir, &[args, &["--json"]].concat());
    let document: Value = serde_json::from_str(&stdout(&output)).unwrap();
    if output.status.success() {
        Ok(document)
    } else {
        Err(document["error"]["code"].as_str().unwrap().to_owned())
    }
}

// @mara implements VER-ISSUE-FILES
#[test]
fn parallel_updates_of_one_issue_lose_none() {
    let root = temp_dir("parallel-updates");
    run(&root, &["init"]).unwrap();
    let created = run(&root, &["create", "Shared"]).unwrap();
    let id = created["id"].as_str().unwrap();

    let appended: Vec<String> = thread::scope(|scope| {
        let workers: Vec<_> = (0..WORKERS)
            .map(|worker| {
                let root = &root;
                scope.spawn(move || {
                    let mut lines = Vec::new();
                    for update in 0..UPDATES {
                        let line = format!("[worker {worker} update {update}]");
                        let mut attempts = 0;
                        // A conflict means another update landed first; retry
                        // as the agent guide says. On btrfs, a listing taken
                        // during another update can miss the issue (zb-906f27).
                        while let Err(code) = run(root, &["update", id, "--append", &line]) {
                            assert!(
                                ["conflict", "not_found"].contains(&code.as_str()),
                                "{line}: {code}"
                            );
                            attempts += 1;
                            assert!(attempts < MAX_ATTEMPTS, "{line} starved");
                        }
                        lines.push(line);
                    }
                    lines
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|worker| worker.join().unwrap())
            .collect()
    });

    let shown = run(&root, &["show", id]).unwrap();
    let body = shown["issues"][0]["body"].as_str().unwrap();
    let lost: Vec<_> = appended
        .iter()
        .filter(|line| body.matches(line.as_str()).count() != 1)
        .collect();
    assert!(
        lost.is_empty(),
        "{} of {} updates that succeeded are missing: {lost:?}",
        lost.len(),
        appended.len()
    );
    for entry in fs::read_dir(root.join(".zban")).unwrap() {
        let name = entry.unwrap().file_name();
        assert!(!name.to_string_lossy().ends_with(".tmp"), "{name:?}");
    }
}

/// A new project in a directory private to this call, with one issue made by
/// `zban create` with `create_args`. Returns the directory and the issue's ID.
fn project_with(name: &str, create_args: &[&str]) -> (PathBuf, String) {
    let root = temp_dir(name);
    run(&root, &["init"]).unwrap();
    let created = run(&root, &[&["create"], create_args].concat()).unwrap();
    (root, created["id"].as_str().unwrap().to_owned())
}

/// Takes the lock on `.zban/` in `root`, as a zban write does, then starts
/// `zban args` there. Returns the command line, the lock and the command.
fn start_while_locked(root: &Path, args: &[&str]) -> (String, File, Child) {
    let lock = File::open(root.join(".zban")).unwrap();
    lock.lock().unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_zban"))
        .current_dir(root)
        .args(args)
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    (format!("zban {}", args.join(" ")), lock, child)
}

// @mara implements VER-ISSUE-FILES
#[test]
fn replacing_moving_and_deleting_wait_for_the_lock() {
    let (updating, updated) = project_with("lock-update", &["Updated"]);
    let (deleting, deleted) = project_with("lock-delete", &["Deleted"]);
    // A closed issue left in `.zban/`, as an interrupted close leaves it.
    let (moving, closed) = project_with("lock-move", &["Closed", "--status", "done"]);
    let name = format!("{closed}.md");
    let archived = moving.join(".zban/archive").join(&name);
    fs::rename(&archived, moving.join(".zban").join(&name)).unwrap();

    // One project each, because `check --fix` deletes the temporary file of
    // a write that waits in the same project.
    let mut running = [
        start_while_locked(&updating, &["update", &updated, "--append", "Waited"]),
        start_while_locked(&deleting, &["delete", &deleted]),
        start_while_locked(&moving, &["check", "--fix"]),
    ];
    thread::sleep(LOCK_WAIT);
    for (command, _, child) in &mut running {
        let status = child.try_wait().unwrap();
        assert_eq!(status, None, "`{command}` finished while locked");
    }
    for (command, lock, mut child) in running {
        drop(lock);
        assert!(child.wait().unwrap().success(), "`{command}` failed");
    }

    let shown = run(&updating, &["show", &updated]).unwrap();
    let body = shown["issues"][0]["body"].as_str().unwrap();
    assert!(body.contains("Waited"), "{body}");
    assert_eq!(run(&deleting, &["show", &deleted]), Err("not_found".into()));
    assert!(archived.exists());
}
