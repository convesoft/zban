---
title: Close VER-CHECK gaps for interrupted-write temporary files and findings on moved files
status: todo
type: task
priority: deferred
tags:
  - deferred
  - backlog
discovered_from: zb-a76tqw
created_at: 2026-11-22T13:03:10Z
---

Two gaps between `VER-CHECK` and its tests, both older than zb-a76tqw and found in its review.

- The temporary file of a real interrupted write is checked only through the library (`src/store.rs`, test near line 769). The CLI test in `tests/check.rs` (near line 474, `fix_moves_only_misplaced_files_and_deletes_only_temporary_files`) uses a handwritten `.zban-*.tmp` file instead, so the binary is never shown to report a file left behind by a real interrupted write.
- `VER-CHECK` says `zban check --fix` leaves every other finding "untouched", but when it moves a misplaced issue that also has an unresolved finding, the finding is still reported, now at the moved path. Either the item should say that findings on a moved file follow it, or the behaviour should change; decide which in Mara first.

## Tasks

- [ ] Cover the temporary file of a real interrupted write through the binary, or record in `VER-CHECK` why the library check suffices
- [ ] Reconcile `VER-CHECK`'s "untouched" with findings reported at a moved file's new path, in Mara, and test the result
