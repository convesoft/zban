---
title: Fall back when hard links are not supported
status: in-progress
type: bug
priority: low
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
---

Suspicion from the v0.1.0 review, not reproduced: `init` and `create` link the staged file into place, which would fail with `io` on filesystems without hard links (exFAT, some shared folders and FUSE mounts) (`DES-SHARED-CORE`).

- [x] Reproduce on such a filesystem
- [ ] Once the maintainer settles `ADR-CREATE-WITHOUT-HARD-LINKS`, implement the agreed fallback, keeping "create never replaces" as far as it says

## Handoff (2026-11-12T15:26:04Z)

- Branch `zb-gjwr73`, based on `origin/main` at f911097; the note is committed with the draft decision, and the working tree is clean after that commit.
- Blocked on a maintainer decision: the options and a proposed answer are in the draft `ADR-CREATE-WITHOUT-HARD-LINKS` (docs/decisions.mara.md). Nothing is implemented, as agreed for this issue.
- Reproduced with the installed zban 0.1.0, without root: `rclone mount SRC MNT --daemon --config /dev/null --vfs-cache-mode writes` (rclone 1.75.2 through the setuid `fusermount3`). `ln a b` there fails with "Operation not permitted". `zban init` fails with `io` and "config.toml: Operation not permitted (os error 1)"; with a valid config copied in, `zban create` fails the same way for an open and for a closed issue. `update`, close, reopen, `delete` and `check` work, and `flock` on `.zban/` excludes a second holder.
- Documentation agrees: Linux `link(2)` lists EPERM for a file system that does not support hard links; macOS `linkat(2)` lists ENOTSUP for one that does not support the call. Rust reports EPERM as `PermissionDenied`, and ENOSYS and EOPNOTSUPP as `Unsupported`.
- Proposed answer: keep the hard link and, when it fails with EPERM or as unsupported, take the exclusive `.zban/` lock, check that the name is unused in `.zban/` and the archive, and rename the staged file into place, as `place` already does for moves. Writes stay atomic (`REQ-PRESERVE-CONTENT`); "never replaces" weakens only against writers that do not take the lock, which `ADR-WRITE-LOCK` already accepts for replace and move.
- Rejected: exclusive creation of the final file then write and sync, which the checklist suggested, because an interrupted create leaves a partial issue file; `renameat2`/`renamex_np` without replacing, because it needs a new dependency and FUSE servers may not support it; hard links only with a clearer message, because zban stays unusable on FAT, exFAT and many FUSE mounts.
- Open choice for the maintainer: rename only as a fallback (proposed), or always check then rename and drop the hard link, which is simpler but carries the caveat on every file system.
- Next, once agreed: accept the ADR, update the exclusive-creation sentence in `DES-SHARED-CORE`, add a check to `VER-ISSUE-FILES`, then implement in `Staged`/`Project::init`/`Project::create_among` in src/store.rs with a test that drives the fallback directly.
- Found on the way and filed separately: zb-bjyq1s, a failed `init` leaves an empty `.zban/` behind, so a retry fails with `project_exists`.
- Files touched: docs/decisions.mara.md, .zban/zb-gjwr73.md, .zban/zb-bjyq1s.md.
