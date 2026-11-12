---
title: Move shared request validation into the core
status: done
type: task
priority: low
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
closed_at: 2026-11-12T15:27:34Z
---

Found by the v0.1.0 review (`DES-SHARED-CORE`): the replace-pair length check and `removable` are duplicated in `src/cli.rs` and `src/mcp.rs`, and `Graph::ready`, `Project::create`, `Project::delete` and `create_with` are used only by tests.

- [x] Move the shared validation into the core operations
- [x] Remove or justify the test-only functions

## Summary of Changes

- `UpdateRequest` now carries the parameters as both surfaces give them: `parent`/`no_parent`, `discovered_from`/`no_discovered_from`, `handoff`/`no_handoff` and the `replace_old`/`replace_new` lists. `operations::update` takes the `Surface` and checks first that the replacement lists pair up and that no value is both set and removed, with the same `usage` messages as before (`DES-SHARED-CORE`). The `Replacement` type and both adapters' `removable` helpers are gone; `src/cli.rs` and `src/mcp.rs` only map fields. The command line still rejects `--parent` with `--no-parent` and a lone `--replace-old` while parsing, with clap's messages.
- `Graph::ready` and the public `Project::delete` were used only by tests on current `main`. Both are removed; `delete_among` became the crate-private `Project::delete`, and `tests/relations.rs` now uses `operations::list` with `ready` and `operations::delete`.
- `Project::create` and `Project::create_with` stay, each with a one-line reason: tests write exact issues that a `CreateRequest` cannot describe, and choose the IDs to draw.
- New test `unpaired_replacements_and_set_and_removed_values_are_usage_errors` in `tests/mcp.rs` (`VER-SURFACE-PARITY`) pins the messages on both surfaces. It passed before the refactor and after it. The existing parity tests are unchanged.
- Behaviour note: these `usage` checks now run after project discovery, as the core's other request checks do. A request that also fails discovery (or, over MCP, has a relative `directory`) now reports that failure first.
- Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (173 passed, 0 failed), `mara schema validate` and `mara project validate` (`valid: true`, evaluation complete).
