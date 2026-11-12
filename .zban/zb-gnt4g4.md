---
title: Command line with JSON output
status: todo
type: feature
priority: high
parent: zb-9jra34
blocked_by:
  - zb-2b3mf7
  - zb-ramrc1
created_at: 2026-11-11T21:00:06Z
---

Implements `DES-CLI` for `init`, `create`, `list`, `show`, `update` and `delete`, and satisfies `REQ-NO-GIT` and `REQ-DAMAGED-DATA`. Dependencies follow `ADR-DEPENDENCIES`.
The `check`, `prime` and `mcp` commands come with their own issues, and the handoff
options and fields with zb-qkj8tj.

## Done when

- [ ] `VER-CLI` passes for the commands above
- [ ] `VER-NO-GIT` passes
