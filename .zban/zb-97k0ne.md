---
title: Fail list --parent on an unknown issue
status: todo
type: bug
priority: high
parent: zb-9jra34
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
---

Found by the v0.1.0 review: `zban list --parent` with an unknown or malformed ID returns an empty list with exit 0, while `show` and `update` fail with `not_found`. Also, `DES-CLI` says every command except `init` and `prime` fails with `project_not_found`, but `mcp` starts anyway, as `DES-MCP-TOOLS` says.

Maintainer decision: `list --parent` fails with `not_found` (or the usual ID error) for an unknown or malformed ID, on both surfaces. Record it in `DES-CLI` first, and correct the `mcp` exception there.

- [ ] Mara updated (`DES-CLI`, `VER-CLI`)
- [ ] CLI and MCP behave so, with tests
