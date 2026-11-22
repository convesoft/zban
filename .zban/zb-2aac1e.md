---
title: Fail update --remove-related when the link may be stored on a damaged issue
status: todo
type: bug
tags:
  - audit
created_at: 2026-11-22T14:28:49Z
---

Found by the architecture audit after milestone zb-v90b0k, at e1c9354, and reproduced with the built binary. The behaviour is the same in 0.1.0.

A `related` link is stored on one of the two issues (`DES-ISSUE-FILE`: removing a link edits whichever issue stores it). When B stores `related: [A]` and B's file is unreadable, for example with merge conflict markers, or B's ID names two files, `zban update A --remove-related B` finds no link on A, writes nothing and exits 0. The link stays on B. `Graph::remove_related` returns `None` for any target that is not readable (`src/relations.rs`, about lines 465–481; called from `src/operations.rs`, about lines 372–375).

`REQ-DAMAGED-DATA` says a mutation fails when it must read or write an unreadable file, and that an ID naming more than one file is reported as ambiguous rather than resolved by guessing.

Suggested fix from the audit: still remove the link when the issue being updated stores it, so that a link to a broken issue can be cleaned up; when it does not, fail with `unreadable` or `ambiguous_id` for such a target, and keep an unknown ID a no-op.

## Tasks

- [ ] Make `update --remove-related` fail with `unreadable` or `ambiguous_id` when the other issue may store the link and cannot be read or is ambiguous
- [ ] Update `DES-ISSUE-FILE` or `DES-CLI` if their wording needs it
- [ ] Cover both cases, and the cleanup of a link stored on the readable side, through the real binary and the MCP server

## Done when

- [ ] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [ ] `zban check` reports no errors
