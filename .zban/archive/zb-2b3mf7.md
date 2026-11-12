---
title: Relations and readiness
status: done
type: feature
priority: high
parent: zb-9jra34
blocked_by:
  - zb-n938cw
created_at: 2026-11-11T21:00:03Z
closed_at: 2026-11-12T09:06:49Z
---

Implements `REQ-RELATIONS` and `DES-READINESS`, with `petgraph` per `ADR-DEPENDENCIES`.

## Done when

- [x] `VER-RELATIONS-READY` passes

## Summary of Changes

- `src/relations.rs`: `Graph` over one `Project::load`. It resolves IDs (`not_found`, `ambiguous_id`, `unreadable`), lists issues in listing order, computes `ready` and the `not_ready` reasons, the reverse relations (`children`, `blocks`, `related_from`, `discovered`) and the referrers of an issue, checks the relations a create or update adds (`check`), and adds or removes a `related` link on whichever issue stores it. Cycle checks use `petgraph` (`DiGraphMap`, and `astar` for the cycle path in the error message), with only its `std` and `graphmap` features.
- `Project::create` checks relations before writing. The new `Project::delete` refuses with `referenced`, naming the referrers, and shares the `conflict` check of `Project::replace`; `Project::place` builds its `conflict` error with the same helper. `Graph::resolve` and `Project::save` share one `unreadable` error, which names the file's first problem.
- New error codes from `DES-CLI`: `not_found`, `invalid_reference`, `cycle` and `referenced`, beside `ambiguous_id` and `unreadable` from zb-ramrc1, in the order `DES-CLI` lists them.
- The open-children check of `Project::save` stays separate from `Graph::reverse`: it counts both files of an ambiguous ID, so a close is refused conservatively, while `reverse` lists unique readable issues for display.
- Choices within `DES-READINESS` and `REQ-RELATIONS`:
  - Reasons come grouped by kind in the order the design lists them, blockers in `blocked_by` order and ancestors nearest first. A closed ancestor that is also blocked gives both `ancestor_closed` and `ancestor_blocked`, and the walk continues above it.
  - An ID with a file in both directories counts as unknown for readiness and is left out of listings. A new relation naming it fails with `ambiguous_id`.
  - `check` validates only the references and edges a request adds (`REQ-DAMAGED-DATA`), so dangling references and cycles that arrive by merge do not block unrelated changes. A new parent is also checked against the blockers of every descendant.
  - Removing a `related` link that both issues store removes it from both.
  - Deleting an unreadable issue fails with `unreadable`.
- Tests: `tests/relations.rs` checks `VER-RELATIONS-READY` through the library, with the fixture the item describes plus a closed parent, an unknown blocker and an in-progress issue. Updates are checked with `Graph::check`, and related-link edits with `add_related` and `remove_related` followed by `Project::save`, because the `update`, `list` and `show` operations come with zb-gnt4g4, which builds them on `Graph` and covers those surfaces in `VER-CLI`. Tests write issues with `Project::save`, so closed issues move to `.zban/archive/`.

Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (76 passed: 29 unit, 3 `cli`, 5 `ids`, 11 `issue_files`, 12 `lifecycle`, 6 `project_layout`, 10 `relations`), `mara schema validate` and `mara project validate` (`valid: true`, evaluation complete, no diagnostics).
