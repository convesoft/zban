---
title: Fall back when hard links are not supported
status: done
type: bug
priority: low
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
closed_at: 2026-11-12T15:49:25Z
---

Suspicion from the v0.1.0 review, not reproduced: `init` and `create` link the staged file into place, which would fail with `io` on filesystems without hard links (exFAT, some shared folders and FUSE mounts) (`DES-SHARED-CORE`).

- [x] Reproduce on such a filesystem
- [x] Once the maintainer settles `ADR-CREATE-WITHOUT-HARD-LINKS`, implement the agreed fallback, keeping "create never replaces" as far as it says

## Summary of Changes

Reproduced with zban 0.1.0 on an unprivileged `rclone mount` of a local directory (rclone 1.75.2 through the setuid `fusermount3`): linking fails with EPERM there, so `init` and `create` failed with `io`, while update, close, reopen, delete and advisory locks worked. Linux `link(2)` documents EPERM for a file system without hard links, and macOS `linkat(2)` documents ENOTSUP.

- The maintainer accepted `ADR-CREATE-WITHOUT-HARD-LINKS`: keep the hard link, and only when linking fails because the file system has none (EPERM on Linux, or unsupported as for the lock), check the name under the exclusive lock and rename the temporary file into place. `DES-SHARED-CORE`, `REQ-NO-LOST-UPDATES` and `VER-ISSUE-FILES` now say precisely when "a create never replaces an existing file" holds: absolutely where hard links work, and against other zban processes otherwise. `REQ-PRESERVE-CONTENT` holds unchanged, because the fallback renames.
- `src/store.rs`: `init` and `create` share `Project::add_file`, which links and falls back as decided. The link function is a parameter, so tests can make it fail. The lock's unsupported check became `unsupported`, and `linking_unsupported` adds Linux EPERM.
- Unit tests make linking fail. They check the error mapping, that `init` and `create` rename into place for open and closed issues, that an ID taken in `.zban/` or `.zban/archive/` is skipped while the taken files stay unchanged and no temporary file is left, and that any other link error fails with `io` and writes nothing. Mutation checks confirmed that the tests fail without the fallback and without the locked re-check.
- Rejected, as recorded in the ADR: exclusive creation of the final file (an interrupted create would leave a partial issue), `renameat2`/`renamex_np` (would need a new dependency, and FUSE support varies), hard links only (zban would stay unusable on such file systems), and always renaming (every file system would carry the caveat).

Checks run: `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked` (176 passed), Mara `schema validate` and `project validate` (valid, complete, no diagnostics), `zban check`. End to end on the rclone mount with this branch's binary, `init`, `create` of an open and a closed issue, update, close, reopen, `list --all` and `check` all succeeded. zb-bjyq1s (a failed `init` leaves an empty `.zban/`) stays open as its own issue.
