---
title: Give every test a fresh fixture project
status: done
type: task
priority: low
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
closed_at: 2026-11-12T15:32:20Z
---

Found by the v0.1.0 review: the per-name counter added in zb-rvab3w is per process, so a process-per-test runner such as cargo-nextest would bring back the shared-directory race (`VER-AGENT-GUIDE`). The directories also live inside this repository, so a test that skips `init` would find the real `.zban/`.

Maintainer decision: every test builds a fresh fixture, its own disposable project, through one shared helper; no fixture is shared between tests. Use the `tempfile` crate as a dev-dependency: each fixture is a `TempDir` with a random name under the system temporary directory, removed when the test ends. Record the dev-dependency in `ADR-DEPENDENCIES` first.

- [x] `ADR-DEPENDENCIES` names `tempfile` as a test-only dependency
- [x] `temp_dir` and `temp_dir_outside_projects` in `tests/common/mod.rs` are replaced by a helper that returns the `TempDir` itself, not a path, so the test owns it. Each fixture is deleted when its own test ends, after any zban process the test started has exited (create the fixture before an MCP client, so the client drops first). The per-name counter goes
- [x] Paths compared with zban's absolute JSON paths use the canonical temp path (on macOS `/tmp` is a symlink to `/private/tmp`)
- [x] `cargo test` passes repeatedly, and under `cargo nextest run` if it is available

## Summary of Changes

- `ADR-DEPENDENCIES` names `tempfile` as the tests' one dependency of their own, and `Cargo.toml` adds it under `[dev-dependencies]`.
- `tests/common/mod.rs`: `temp_dir()` returns a `tempfile::TempDir` with a random `zban-test-` name under the canonicalized system temporary directory. `temp_dir_outside_projects`, `unique` and the per-name counter are gone, and so are the explicit `fs::remove_dir_all` calls. The per-file helpers return the `TempDir`, alone or with the `Project` or the root path, and each test binds it before it starts a `zban` process. `Surfaces` in `tests/mcp.rs` holds its directories in its last field, so its client drops first. `tool_schemas()` in `tests/prime.rs` now binds its project before starting the client; before this change the directory was a temporary value. Tests that read this repository through `CARGO_MANIFEST_DIR` still do so explicitly.
- The `src/store.rs` unit tests use `tempfile::tempdir()` instead of a directory named after the process ID.

Checks run:

- `cargo fmt --all -- --check` and `cargo clippy --all-targets -- -D warnings`.
- `cargo test` 10 times in a row, 173 passed each time.
- Every test in its own process, 32 at a time, for 3 rounds: no failures.
- `cargo nextest run` (0.9.151, built into a scratch directory) 5 times, 173 passed each time.
- `mara schema validate` and `mara project validate`: valid, with evaluation complete.
- `zban check`: no problems.
- No new-style `zban-test-` directory was left behind.

The same nextest runs on `origin/main` also passed 5 times, so the old race did not reproduce in that sample. The fix holds because no two fixtures share a name.
