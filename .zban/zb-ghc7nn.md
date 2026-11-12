---
title: Apply the closing rules to create
status: todo
type: bug
priority: high
parent: zb-9jra34
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
---

Found by the v0.1.0 review: `zban create "x" -s done --body "- [ ] y"` creates a done issue with an unchecked item, because only `update` applies the closing rules (`REQ-CLOSE-CHECKLIST`, `DES-LIFECYCLE`).

Maintainer decision: `create` refuses `done` with unchecked task items (`unchecked_items`), the same as `update`. `create` gets no `--force`; to record a closed issue with open items, create it open and close it with `update --force`. Record this in `DES-LIFECYCLE` first.

- [ ] Mara updated (`DES-LIFECYCLE`, `VER-LIFECYCLE`)
- [ ] CLI and MCP `create` refuse it, with a test through the binary
