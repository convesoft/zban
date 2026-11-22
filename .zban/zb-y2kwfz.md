---
title: Report direct-child progress counts on issues (child_counts)
status: in-progress
type: feature
tags:
  - cli
  - mcp
parent: zb-v90b0k
created_at: 2026-11-22T10:56:28Z
---

Progress questions were the single most frequent maintainer prompt in orchestrated zban threads, leading projects to write external scripts over `zban list --all --json` just to count closed vs. total children of epics.

Expose direct-child progress counts on issues alongside `checklist`:

- Always include `child_counts: {closed, total}` on `IssueView` in JSON (for `list`, `show` and mutations), counting direct children only — matching `--parent` and `open_children` — and avoiding collision with `show`'s `children` ID array.
- Show child progress in human output (`list` and `show`) when `total > 0`.

## Tasks

- [ ] Update `DES-CLI` (and `VER-CLI` / `VER-SURFACE-PARITY`) to specify `child_counts: {closed, total}` for direct children in JSON and human output
- [ ] Compute and include `child_counts` on `IssueView` in core operations and render it in CLI human output when `total > 0`
- [ ] Add tests covering leaf issues (`{closed: 0, total: 0}`), parents with open and closed direct children, nested epics (direct children only), and CLI/MCP parity

## Done when

- [ ] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [ ] `zban check` reports no errors
