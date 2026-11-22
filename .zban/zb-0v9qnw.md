---
title: Read issue files and temporary files under one lock in zban check
status: todo
type: bug
priority: low
tags:
  - audit
created_at: 2026-11-22T14:28:49Z
---

Found by the architecture audit after milestone zb-v90b0k, at e1c9354, by reading the code. The behaviour is the same in 0.1.0.

`Project::check` calls `project.load()` and then `project.temporary_files()` (`src/check.rs`, about lines 61–62; `src/store.rs`, about lines 198–231). Each takes the shared directory lock and walks `.zban/` and `.zban/archive/` on its own, and the lock is released in between. A write by another zban process can land between the two, so `check` can judge issue files from before the write together with temporary files from after it, and every `check` walks the directories twice.

Relevant items: `ADR-WRITE-LOCK`, `DES-CHECK`.

Suggested fix from the audit: one function in `src/store.rs` that takes the shared lock once, walks the entries once and returns the issue files and the temporary files together; `Project::load` uses it and drops the temporary paths, and `temporary_files` goes away.

## Tasks

- [ ] Read the issue files and the temporary files in one pass under one shared lock
- [ ] Keep `VER-CHECK` and the simultaneous-changes tests passing, and add a test only if one can show the single pass without timing tricks

## Done when

- [ ] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [ ] `zban check` reports no errors
