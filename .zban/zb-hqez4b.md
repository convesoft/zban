---
title: Report and remove leftover temporary files in check
status: todo
type: task
priority: low
parent: zb-9jra34
blocked_by:
  - zb-yj8xta
discovered_from: zb-n938cw
created_at: 2026-11-12T08:16:07Z
---

An interrupted write (`REQ-PRESERVE-CONTENT`) can leave a `.zban-<random>.tmp` file in `.zban/` or `.zban/archive/`. zban never reads it as an issue, but Git lists it as untracked, so `git add -A` could commit it.

Maintainer decision: `zban check` reports leftover `.zban-*.tmp` files, and `zban check --fix` deletes them.

Update `DES-CHECK` and `VER-CHECK` in Mara first, in the same change, then implement and test the new finding and its repair.
