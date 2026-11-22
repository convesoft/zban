---
title: Rename MCP tag parameter to tags on create_issue and list_issues
status: in-progress
type: task
tags:
  - mcp
parent: zb-v90b0k
created_at: 2026-11-22T10:56:29Z
---

Issue frontmatter and JSON output name the field `tags`, while MCP `create_issue` and `list_issues` currently accept `tag` (mirroring repeatable CLI `--tag`). Across T3 Code sessions, agents repeatedly guessed `tags` on `create_issue` and hit `unknown field 'tags'`.

Make a clean breaking rename in MCP (no alias): rename `tag` to `tags` on `create_issue` and `list_issues` (keeping CLI `--tag` singular for repeatable flags).

## Tasks

- [ ] Update `DES-MCP-TOOLS`, `docs/mcp.mara.md`, `VER-SURFACE-PARITY` and `VER-AGENT-GUIDE` (if the guide/parity test checks parameter names) to document `tags` in MCP
- [ ] Rename `tag` to `tags` in `CreateArgs` and `ListArgs` in `src/mcp.rs` without adding a `tag` alias
- [ ] Update MCP and guide parity tests (`VER-SURFACE-PARITY`, `VER-AGENT-GUIDE`)

## Done when

- [ ] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [ ] `zban check` reports no errors
