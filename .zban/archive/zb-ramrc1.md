---
title: Close checklists and auto-archive
status: done
type: feature
priority: high
parent: zb-9jra34
blocked_by:
  - zb-n938cw
created_at: 2026-11-11T21:00:04Z
closed_at: 2026-11-12T09:05:10Z
---

Implements `DES-LIFECYCLE`.

## Done when

- [x] `VER-LIFECYCLE` passes

## Summary of Changes

- New core module `lifecycle`: `Checklist` (`{checked, total}`, which listings will report) and `Issue::checklist`, plus `Project::save(files, file, issue, force)`, which writes the issue as a request leaves it. Without force it refuses a change to `done` with unchecked task items (`unchecked_items`) and a change to `done` or `cancelled` with open children (`open_children`, naming them sorted by ID), and it writes nothing then. It sets and clears `closed_at`, rewrites the file in place and, after a change between open and closed, renames it into the other directory. It never touches another issue.
- `Project::place` in the store renames a readable file into the directory its status requires. It refuses with `ambiguous_id` when the target name is taken, and `save` checks that before writing. `zban check --fix` (zb-yj8xta) can reuse it.
- A crate-private `markdown` module finds lines outside fenced code blocks and classifies task items. zb-qkj8tj can reuse the fence handling for handoff headings.
- New error codes from `DES-CLI`: `ambiguous_id`, `unreadable`, `unchecked_items`, `open_children`.
- Reading choices within `DES-LIFECYCLE`: the closing rules apply only when the status changes. `closed_at` is set when an open status becomes closed, removed when it becomes open, and otherwise kept, so `done` to `cancelled` keeps it. The file moves only after a change between open and closed, into the directory the new status requires, so a misplaced file not crossing stays for `check --fix`. Code fences are three or more backticks or tildes at any indentation, closed as in CommonMark, and an unclosed fence runs to the end of the body.
- Tests: `tests/lifecycle.rs` (12 tests for `VER-LIFECYCLE`) and unit tests for fences, task items and checklist counting. The `VER-LIFECYCLE` clauses about `show`, search, relations, `list --all`, the default listing and checklist progress are checked through the library: archived issues load with their relations and checklist, and a closed child no longer holds its parent open. Those surfaces arrive with zb-gnt4g4 and zb-2b3mf7; filed zb-vqradd to repeat the clauses through the binary.

Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (66 passed: 29 unit, 3 `cli`, 5 `ids`, 11 `issue_files`, 12 `lifecycle`, 6 `project_layout`), `mara schema validate` and `mara project validate` (`valid: true`, evaluation complete).
