---
title: Handoff notes
status: done
type: feature
parent: zb-9jra34
blocked_by:
  - zb-bbyjed
created_at: 2026-11-12T19:00:00Z
closed_at: 2026-11-12T10:49:32Z
---

Implements `REQ-HANDOFF` and `DES-HANDOFF` on both the CLI and the MCP server,
including the handoff section of the agent guide.

## Done when

- [x] `VER-HANDOFF` passes
- [x] `VER-SURFACE-PARITY` and `VER-AGENT-GUIDE` still pass

## Summary of Changes

- Core: `src/handoff.rs` reads, writes and removes the handoff section; `src/markdown.rs` gains line offsets, ATX heading detection and a fence tracker; new error code `invalid_handoff`. `update` applies the handoff after `--append`; `show` adds `handoff`, `list` adds `handoff_at`, and human output shows the note's age.
- Adapters: CLI `--handoff TEXT|-` / `--no-handoff`; MCP `handoff` / `no_handoff`.
- Guide: a Handoffs section and the new parameters through the operation table; CLI alternatives now render as `[a | b]`. Both snapshots updated.
- Mara: `DES-HANDOFF` now also rejects text that leaves a fence open, and removing a note that ends the body drops the blank lines before it; `DES-CLI` records the edit order and the `invalid_handoff` cases. Removed the "not implemented" notes from the docs and README.

Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (`tests/handoff.rs` for `VER-HANDOFF`, `tests/mcp.rs` for `VER-SURFACE-PARITY`, `tests/prime.rs` for `VER-AGENT-GUIDE`), `mara schema validate` and `mara project validate` (valid, evaluation complete).
