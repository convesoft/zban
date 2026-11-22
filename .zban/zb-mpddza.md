---
title: Validate every value of create and update before loading the project
status: todo
type: bug
tags:
  - audit
created_at: 2026-11-22T14:28:49Z
---

Found by the architecture audit after milestone zb-v90b0k, at e1c9354, and reproduced with the built binary. The behaviour is the same in 0.1.0.

`create` and `update` check their values in three phases (`src/operations.rs`, about lines 210–235 and 326–395; `Issue::validate` in `src/issue.rs`, about lines 378–399): some before the project is loaded, some after the issue is resolved, and the title and tags only when the file is encoded, after the closing rules ran. So which error a request gets depends on what else is wrong with it:

- `zban update zb-000000 --add-blocked-by nonsense` fails with `invalid_reference`, but `zban update zb-000000 --status nope` fails with `not_found`.
- `zban update ID --add-tag "Bad Tag" --status done` on an issue with an unchecked task item fails with `unchecked_items`, while the same tag alone fails with `invalid_value`.
- `zban create "" -s done --body "- [ ] x"` fails with `unchecked_items`, not `invalid_value`, and `zban create ""` loads the whole project before refusing the title.

Nothing is written in any of these cases. `list` already validates its values before loading (`src/operations.rs`, about lines 241–255).

The audit also found that `Issue::parse` repeats the messages of `check_tag` and of the title check (`src/issue.rs`, about lines 316–328, 378–384 and 473–488). Sharing a `check_title` next to `check_tag` gives this fix its seam.

Relevant items: `DES-CLI` (error codes), `DES-SHARED-CORE`, `DES-ISSUE-FILE`. Related: zb-32nknc (`update --remove-tag` accepts an invalid tag) and zb-rby6kf (the tag rule in `DES-ISSUE-FILE`).

## Tasks

- [ ] Decide in `DES-CLI` which error wins when a request has several faults, and record it
- [ ] Share `check_title` and `check_tag` between `Issue::parse`, `Issue::validate` and the operations, so each message exists once
- [ ] Validate every value of `create` and `update` before loading the project and before the closing rules
- [ ] Cover the cases above through the real binary and the MCP server

## Done when

- [ ] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [ ] `zban check` reports no errors
