---
title: Report direct-child progress counts on issues (child_counts)
status: done
type: feature
tags:
  - cli
  - mcp
parent: zb-v90b0k
created_at: 2026-11-22T10:56:28Z
closed_at: 2026-11-22T13:17:19Z
---

Progress questions were the single most frequent maintainer prompt in orchestrated zban threads, leading projects to write external scripts over `zban list --all --json` just to count closed vs. total children of epics.

Expose direct-child progress counts on issues alongside `checklist`:

- Always include `child_counts: {closed, total}` on `IssueView` in JSON (for `list`, `show` and mutations), counting direct children only — matching `--parent` and `open_children` — and avoiding collision with `show`'s `children` ID array.
- Show child progress in human output (`list` and `show`) when `total > 0`.

## Tasks

- [x] Update `DES-CLI` (and `VER-CLI` / `VER-SURFACE-PARITY`) to specify `child_counts: {closed, total}` for direct children in JSON and human output
- [x] Compute and include `child_counts` on `IssueView` in core operations and render it in CLI human output when `total > 0`
- [x] Add tests covering leaf issues (`{closed: 0, total: 0}`), parents with open and closed direct children, nested epics (direct children only), and CLI/MCP parity

## Done when

- [x] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [x] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [x] `zban check` reports no errors

## Summary of Changes

- Every issue view in JSON now carries `child_counts: {closed, total}` for its direct children, on `list`, `show` and mutation results, through the CLI and MCP alike: `Graph` counts the children once per load (`ChildCounts`, `Graph::child_counts` in `src/relations.rs`) and `IssueView` reports them (`src/operations.rs`).
- Human output: `list` appends `children closed/total closed` to the line of an issue with children, and `show` adds a `child_counts: closed/total closed` line; nothing is shown for an issue without children (`src/cli/human.rs`).
- `DES-CLI`, `VER-CLI` and `VER-SURFACE-PARITY` describe the field, the human output and the checks.
- Tests: `child_counts_count_direct_children_only` in `tests/cli.rs` (leaf `{0, 0}`, a parent with open, closed and archived children, a grandchild not counted, mutation results, human output); the draft-ancestor parity test in `tests/mcp.rs` compares `show` of a parent with a nested child on both surfaces and asserts the direct count; `tests/lifecycle_cli.rs` now checks the checklist segment of a listing line rather than its end, since an issue with children has child progress after it.

- Review round: `DES-CLI` now names `list --all --parent`, says that `done` and `cancelled` children count as closed whether archived or not, gives the order of the human text on a `list` line, and states the ambiguous-child limit; `VER-CLI` names the new cases. `child_counts_count_direct_children_only` adds a `cancelled` child, an epic under an epic and a reparented child; the new test `an_ambiguous_child_is_left_out_of_child_counts_but_blocks_closing` in `tests/cli.rs` pins the duplicate-ID case; the parity test in `tests/mcp.rs` compares a parent with a closed child on both surfaces and is linked to `VER-SURFACE-PARITY`. The self-parented edge case is filed as zb-q9hg97.

Checks run (again after the review round and merging `origin/main`): `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all pass); `mara schema validate` (valid) and `mara project validate --format json` (`valid: true`, `evaluation_complete: true`); `zban check` with the branch binary (`No problems found.`).

## Decisions made

- The counted children are exactly the issues of `show`'s `children`: readable issues with a unique ID whose `parent` is this issue. Unreadable or duplicated children are left out, as they are of `children`, and are reported as warnings. Recorded in `DES-CLI`.
- A child counts as closed when its status is `done` or `cancelled`, archived or not. Recorded in `DES-CLI` and `VER-CLI`.
- Human format: `list` puts `children closed/total closed` after the checklist `[checked/total]` and before the handoff age; `show` puts `child_counts: closed/total closed` after the `checklist` line, named after the JSON field because `children` already lists the IDs. Recorded in `DES-CLI`.
- The agent guide is unchanged: no Mara item requires it to mention `child_counts`.
- A child whose file is unreadable or whose ID names two files is left out of `children` and `child_counts` and reported as a warning, while the close guard stays conservative and still refuses with `open_children` when a readable copy of it is open (`REQ-DAMAGED-DATA`, `DES-CLI`). Decided by the maintainer in review; recorded in `DES-CLI` and pinned by a test.
