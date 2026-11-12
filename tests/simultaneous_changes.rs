//! `VER-ISSUE-FILES`: simultaneous changes to one issue, through the real
//! binary.

mod common;

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use common::{stdout, temp_dir, zban};
use serde_json::Value;

/// Processes that update the issue at the same time.
const WORKERS: usize = 8;
/// Successful updates each worker makes.
const UPDATES: usize = 25;
/// Attempts after which one update counts as starved, which only a broken
/// conflict check can cause.
const MAX_ATTEMPTS: usize = 1000;
/// How long a command must keep waiting while the lock is held. An unlocked
/// command finishes in a few milliseconds.
const LOCK_WAIT: Duration = Duration::from_millis(500);
/// How long a read may take while another process holds the lock shared.
const READ_LIMIT: Duration = Duration::from_secs(10);
/// Processes that read the project while its issue is replaced.
const READERS: usize = 8;
/// `zban show` and `zban list` runs each reader makes.
const READS: usize = 50;

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
                        // as the agent guide says. Any other failure is a bug.
                        while let Err(code) = run(root, &["update", id, "--append", &line]) {
                            assert_eq!(code, "conflict", "{line}");
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

/// Takes the lock on `.zban/` in `root` with `lock`: `File::lock` as a zban
/// write does, or `File::lock_shared` as a listing does. Then starts
/// `zban args` there. Returns the command line, the lock and the command.
fn start_while_locked(
    root: &Path,
    lock: fn(&File) -> io::Result<()>,
    args: &[&str],
) -> (String, File, Child) {
    let held = File::open(root.join(".zban")).unwrap();
    lock(&held).unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_zban"))
        .current_dir(root)
        .args(args)
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    (format!("zban {}", args.join(" ")), held, child)
}

/// Asserts that none of `running` finishes within [`LOCK_WAIT`], then
/// releases each lock and asserts that its command succeeds.
fn assert_wait(mut running: Vec<(String, File, Child)>) {
    thread::sleep(LOCK_WAIT);
    for (command, _, child) in &mut running {
        let status = child.try_wait().unwrap();
        assert_eq!(status, None, "`{command}` finished while locked");
    }
    for (command, held, mut child) in running {
        drop(held);
        assert!(child.wait().unwrap().success(), "`{command}` failed");
    }
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

    // A shared lock, as another process's listing holds, lets each
    // command's own listing through, so each waits in its write. One project
    // each, because `check --fix` deletes the temporary file of a write that
    // waits in the same project.
    let shared = File::lock_shared;
    assert_wait(vec![
        start_while_locked(
            &updating,
            shared,
            &["update", &updated, "--append", "Waited"],
        ),
        start_while_locked(&deleting, shared, &["delete", &deleted]),
        start_while_locked(&moving, shared, &["check", "--fix"]),
    ]);

    let shown = run(&updating, &["show", &updated]).unwrap();
    let body = shown["issues"][0]["body"].as_str().unwrap();
    assert!(body.contains("Waited"), "{body}");
    assert_eq!(run(&deleting, &["show", &deleted]), Err("not_found".into()));
    assert!(archived.exists());
}

// @mara implements VER-ISSUE-FILES
#[test]
fn reads_wait_for_a_write_but_not_for_another_read() {
    let (showing, shown) = project_with("lock-show", &["Shown"]);
    let (listing, _) = project_with("lock-list", &["Listed"]);
    // One project each, because two exclusive locks of this process on one
    // directory would wait for each other.
    assert_wait(vec![
        start_while_locked(&showing, File::lock, &["show", &shown]),
        start_while_locked(&listing, File::lock, &["list"]),
    ]);

    for (root, args) in [
        (&showing, &["show", shown.as_str()][..]),
        (&listing, &["list"]),
    ] {
        let started = Instant::now();
        let (command, _held, mut child) = start_while_locked(root, File::lock_shared, args);
        while child.try_wait().unwrap().is_none() {
            assert!(
                started.elapsed() < READ_LIMIT,
                "`{command}` waited for a shared lock"
            );
            thread::sleep(Duration::from_millis(10));
        }
        assert!(child.wait().unwrap().success(), "`{command}` failed");
    }
}

// @mara implements VER-ISSUE-FILES
#[test]
fn reads_see_an_issue_that_another_process_replaces() {
    let (root, id) = project_with("read-while-replaced", &["Replaced"]);
    let dir = root.join(".zban");
    let path = dir.join(format!("{id}.md"));
    let bytes = fs::read(&path).unwrap();
    let reading = AtomicBool::new(true);

    let (reads, replacements) = thread::scope(|scope| {
        // Renames a copy of the issue over it, again and again, under the
        // lock on `.zban/` as a zban write does.
        let replacer = scope.spawn(|| {
            let staged = dir.join(".zban-replacement.tmp");
            let mut replacements = 0_usize;
            while reading.load(Ordering::Relaxed) {
                fs::write(&staged, &bytes).unwrap();
                let lock = File::open(&dir).unwrap();
                lock.lock().unwrap();
                fs::rename(&staged, &path).unwrap();
                drop(lock);
                replacements += 1;
            }
            replacements
        });
        let readers: Vec<_> = (0..READERS)
            .map(|_| {
                scope.spawn(|| {
                    for _ in 0..READS {
                        let shown = run(&root, &["show", &id]);
                        assert!(shown.is_ok(), "show {id}: {shown:?}");
                        let listed = run(&root, &["list"]).unwrap();
                        let ids: Vec<_> = listed["issues"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|issue| issue["id"].as_str().unwrap())
                            .collect();
                        assert_eq!(ids, [id.as_str()], "list");
                    }
                })
            })
            .collect();
        let reads: Vec<_> = readers.into_iter().map(|reader| reader.join()).collect();
        // Stop the replacer before a failed reader ends the test.
        reading.store(false, Ordering::Relaxed);
        (reads, replacer.join().unwrap())
    });
    for read in reads {
        if let Err(panic) = read {
            std::panic::resume_unwind(panic);
        }
    }
    assert!(replacements > 0);
}
