---
title: Check archive visibility and checklist progress through the binary
status: done
type: task
parent: zb-9jra34
blocked_by:
  - zb-gnt4g4
discovered_from: zb-ramrc1
created_at: 2026-11-12T09:05:10Z
closed_at: 2026-11-12T10:02:18Z
---

zb-ramrc1 checked `VER-LIFECYCLE` through the core library, because `list`, `show`
and their search and relation output did not exist yet. Repeat its clauses about
`show`, search, relations, `list --all`, the default listing and checklist progress
through the real `zban` binary, as `AGENTS.md` prefers.

## Done when

- [x] Those `VER-LIFECYCLE` clauses pass through the real binary

## Summary of Changes

- New `tests/lifecycle_cli.rs` (3 tests for `VER-LIFECYCLE` through the real `zban` binary, in a new file so `tests/cli.rs` stays untouched). The issues are closed and reopened with `zban update`, not written into `.zban/archive/` by hand.
- Closing: `done` with an unchecked item fails with `unchecked_items` while a fenced example does not count, `cancelled` with an open child fails with `open_children` naming it, and both change nothing. A forced close sets `closed_at`, moves the file to `.zban/archive/` and leaves the child unchanged, and reopening restores the file byte for byte. With the child closed, one update checks the last item, appends a summary and closes without force.
- Archived issues: absent from the default listing and from search without `--all`, present with `--all` or a closed `--status`, with their archive path. `show` returns them alone or among open issues in the order requested, with body, checklist and reverse relations, in JSON and human output. Open issues still read relations to them: a closed blocker no longer blocks, a closed parent gives `ancestor_closed`, and `children` and `related_from` list them. New relations to an archived issue can be added, removing a `related` link stored on it rewrites it in place in the archive, and deleting it fails with `referenced`.
- Checklist progress: `list` reports `{checked, total}` in JSON and `[checked/total]` in human output, `{0, 0}` and no brackets without items, and follows body edits, also for archived issues in `list --all`.
- No product bug showed up. A mutation check (the default listing no longer leaving out closed issues) made the archive test fail as expected.

Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (107 passed: 32 unit, 8 `check`, 18 `cli`, 5 `ids`, 11 `issue_files`, 12 `lifecycle`, 3 `lifecycle_cli`, 2 `no_git`, 6 `project_layout`, 10 `relations`), `mara schema validate` and `mara project validate` (`valid: true`, evaluation complete, 0 errors, 0 warnings).
