---
title: Manage this backlog with zban
status: done
type: task
parent: zb-9jra34
blocked_by:
  - zb-gnt4g4
  - zb-yj8xta
  - zb-y67s20
created_at: 2026-11-11T21:00:09Z
closed_at: 2026-11-12T10:13:22Z
---

Switch this repository from hand-edited issue files to the zban CLI. This starts `VAL-DOGFOOD`.

- [x] `zban check` is clean on this repository
- [x] `AGENTS.md` loads the workflow with the `zban prime` line from `REQ-AGENT-GUIDE`
  and keeps only project-specific rules, replacing the hand-editing steps
- [x] A follow-up issue exists to evaluate `VAL-DOGFOOD` after ten issues have been worked on through zban

## Summary of Changes

- `AGENTS.md` now opens with the `zban prime` line that `docs/agents.mara.md` recommends (`REQ-AGENT-GUIDE`) and says to run zban from the checkout with `cargo run -q -- <command>` until there is a release. Four project-specific rules replace the hand-editing steps: read the cited Mara items, close only when the `Done when` checks pass, create issues only with `zban create`, and keep one issue per pull request, filing new work with `--discovered-from`. The project status line now says implementation is in progress. `CLAUDE.md` is still a symlink to it.
- This issue was started, ticked and closed with zban. The follow-up zb-hr38j4, a `draft` until ten issues have gone through zban, was created with `zban create`. This starts `VAL-DOGFOOD`.
- New test `this_repository_loads_the_guide` in `tests/prime.rs`: `AGENTS.md` contains the documented line, and `zban prime` run in this repository prints the guide. `this_repository_backlog_is_clean` in `tests/check.rs` already keeps `zban check` clean here.
- Rejected: telling agents to `cargo install --path .`, because one binary on `PATH` is shared by every worktree and can be stale or come from another branch.
- Checks run: `zban check` (no findings), `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all pass; the new test fails when the line is removed from `AGENTS.md`), `mara schema validate` and `mara project validate` (valid, evaluation complete). No Mara knowledge changed.
