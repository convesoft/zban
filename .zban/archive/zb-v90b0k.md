---
title: "zban 1.0.0-rc.1: CLI, readiness, MCP and docs improvements"
status: done
type: epic
priority: high
tags:
  - release-1-0-rc1
created_at: 2026-11-22T12:35:06Z
closed_at: 2026-11-22T14:18:44Z
---

Milestone epic grouping the post-0.1.0 improvements identified from real-world zban usage across T3 Code threads plus bundling docs into the npm package.

## Done when

- [x] All child issues are closed
- [x] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [x] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [x] `zban check` reports no errors

## Summary of Changes

All seven children are closed and merged to `main`, each through its own pull request with green CI and a review:

- zb-a76tqw (#49): `zban check` prints `No problems found.` when it finds nothing.
- zb-banzx5 (#53, breaking): a `draft` ancestor holds back its descendants, with the reason `ancestor_draft:<id>`, and the agent guide says to park planned epics, backlog and human-only work as `draft`.
- zb-q3z2gz (#50): the `@convesoft/zban` npm package carries the `docs/` folder.
- zb-y2kwfz (#54): every issue view carries `child_counts: {closed, total}` for its direct children.
- zb-wbm14r (#55, breaking): the MCP tools `create_issue` and `list_issues` take `tags` instead of `tag`.
- zb-35d4qp (#56): the `not_found` message for an unknown ID in the project's format hints at another branch or worktree.
- zb-esx9nv (#57): `zban list --not-tag T` and `not_tags` on `list_issues` leave out issues by tag.

The milestone was renamed from 0.2 to 1.0.0-rc.1 while it ran (#51). The decisions made on the maintainer's behalf are in zb-r2nnaz, and one question is open in zb-q6q2fz. Reviews filed five deferred backlog issues: zb-ym7w1t, zb-t2h8tb, zb-q9hg97, zb-32nknc and zb-rby6kf.

Checks run by the orchestrator on `main` at e1c9354, after the last child merged: `cargo fmt --all -- --check` (pass), `cargo clippy --all-targets -- -D warnings` (pass), `cargo test` (188 passed, 0 failed), `mara schema validate` (valid), `mara project validate --format json` (`valid: true`, `evaluation_complete: true`, no diagnostics) and `zban check` with the installed binary and the one built from `main` (no problems). CI on this pull request covers the same commands again.

Not done here, and left to the maintainer: the version bump to `1.0.0-rc.1`, the tag and the release.
