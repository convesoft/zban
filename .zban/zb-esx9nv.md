---
title: Add --not-tag / not_tags negative tag filter to list
status: todo
type: feature
tags:
  - cli
  - mcp
related:
  - zb-wbm14r
  - zb-banzx5
created_at: 2026-11-22T10:56:42Z
---

Even with `draft` cascading (zb-banzx5), orthogonal tags on active `todo` work (such as `maintainer` or `slow`) benefit from a direct exclusion filter on `list` / `list_issues`.

Add a repeatable negative tag filter:

- CLI: `--not-tag T` (repeatable) on `zban list`
- MCP: `not_tags` (list of strings) on `list_issues`, aligned with the `tags` naming in zb-wbm14r
- Semantics: excludes any issue carrying any of the given tags, composing with `--tag` and all other filters (`REQ-LIST-FILTERS`).

## Tasks

- [ ] Update `REQ-LIST-FILTERS`, `DES-CLI`, `DES-MCP-TOOLS`, `DES-AGENT-GUIDE`, `VER-CLI`, `VER-SURFACE-PARITY` and `VER-AGENT-GUIDE`
- [ ] Implement `--not-tag` in core `ListRequest`, CLI `list`, MCP `list_issues` (`not_tags`), and the CLI guide synopsis (keeping MCP instructions within the character budget in `DES-AGENT-GUIDE`)
- [ ] Add CLI and MCP parity tests covering single and repeated `--not-tag`/`not_tags` values, composition with `--tag`/`tags` and `--ready`, and tag validation

## Done when

- [ ] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [ ] `zban check` reports no errors
