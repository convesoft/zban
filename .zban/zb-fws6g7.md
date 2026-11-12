---
title: Move shared request validation into the core
status: todo
type: task
priority: low
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
---

Found by the v0.1.0 review (`DES-SHARED-CORE`): the replace-pair length check and `removable` are duplicated in `src/cli.rs` and `src/mcp.rs`, and `Graph::ready`, `Project::create`, `Project::delete` and `create_with` are used only by tests.

- [ ] Move the shared validation into the core operations
- [ ] Remove or justify the test-only functions
