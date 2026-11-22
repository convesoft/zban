---
title: Disable Claude auto memory for this repository
status: in-progress
type: task
created_at: 2026-11-22T12:55:50Z
---

Commit a `.claude/settings.json` containing only `"autoMemoryEnabled": false`.

Agents working in Git worktrees of this repository load the main checkout's Claude auto memory, so memory written in one session would silently steer every other agent. Durable knowledge belongs in `AGENTS.md` and the Mara docs instead. The auto memory is empty today, so nothing needs migrating.

## Done when

- [ ] `.claude/settings.json` holds exactly `{"autoMemoryEnabled": false}` and is valid JSON
- [ ] The file is tracked by Git
- [ ] fmt, clippy, tests, Mara validation and `zban check` pass
