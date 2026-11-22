---
title: Print a success line from zban check when no problems are found
status: todo
type: feature
priority: high
tags:
  - cli
parent: zb-v90b0k
created_at: 2026-11-22T10:56:26Z
---

`zban check` prints nothing to stdout when it finds no problems. Across T3 Code sessions using zban, 41% of CLI invocations (1,119 of 2,747) were wrapped with `echo $?` or `&& echo ok` because empty output looks ambiguous to agents and reviewers.

## Tasks

- [ ] Update `DES-CHECK` (and `DES-CLI` if needed) to specify a human-output success line when `check` finds no problems (e.g. `No problems found.`, matching `list`'s `No issues.`), leaving `--json` output and exit status unchanged
- [ ] Print the success line in CLI human output when `findings` is empty
- [ ] Update `VER-CHECK` and add/update CLI tests covering clean human output and unchanged `--json` output

## Done when

- [ ] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [ ] `zban check` reports no errors
