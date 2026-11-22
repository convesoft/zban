---
title: Add a branch or worktree hint to not_found error messages
status: done
type: feature
tags:
  - cli
  - mcp
parent: zb-v90b0k
created_at: 2026-11-22T10:56:30Z
closed_at: 2026-11-22T13:45:49Z
---

In parallel-worktree workflows, agents occasionally look up an issue ID that was created on another branch or worktree and get a bare `not_found` error (6 observed cases across T3 Code threads).

Keep the `not_found` error code unchanged (`DES-CLI`) and add a brief hint to the human-readable error message noting that the issue may exist on another branch or worktree.

## Tasks

- [x] Check `DES-CLI` and `VER-CLI` for any wording updates needed
- [x] Add the branch/worktree hint to `not_found` error messages in core operations
- [x] Update or add tests verifying the `not_found` error code and hint message through CLI and MCP

## Done when

- [x] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [x] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [x] `zban check` reports no errors

## Summary of Changes

- `Graph::resolve` (`src/relations.rs`), the one core path behind every lookup of a well-formed ID that names no issue, now says `issue <id> does not exist in this project; it may exist on another branch or worktree`. The error code stays `not_found`, and the CLI and the MCP server share the message (`DES-SHARED-CORE`). zban does not look at branches, worktrees or other checkouts.
- `DES-CLI` (the `not_found` row) and `VER-CLI` now describe the hint and which messages carry it.
- Tests: `tests/cli.rs` checks the hint in human and JSON output for an unknown ID and its absence for a malformed one; `tests/mcp.rs` checks that `show`, `update`, `delete` and `list --parent` with an unknown ID give the same `not_found` error and message on both surfaces, with the hint, and that a malformed ID gets no hint.
- Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all pass); `mara schema validate` (valid) and `mara project validate --format json` (`valid: true`, `evaluation_complete: true`); `zban check` (no problems).

## Decisions made

- Only an ID in this project's format, with its prefix, that names no issue gets the hint (recorded in `DES-CLI`); an ID with another project's prefix gets the ID-format message. A value that cannot be an ID keeps its own message showing the ID format, because it cannot exist on another branch either. Adding a relation that names an unknown issue fails with `invalid_reference`, not `not_found`, and stays unchanged.

## Review follow-up

- `tests/cli.rs` now checks that the message for a malformed ID, in human and JSON output, has no hint and does show the ID format.
- The `not_found` row of `DES-CLI` now says which values get the hint: IDs in this project's format, with its prefix. `VER-CLI` uses the same wording.
- The decision entry above now says that adding a relation that names an unknown issue fails with `invalid_reference`.
- Checks rerun: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `mara schema validate`, `mara project validate --format json` and `zban check`.
