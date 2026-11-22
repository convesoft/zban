---
title: Add a branch or worktree hint to not_found error messages
status: in-progress
type: feature
tags:
  - cli
  - mcp
parent: zb-v90b0k
created_at: 2026-11-22T10:56:30Z
---

In parallel-worktree workflows, agents occasionally look up an issue ID that was created on another branch or worktree and get a bare `not_found` error (6 observed cases across T3 Code threads).

Keep the `not_found` error code unchanged (`DES-CLI`) and add a brief hint to the human-readable error message noting that the issue may exist on another branch or worktree.

## Tasks

- [ ] Check `DES-CLI` and `VER-CLI` for any wording updates needed
- [ ] Add the branch/worktree hint to `not_found` error messages in core operations
- [ ] Update or add tests verifying the `not_found` error code and hint message through CLI and MCP

## Done when

- [ ] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [ ] `zban check` reports no errors
