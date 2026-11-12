---
title: Update how agents run zban here after the release
status: done
type: task
priority: low
parent: zb-9jra34
blocked_by:
  - zb-aapz52
discovered_from: zb-as3s58
created_at: 2026-11-12T10:14:08Z
closed_at: 2026-11-12T14:45:08Z
---

`AGENTS.md` tells agents to run zban from their checkout with `cargo run -q -- <command>`, because zban has no release yet. Once zb-aapz52 publishes one, that sentence is out of date.

- [x] Decide whether agents here use the released `zban` or keep running the checkout's build, which always reads this branch's issue format
- [x] Update `AGENTS.md` to match
- [x] Update the project status line in `AGENTS.md`, which still says implementation is in progress (found in the `VAL-SCOPE-REVIEW` of zb-aapz52)

## Summary of Changes

Maintainer decision (2026-11-12), a repository convention rather than product knowledge, so no Mara item changes: agents in this repository use the `zban` on `PATH`, installed from a GitHub release or with `cargo install --path . --locked` from this checkout, and the zban MCP server from the committed `.mcp.json`, which starts `zban mcp` from `PATH`; in a worktree they pass `directory` or start the session in that worktree. `cargo run -q -- <command>` is only for trying out an unreleased change to zban itself that the task depends on, such as a new option or a changed issue format, because the installed `zban` reads and writes the same issue format as `main`. The opening `zban prime` line stays as `REQ-AGENT-GUIDE` and `this_repository_loads_the_guide` require. No Mara item stated the old convention.

- `AGENTS.md` now states that convention in place of the "no release yet" paragraph, and its project status line says the v1 specification is implemented and released as v0.1.0.
- `README.md` already describes the released state, and no page in `docs/` describes this repository's release or implementation status, so neither changed.

Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all pass, including `this_repository_loads_the_guide` and `this_repository_backlog_is_clean`), `mara schema validate`, `mara project validate` (`valid: true`, evaluation complete, no errors or warnings) and `zban check` (no findings).
