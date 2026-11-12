---
title: Integrity check
status: done
type: feature
parent: zb-9jra34
blocked_by:
  - zb-2b3mf7
  - zb-ramrc1
created_at: 2026-11-11T21:00:05Z
closed_at: 2026-11-12T09:26:56Z
---

Implements `REQ-CHECK` and `DES-CHECK`, including the `check` command.

## Done when

- [x] `VER-CHECK` passes
- [x] `zban check` is clean on this repository

## Summary of Changes

- New core module `check`: `Project::check(directory, fix)` finds the project, checks it and returns `Finding {severity, code, path, message}` ordered by path, then code, with the absolute path of each file. An invalid configuration is reported as the only finding, so `Project::find_root` and `Project::try_open` now split discovery from reading the configuration. With `fix`, each misplaced file moves through `Project::place`. A taken target leaves the file and its finding in place, and any other failed move fails the check.
- `ProblemCode` now holds all twelve `DES-CHECK` codes, with `severity()`, and the new `Severity` gives `error` or `warning`. Both serialize as their code words. `Graph::problems` in `relations` reports references to unknown issues or to the issue itself, blockers on an ancestor, cycles found with `petgraph`'s `tarjan_scc`, `related` links stored on both issues, and open issues under a closed parent.
- `operations::check` returns the `Checked` document, `{"findings": [...]}`. Like `init`, it takes a directory, because an invalid configuration is a finding rather than a failure. `zban check [--fix]` is a `Command` in `src/cli.rs` that returns `Reply::new(&checked, text).failed(has_errors)`, so it exits with 1 on errors and 0 on warnings alone, with or without `--json`. Human output in `src/cli/human.rs` prints one line per finding, `<path>: <severity>: <message> [<code>]`, with paths shown relative to the working directory when below it, and nothing for a clean check.
- Choices within the design:
  - Findings with the same path are sorted by their code string, and findings with the same path and code keep the order in which they were found.
  - `duplicate_id` is reported on both files. `related_both_sides` is reported once, on the issue with the later ID, so removing the reported copy keeps the link.
  - `cycle` gives one finding for each issue on a cycle, naming the shortest cycle through it, plus one for each blocker that is the issue's own ancestor.
  - `open_under_closed` looks at the direct parent only. `misplaced` concerns readable issues only, because an unreadable file's status is unknown.
  - Cycles are computed over IDs with one readable file, like readiness. The other relation findings come from each readable file's own references.
- Docs: `docs/issues.mara.md` shows `zban check` output and `--fix` under "Broken files". `DES-CLI` now says that a finding's `path` is the absolute path of the file, as for issues. The note in `docs/cli.mara.md` and the README status line no longer list `check` as missing.
- Tests: `tests/check.rs` (8 tests for `VER-CHECK`) runs the real binary over a corpus of broken projects, one or more for each `DES-CHECK` code. It checks code, severity, path, order and exit status, the warnings-only exit 0, `--fix` moving only misplaced files with their bytes unchanged, a taken target staying reported, the library's finding fields, this repository's backlog, and project discovery with `-C`. The `--json` test checks the exact document with absolute paths, also when run elsewhere with `-C`, exit 1 on errors and 0 on warnings alone, an empty list for a clean project, an invalid configuration, and the error object outside a project. zb-x59t84 adds `check` to `tests/no_git.rs` and to the JSON document test in `tests/cli.rs`.

Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (104 passed: 32 unit, 8 `check`, 18 `cli`, 5 `ids`, 11 `issue_files`, 12 `lifecycle`, 2 `no_git`, 6 `project_layout`, 10 `relations`), `mara schema validate` and `mara project validate` (`valid: true`, evaluation complete), and `zban check` on this repository after closing this issue (no findings, exit 0).
