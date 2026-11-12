---
title: Report and remove leftover temporary files in check
status: done
type: task
priority: low
parent: zb-9jra34
blocked_by:
  - zb-yj8xta
discovered_from: zb-n938cw
created_at: 2026-11-12T08:16:07Z
closed_at: 2026-11-12T10:14:43Z
---

An interrupted write (`REQ-PRESERVE-CONTENT`) can leave a `.zban-<random>.tmp` file in `.zban/` or `.zban/archive/`. zban never reads it as an issue, but Git lists it as untracked, so `git add -A` could commit it.

Maintainer decision: `zban check` reports leftover `.zban-*.tmp` files, and `zban check --fix` deletes them.

Update `DES-CHECK` and `VER-CHECK` in Mara first, in the same change, then implement and test the new finding and its repair.

The agent guide (`src/guide.md`) says what `--fix` repairs; update it and its snapshot too.

- [x] Record the finding and its repair in `REQ-CHECK`, `DES-CHECK` and `VER-CHECK`
- [x] Report leftover temporary files in `zban check`
- [x] Delete them in `zban check --fix`
- [x] Update the user docs, the agent guide and its snapshot
- [x] Test the finding and its repair through the binary

## Summary of Changes

- Recorded the decision in Mara: `REQ-CHECK` now names leftover temporary files as a warning and lets `--fix` delete them; `DES-CHECK` adds the `temporary_file` row (warning) and says `--fix` deletes such files, and that a write running at the same time then fails with `io` without changing its issue; `VER-CHECK` covers the new finding, its repair and lookalike names; `DES-PROJECT-LAYOUT` notes the exception to "other files are ignored". The user guidance in `docs/issues.mara.md` and `docs/cli.mara.md` follows.
- New code `temporary_file`, severity warning: zban never reads the file, no issue is wrong because of it, and one can exist briefly while another zban process writes, so it should not fail the check.
- `Project::temporary_files` lists `.zban-*.tmp` regular files directly inside `.zban/` and `.zban/archive/`, sharing one directory listing with `load`; the name pattern now lives next to `Staged`, which builds its names from it. `zban check --fix` deletes them, treating one already gone as deleted.
- Updated the `--fix` help, the agent guide and its snapshot.
- Tests: a `temporary-file` corpus case with lookalike names that are not reported, `--fix` deleting only the two temporary files while every other file stays byte for byte, the finding message, the invalid-configuration case reporting nothing else, and a unit test where real interrupted writes in both directories are reported and then deleted.
- Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (117 passed), `mara schema validate` and `mara project validate` (`valid: true`, evaluation complete, no diagnostics).
