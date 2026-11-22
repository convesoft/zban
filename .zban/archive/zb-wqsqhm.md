---
title: Disable Claude auto memory for this repository
status: done
type: task
created_at: 2026-11-22T12:55:50Z
closed_at: 2026-11-22T13:00:35Z
---

Commit a `.claude/settings.json` containing only `"autoMemoryEnabled": false`.

Agents working in Git worktrees of this repository load the main checkout's Claude auto memory, so memory written in one session would silently steer every other agent. Durable knowledge belongs in `AGENTS.md` and the Mara docs instead. The auto memory is empty today, so nothing needs migrating.

## Done when

- [x] `.claude/settings.json` holds exactly `{"autoMemoryEnabled": false}` and is valid JSON
- [x] The file is tracked by Git
- [x] fmt, clippy, tests, Mara validation and `zban check` pass

## Summary of Changes

Added `.claude/settings.json` with only `"autoMemoryEnabled": false`. Checked: valid JSON (`python3 -m json.tool`), tracked (`git ls-files`), `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `mara schema validate`, `mara project validate` and `zban check` all pass.
