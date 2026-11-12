---
title: Decide how to handle temporary files left by interrupted writes
status: draft
type: task
priority: low
parent: zb-9jra34
discovered_from: zb-n938cw
created_at: 2026-11-12T08:16:07Z
---

An interrupted write (`REQ-PRESERVE-CONTENT`) can leave a `.zban/.zban-<random>.tmp` file next to the issue it was replacing. zban never reads it as an issue, but Git lists it as untracked, so `git add -A` could commit it.

Options: have `zban check` report such files and `--fix` remove them, which changes `DES-CHECK`, or mention them in the user guide and leave them to the user. Record the decision in Mara before implementing (`ADR-SMALL-SCOPE`).
