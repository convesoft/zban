---
title: Refuse an invalid tag in update --remove-tag
status: todo
type: task
priority: deferred
tags:
  - deferred
  - backlog
discovered_from: zb-esx9nv
created_at: 2026-11-22T14:01:49Z
---

`zban update ID --remove-tag "Bad Tag"` succeeds and changes nothing, although `DES-CLI` says a value that breaks `DES-ISSUE-FILE` fails with `invalid_value`. A tag that no readable issue can carry is a typo, and removing it silently hides that. `--add-tag` already refuses such a value (through `Issue::validate`), and since zb-esx9nv so do `list --tag` and `list --not-tag` (`check_tag` in `src/issue.rs`). Removing a valid tag the issue does not carry stays a no-op.

Found while working on zb-esx9nv. The gap predates that change.

## Tasks

- [ ] Validate `--remove-tag` / `remove_tag` values with `check_tag` in `operations::update`, failing with `invalid_value` without writing
- [ ] State the rule in `DES-CLI` and cover it in `VER-CLI` and the MCP parity error-code test (`VER-SURFACE-PARITY`)
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `mara project validate` pass
