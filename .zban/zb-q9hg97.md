---
title: "Self-parented issue: child_counts counts itself and mutation results can be stale"
status: todo
type: bug
priority: deferred
tags:
  - deferred
  - backlog
discovered_from: zb-y2kwfz
created_at: 2026-11-22T13:27:04Z
---

A hand-edited issue file can name its own ID as `parent`; the CLI and MCP refuse to create such a reference, and `zban check` is the place that reports it. For such an issue, `child_counts` (`DES-CLI`) counts the issue as its own child in `show` and `list`, while the close guard (`open_children`, `DES-LIFECYCLE`) skips it.

The result of an update that closes such an issue or clears its parent is built from the graph loaded before the write (`update` in `src/operations.rs`, the `view_of` call), so its `child_counts` can carry the count from before the update.

Found while reviewing zb-y2kwfz. Exotic: it needs a hand edit that the tools refuse.

- [ ] Decide whether `child_counts` should skip an issue that names itself as parent, matching the close guard
- [ ] Make mutation results report counts that match the written state, with a test through the real binary
