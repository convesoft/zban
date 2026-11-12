---
title: Decide how agents close an issue whose Done when check runs only in CI
status: draft
type: task
priority: low
discovered_from: zb-hr38j4
created_at: 2026-11-12T16:23:58Z
---

Found by the `VAL-DOGFOOD` evaluation in zb-hr38j4. Three workers pushed their pull request with the issue still `in-progress` and stopped, and closed it only after a reminder: zb-71kr2e (#20, closed in a later commit, 5d45f56), zb-c83nw7 (#37, closed in 69e3032 after release run 43253925537) and zb-ve67q7 (#39, still `in-progress`). In each, the last unticked item was a build-only run of `release.yml` on the pull request itself, which can only run after the push. The guide says to close the issue together with the change, and `AGENTS.md` says to close only when the `Done when` checks pass, but neither says what to do when a check can only run in CI.

## Options

1. `AGENTS.md`: when a `Done when` check can only run in CI on the pull request, push, wait for the run to finish, then tick the item, close the issue in a follow-up commit on the same branch and push again before reporting.
2. Write only `Done when` checks that run locally, and leave CI results to the pull request and its reviewer. The release-workflow checks of the three issues above could then not be stated in the issue.
3. Change the agent guide (`DES-AGENT-GUIDE`) to say to wait for pending checks, such as CI, before closing. This applies to every project, needs a `REQ-AGENT-GUIDE` and `DES-AGENT-GUIDE` change and costs guide lines.
4. Keep relying on whoever runs the agents to remind them.

## Proposed answer

Option 1. It is a convention of this repository, like the one zb-mq3eqv recorded, so no Mara item changes.

## Tasks once decided

- [ ] Record the chosen convention in `AGENTS.md`, or in Mara first if the guide changes
