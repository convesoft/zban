---
title: Decide how agents close an issue whose Done when check runs only in CI
status: done
type: task
priority: low
discovered_from: zb-hr38j4
created_at: 2026-11-12T16:23:58Z
closed_at: 2026-11-12T16:30:56Z
---

Found by the `VAL-DOGFOOD` evaluation in zb-hr38j4. Three workers pushed their pull request with the issue still `in-progress` and stopped, and closed it only after a reminder: zb-71kr2e (#20, closed in a later commit, 5d45f56), zb-c83nw7 (#37, closed in 69e3032 after release run 43253925537) and zb-ve67q7 (#39, still `in-progress`). In each, the last unticked item was a build-only run of `release.yml` on the pull request itself, which can only run after the push. The guide says to close the issue together with the change, and `AGENTS.md` says to close only when the `Done when` checks pass, but neither says what to do when a check can only run in CI.

## Options

1. `AGENTS.md`: when a `Done when` check can only run in CI on the pull request, push, wait for the run to finish, then tick the item, close the issue in a follow-up commit on the same branch and push again before reporting.
2. Write only `Done when` checks that run locally, and leave CI results to the pull request and its reviewer. The release-workflow checks of the three issues above could then not be stated in the issue.
3. Change the agent guide (`DES-AGENT-GUIDE`) to say to wait for pending checks, such as CI, before closing. This applies to every project, needs a `REQ-AGENT-GUIDE` and `DES-AGENT-GUIDE` change and costs guide lines.
4. Keep relying on whoever runs the agents to remind them.

## Proposed answer

Option 1. It is a convention of this repository, like the one zb-mq3eqv recorded, so no Mara item changes.

## Decision

Maintainer decision (2026-11-12), in place of option 1 because delegated agents here cannot wait on pull request checks: if the only remaining check of an issue is CI on the issue's own pull request, close the issue in that pull request and say in its `## Summary of Changes` that CI on the pull request covers that check. Pull requests are merged only once CI is green, so the issue reaches `main` closed only if the check passed. If CI fails, fix it in the same pull request. It is a repository convention, so Mara changes only if an item, such as `VAL-DOGFOOD`, states the opposite.

## Tasks

- [x] Add the rule to "Working on an issue" in `AGENTS.md`
- [x] Reword the `AGENTS.md` bullet on setting `done` so that it does not contradict the rule
- [x] Check that no Mara item states the opposite

## Done when

- [x] `mara schema validate`, `mara project validate`, `zban check` and `cargo test` pass
- [x] CI passes on this issue's pull request

## Summary of Changes

Recorded the maintainer decision above in `AGENTS.md` under "Working on an issue". The bullet on setting `done` now excepts CI on the issue's own pull request, and a new bullet says what to do then: tick the check, close the issue in that pull request, and say in `## Summary of Changes` that CI on the pull request covers it, fixing any CI failure in the same pull request. No Mara item states the opposite: `VAL-DOGFOOD` and `SCN-AGENT-SESSION` already have the agent close its issue with the change, so no Mara item changed.

Checks run: `mara schema validate` (valid), `mara project validate` (`valid: true`, evaluation complete, no diagnostics), `zban check` (no findings), `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` (all pass, including `this_repository_loads_the_guide` and `this_repository_backlog_is_clean`). The last `Done when` item, CI on this issue's pull request, is covered by CI on that pull request under the new rule, which this issue applies to itself.
