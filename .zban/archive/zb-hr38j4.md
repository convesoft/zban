---
title: Evaluate dogfooding after ten issues
status: done
type: task
discovered_from: zb-as3s58
created_at: 2026-11-12T10:12:48Z
closed_at: 2026-11-12T16:24:34Z
---

Evaluate `VAL-DOGFOOD`. This stays a `draft` until at least ten issues, counting from zb-as3s58, have been worked on and closed through zban, some of them in parallel worktrees. Then set it `todo`.

- [x] Review the history of those issues against `VAL-DOGFOOD`
- [x] File each shortfall as an issue, or change the Mara knowledge


## Summary of Changes

Scope: the 23 issues closed through zban after zb-as3s58 (#12), in #13 to #38, with up to five pull requests open at once (#20 to #24 at 13:24Z), plus the open #37 and #39. Sources: `.zban/archive/`, `git log` on `main`, the commits of #12 to #39, `zban check` at every `main` commit, and the coordinator's brief.

`VAL-DOGFOOD`, claim by claim:

- **Backlog managed only through zban: met.** Every issue file on `main` (271c19e) is byte-identical after a zban rewrite. No merge of `main` into a branch of #12 to #39 resolved a conflict in `.zban/`, and no commit mentions a repair. Git cannot show which command made each edit, for example the removal of zb-7mbhpw and the edit of the archived zb-0psza8 in 7495cc4, but none was a repair.
- **Agents found ready work: not met as worded, and the wrong measure.** Workers got one issue in their brief. The coordinator computed readiness by hand before #12 and with `zban list --ready --json` after it, and skipped issues with a running worker or an open pull request. This is the model in `ADR-NO-CLAIMS`. Readiness on a branch can be stale: zb-mq3eqv was closed in #32 while its blocker zb-aapz52 was still `in-progress` on that branch. The release was already out, and zb-aapz52 was closed in #31. zban allows any status change whatever the blockers, as designed. Changed `VAL-DOGFOOD`: whoever chooses the work finds it with `zban list --ready`.
- **Checklists and summaries without further prompting: partly met.** All 23 issues have every item ticked and a `## Summary of Changes` that names the checks run. History cannot show whether items were ticked as work went on, because most issues go from `todo` to the archive in one squash commit. Three workers stopped with the issue `in-progress` and needed a reminder to close it: zb-71kr2e (#20, 5d45f56), zb-c83nw7 (#37, 69e3032) and zb-ve67q7 (#39, still open). In each, the last item was a CI run on the pull request itself. Filed as draft zb-gnb630, with options and a proposed answer.
- **Discovered work filed as issues: met.** Issues with `discovered_from`: zb-x481ey, zb-2fcnft (#16), the 15 review findings (#19), zb-906f27 (#26), zb-45235x (#28), zb-c83nw7 and zb-ve67q7 (#34), zb-bjyq1s (#38) and zb-e7vs4j (#39). zb-rvab3w (#18), found in the CI run of another pull request, has no `discovered_from`. One duplicate: zb-7mbhpw (c5c7417, branch of zb-0psza8) and zb-906f27 (#26) report the same listing race. They were filed four minutes apart in parallel worktrees, and zb-7mbhpw was dropped in 7495cc4 once `main` brought in zb-906f27. Recorded in `RISK-DUPLICATE-WORK`, not as a feature, because preventing it needs state shared across worktrees (`ADR-SMALL-SCOPE`).
- **No manual frontmatter repair: met.** No repair after #12, and no merge conflict in `.zban/`. One older oddity: zb-qkj8tj, written by hand in #1, has `created_at` 19:00:00Z after its `closed_at` 10:49:32Z. `zban check` does not compare timestamps, and zban now sets both, so no issue was filed.
- **`zban check` clean on `main`: met.** zban 0.1.0 reported no findings at each of the 26 `main` commits from 4b3a003 (#12) to 271c19e (#38). CI, which runs `this_repository_backlog_is_clean`, passed on each of them.

The coordinator's observations, checked:

- Workers worked from the issue named in their brief, and the coordinator computed readiness (above): confirmed by the brief.
- Duplicate zb-906f27 and zb-7mbhpw: confirmed (above).
- New issues and release blockers reached fresh worktrees only through `main`: confirmed. #19 created 15 issues and added eight blockers to zb-aapz52, and #20 to #27 opened after it merged. This follows from `ADR-SMALL-SCOPE`, so it stays a process note.
- An issue being worked on shows `todo` on `main` until its pull request merges: confirmed. Of the 23 issues, only zb-aapz52 was ever `in-progress` on `main`, from #16 to #31, while it was handed off for the tag. This is expected under `SCN-PARALLEL-BRANCHES` and `ADR-NO-CLAIMS`: whoever runs the agents tracks who works on what.
- zb-mq3eqv closed with its blocker open on its branch: confirmed (above). No change, because the blocker had in fact been resolved on another branch.
- Workers waited for CI and needed a reminder: confirmed (above, zb-gnb630).
- The installed `zban` lags `main`: confirmed. #12 said to use `cargo run`, #19 committed `.mcp.json`, and #32 moved to the installed binary and the MCP server. The installed binary is 0.1.0, without the fixes of #34 to #38, and the issue format has not changed. `AGENTS.md` already says when to use `cargo run`. No action.
- No merge conflicts in `.zban/`, but semantic conflicts in code: confirmed. Example: 6bf5328 in #38 adapted new tests to the helper from #36. CI caught such conflicts on the branches, and `main` stayed green.

Knowledge changed:

- `VAL-DOGFOOD`: "Agents found ready work" now reads that whoever chose the work, a single agent or whoever ran the parallel agents (`ADR-NO-CLAIMS`), found it with `zban list --ready`. The checklist claim now reads that, given only an issue to work on, agents kept checklists and summaries and closed their issues without further prompting. Status stays `accepted`: the evaluation does not change whether the check is agreed.
- `RISK-DUPLICATE-WORK`, retitled "Parallel agents duplicate work", now also covers two agents filing the same discovered problem.

Checks run: `mara schema validate` (valid), `mara project validate` (valid, evaluation complete, no errors or warnings) and `zban check` (no findings). No code changed, so fmt, clippy and tests were not run.
