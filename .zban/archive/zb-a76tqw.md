---
title: Print a success line from zban check when no problems are found
status: done
type: feature
priority: high
tags:
  - cli
parent: zb-v90b0k
created_at: 2026-11-22T10:56:26Z
closed_at: 2026-11-22T12:53:10Z
---

`zban check` prints nothing to stdout when it finds no problems. Across T3 Code sessions using zban, 41% of CLI invocations (1,119 of 2,747) were wrapped with `echo $?` or `&& echo ok` because empty output looks ambiguous to agents and reviewers.

## Tasks

- [x] Update `DES-CHECK` (and `DES-CLI` if needed) to specify a human-output success line when `check` finds no problems (e.g. `No problems found.`, matching `list`'s `No issues.`), leaving `--json` output and exit status unchanged
- [x] Print the success line in CLI human output when `findings` is empty
- [x] Update `VER-CHECK` and add/update CLI tests covering clean human output and unchanged `--json` output

## Done when

- [x] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [x] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [x] `zban check` reports no errors

## Summary of Changes

- `zban check` human output prints `No problems found.` when it finds nothing (`src/cli/human.rs`). `--json` output (`{"findings": []}`) and exit status are unchanged, and output with findings, including warnings only, is unchanged.
- `DES-CHECK` specifies the success line and that JSON and exit status do not change; `VER-CHECK` requires a clean project to print it, exit 0 and return `{"findings": []}` in JSON. `DES-CLI` needed no change, as it defers check output to `DES-CHECK`.
- `tests/check.rs`: the clean-project test now expects exactly `No problems found.\n` on stdout and nothing on stderr (renamed to `only_warnings_exit_zero_and_a_clean_project_says_so`); the existing JSON assertion `{"findings": []}` with exit 0 still covers unchanged `--json` output. The `check` helper accepts the success line only as the whole output, so it can never mix with findings.

Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all pass); `mara schema validate` and `mara project validate` (`valid: true`, `evaluation_complete: true`); `zban check` with the worktree binary (`No problems found.`, exit 0).

## Decisions made

- The success line appears only when there are no findings at all; a check with only warnings prints the warnings and no success line, since warnings are problems worth reading (recorded in `DES-CHECK`).
- `check --fix` that repairs every finding also prints the success line, because the fixed findings are not reported (follows from `DES-CHECK`).
