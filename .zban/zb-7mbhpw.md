---
title: Listings can miss an issue that another process is replacing
status: todo
type: bug
parent: zb-9jra34
discovered_from: zb-0psza8
created_at: 2026-11-12T13:23:38Z
---

Found while reproducing zb-0psza8. On btrfs, a directory listing taken while another process renames a temporary file over an issue file often leaves that issue out: 8512 of 20000 listings missed the file during continuous renames, and none did on tmpfs. Every zban command lists `.zban/` (`DES-SHARED-CORE`), so a command that runs while the same issue is updated can fail with `not_found`, `list` can leave the issue out, and `delete` or a relation check can miss a referrer (`REQ-RELATIONS`).

The parallel-update test in `tests/simultaneous_changes.rs` retries `not_found` for this reason.

Proposed fix, which needs a maintainer decision because it extends the lock of zb-0psza8 beyond compare-and-rename: hold a shared lock on `.zban/` while listing `.zban/` and `.zban/archive/`, so that a listing never overlaps a rename by zban. Renames by editors or Git stay unsynchronized.

- [ ] Approach decided and recorded in Mara
- [ ] A listing taken during zban's own writes always includes the issue being written
- [ ] The parallel-update test no longer retries `not_found`
