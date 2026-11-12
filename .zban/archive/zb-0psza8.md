---
title: Close the lost-update window between concurrent updates
status: done
type: bug
priority: high
parent: zb-9jra34
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
closed_at: 2026-11-12T13:31:28Z
---

Found by the v0.1.0 review: two `zban update` runs on one issue at the same moment can both pass the unchanged-bytes comparison, and the later rename drops the earlier change while both exit 0 (31 of 300 rounds on tmpfs, 13 of 200 on btrfs). This breaks `REQ-NO-LOST-UPDATES`; see `DES-SHARED-CORE` and `VER-ISSUE-FILES`.

Maintainer decision: hold a short OS advisory lock on the `.zban/` directory only across the compare-and-rename (and compare-and-delete) step, using std `File::lock` with no new dependency and no lock file. Record it in Mara first, replacing the current "no locks" wording, and set `rust-version` in `Cargo.toml` to the minimum the code then needs.

- [x] Mara updated: `REQ-NO-LOST-UPDATES`, `DES-SHARED-CORE`, the user docs on simultaneous changes, and `VER-ISSUE-FILES`
- [x] Lock taken around every compare-then-write in the store
- [x] `rust-version` set
- [x] A test runs many parallel updates on one issue and loses none

## Summary of Changes

- Reproduced first: `parallel_updates_of_one_issue_lose_none` in `tests/simultaneous_changes.rs` runs 8 processes of the real binary, each appending 25 times to one issue and retrying after `conflict`. Without the lock it lost 34 to 64 of the 200 successful updates on btrfs (5 of 5 runs) and 11 to 31 on tmpfs (5 of 5 runs).
- Mara: the new `ADR-WRITE-LOCK` records the maintainer's decision and the rejected lock file. `REQ-NO-LOST-UPDATES` drops "zban takes no locks" and covers simultaneous zban processes. `DES-SHARED-CORE` describes the lock and the instant that editors and Git can still hit. `DES-CHECK` no longer says that zban takes no locks. The user docs on simultaneous changes explain the lock, and `VER-ISSUE-FILES` names both new tests.
- `src/store.rs`: `Project::locked` holds `File::lock` on a handle of `.zban/` across the comparison and the rename in `replace`, the comparison and the removal in `delete_among`, and the target check and the rename in `place`. No lock file and no new dependency.
- `Cargo.toml` sets `rust-version = "1.89"`: `File::lock` and the locked `uuid` 1.29.2 both need 1.89. The README's source install says Rust 1.89 or later.
- `replacing_moving_and_deleting_wait_for_the_lock` holds the lock while `update`, `delete` and `check --fix` run, and they must not finish until it is released. Removing the lock from any one of the three call sites makes it fail.
- Found: on btrfs, a listing taken while another process renames over an issue file misses that file, so a command can fail with `not_found`. Filed as zb-7mbhpw; the parallel test retries `not_found` until it is fixed.
- Checks run: `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` on Rust 1.99.1, all passing; `cargo +1.89 test --locked`, all passing; the parallel test with the lock passed 10 of 10 runs on btrfs and 5 of 5 on tmpfs; `mara schema validate` and `mara project validate` report valid with complete evaluation. On macOS the lock is `flock`, as on Linux; the pull request's macOS CI job runs both tests there.
