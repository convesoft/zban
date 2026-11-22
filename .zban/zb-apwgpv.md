---
title: Share the integration test helpers in tests/common
status: todo
type: task
priority: deferred
tags:
  - deferred
  - backlog
created_at: 2026-11-22T14:29:23Z
---

Found by the architecture audit after milestone zb-v90b0k, at e1c9354. Structural debt, and the first step before the test files are split.

The test binaries repeat their scaffolding: `CONFIG` is in ten files; the JSON runners and readers (`run_json`, `ok`, `error_code`, `ids`, `listed`, `shown`) are written again in six; the hand-written issue builders (`issue` or `file_text`, `open`, `archived`, `project`) in seven; and `tests/mcp.rs` has a preamble of about 165 lines (`Surfaces`, `fixture`, `unify_times`) next to `tests/common/mcp.rs`. See `tests/common/mod.rs`, `tests/common/mcp.rs`, and the heads of `tests/cli.rs`, `tests/mcp.rs`, `tests/lifecycle_cli.rs`, `tests/handoff.rs`, `tests/check.rs`, `tests/no_git.rs`, `tests/issue_files.rs`, `tests/lifecycle.rs`, `tests/relations.rs` and `tests/prime.rs`.

## Tasks

- [ ] Move the shared CLI, JSON and fixture helpers into `tests/common/mod.rs`, and the shared CLI fixture into `tests/common/cli.rs`
- [ ] Move `Surfaces`, `fixture`, the fixture IDs, `unify_times`, `ids` and `error_code` from `tests/mcp.rs` into `tests/common/mcp.rs`
- [ ] Use them from the existing test files, changing no test's meaning

## Done when

- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass, with the same number of tests as before
- [ ] `zban check` reports no errors
