---
title: Test atomic replacement, conflict through update and every error code
status: done
type: task
priority: high
parent: zb-9jra34
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
closed_at: 2026-11-12T13:30:53Z
---

Found by the v0.1.0 review: the interrupted-write test never calls the real replace path, `conflict` is only asserted on the store and never through `update` or the binary, and the error-code test uses a hand-written list that misses `conflict` and `invalid_handoff` (`VER-ISSUE-FILES`, `VER-CLI`).

- [x] A test fails if replacing an issue stops being an atomic rename (for example, the file's inode changes across an update)
- [x] `conflict` is produced through `update`
- [x] The every-error-code test derives its list from `ErrorCode`, so a new code without a test fails

## Summary of Changes

- `ErrorCode` is declared through a small `error_codes!` macro in `src/error.rs` that also defines `ErrorCode::ALL`, so the list cannot miss a code. JSON names and docs are unchanged.
- `tests/cli.rs` `each_error_code_is_produced` (`VER-CLI`) now checks every code in `ErrorCode::ALL`. It gains an `invalid_handoff` case and gets `conflict` from racing updates through the binary.
- `tests/common/mod.rs` `racing_updates` runs 8 `zban --json update ID --append -` processes at once, released together by closing their standard input, in rounds until one fails with `conflict`.
- `tests/issue_files.rs` (`VER-ISSUE-FILES`):
  - `update_fails_with_conflict_when_another_run_replaced_the_file`: a run that fails with `conflict` exits 1 and writes nothing, and the file holds a successful run's change.
  - `update_renames_a_new_file_over_the_issue`: a hard link to the issue file keeps its old bytes across `zban update` (title, append, close into the archive), so the file is never rewritten in place.
- Deliberate regressions, each restored afterwards:
  - Writing the issue in place with `fs::write` instead of the rename fails the hard-link test.
  - Dropping the unchanged-bytes check fails the conflict test ("no run failed with conflict in 50 rounds").
  - Reporting `conflict` only after the rename fails it on the conflicted run's line.
  - Adding an `ErrorCode` without a case, or removing the `invalid_handoff` case, fails `each_error_code_is_produced` ("no case produces ...").
- Discovered zb-906f27: on btrfs, a directory listing taken during another process's rename can leave out the issue being replaced, so a concurrent update can fail with `not_found`. The racing tests look only for `conflict` and ignore other failed runs.

Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all pass), `mara schema validate`, and `mara project validate` (`valid: true`, evaluation complete). The racing tests passed 25 of 25 runs each on one CPU, on two CPUs and with all CPUs.
