---
title: Give every test its own temporary project with tempfile
status: todo
type: task
priority: low
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
---

Found by the v0.1.0 review: the per-name counter added in zb-rvab3w is per process, so a process-per-test runner such as cargo-nextest would bring back the shared-directory race (`VER-AGENT-GUIDE`). The directories also live inside this repository, so a test that skips `init` would find the real `.zban/`.

Maintainer decision: use the `tempfile` crate as a dev-dependency. Each test gets a `TempDir` with a random name under the system temporary directory, removed when the test ends. Record the dev-dependency in `ADR-DEPENDENCIES` first.

- [ ] `ADR-DEPENDENCIES` names `tempfile` as a test-only dependency
- [ ] `temp_dir` and `temp_dir_outside_projects` in `tests/common/mod.rs` are replaced by `TempDir`, kept alive for as long as a test's zban process runs; the per-name counter goes
- [ ] Paths compared with zban's absolute JSON paths use the canonical temp path (on macOS `/tmp` is a symlink to `/private/tmp`)
- [ ] `cargo test` passes repeatedly, and under `cargo nextest run` if it is available
