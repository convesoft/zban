---
title: Do not leave .zban/ behind when init fails
status: done
type: bug
priority: low
discovered_from: zb-gjwr73
created_at: 2026-11-12T15:24:39Z
closed_at: 2026-11-12T16:18:59Z
---

`zban init` creates `.zban/` and then writes `config.toml` into it (`DES-PROJECT-LAYOUT`). When writing the configuration fails, the empty `.zban/` stays behind. No project is found there, because discovery needs `.zban/config.toml`, yet running `init` again fails with `project_exists` until the directory is removed by hand.

Seen with zban 0.1.0 on an `rclone mount` of a local directory, where linking the configuration into place fails with `io` (EPERM, see zb-gjwr73). A full disk or any other write error leaves the same state.

- [x] Reproduce with a write error that does not depend on hard links
- [x] Remove the directory `init` created when it fails afterwards, or decide in Mara how a half-made project is reported, and cover it with a test

## Summary of Changes

- Reproduced with zban 0.1.0 without hard links: running `init` under a file size limit of 0 (`ulimit -f 0`, `SIGXFSZ` ignored) fails with `io` on the configuration write, leaves an empty `.zban/`, and a second `init` fails with `project_exists`.
- `Project::init` now writes the configuration in `write_config` and, when that fails, removes the `.zban/` it has just created with `fs::remove_dir`, which removes it only while it is empty. A `.zban/` that already existed still fails with `project_exists` and is left alone, and an invalid prefix is still rejected before anything is created.
- Mara: `DES-PROJECT-LAYOUT` states the removal and `VER-ISSUE-FILES` checks it, for a failed link and for a failed write.
- Tests: `init_and_create_fail_with_io_on_any_other_linking_error` now expects no `.zban/` after a failed init and a successful retry; `a_failed_init_removes_only_the_empty_directory_it_created` covers files another writer put there and an existing `.zban/` without configuration; `init_that_cannot_write_its_configuration_leaves_no_zban` runs the real binary under the file size limit. Both new checks fail with the removal disabled.
- Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all passed), `mara schema validate` and `mara project validate` (`valid: true`, evaluation complete, 0 errors).
