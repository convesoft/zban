---
title: Fall back when hard links are not supported
status: todo
type: bug
priority: low
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
---

Suspicion from the v0.1.0 review, not reproduced: `init` and `create` link the staged file into place, which would fail with `io` on filesystems without hard links (exFAT, some shared folders and FUSE mounts) (`DES-SHARED-CORE`).

- [ ] Reproduce on such a filesystem
- [ ] If confirmed, fall back to exclusive create when linking is unsupported, keeping "create never replaces"
