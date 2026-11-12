---
title: Deliver zban v1
status: done
type: epic
created_at: 2026-11-11T21:00:00Z
closed_at: 2026-11-12T15:20:21Z
---

The v1 scope is `ADR-SMALL-SCOPE`. Done when every child issue is closed.

## Summary of Changes

All 27 child issues are closed. zban v1 (`ADR-SMALL-SCOPE`) was built through pull requests #3–#18, reviewed in full before release, hardened by the review findings in #19–#30 (write and read locks on `.zban/`, handoff read-back, closing rules on create, Markdown fence rule, release smoke tests and more), and released as v0.1.0 (https://github.com/convesoft/zban/releases/tag/v0.1.0, tag at `9169669`) with the release issue closed in #31. This repository now manages its own backlog with the released `zban` (#12, #32).

Follow-up work stays in the backlog without a parent: zb-mm8qr5, zb-zqn4z3, zb-fws6g7, zb-gjwr73, and the drafts zb-cpwh20, zb-2fcnft and zb-hr38j4.
