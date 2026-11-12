---
title: Test atomic replacement, conflict through update and every error code
status: todo
type: task
priority: high
parent: zb-9jra34
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
---

Found by the v0.1.0 review: the interrupted-write test never calls the real replace path, `conflict` is only asserted on the store and never through `update` or the binary, and the error-code test uses a hand-written list that misses `conflict` and `invalid_handoff` (`VER-ISSUE-FILES`, `VER-CLI`).

- [ ] A test fails if replacing an issue stops being an atomic rename (for example, the file's inode changes across an update)
- [ ] `conflict` is produced through `update`
- [ ] The every-error-code test derives its list from `ErrorCode`, so a new code without a test fails
