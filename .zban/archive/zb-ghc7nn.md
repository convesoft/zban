---
title: Apply the closing rules to create
status: done
type: bug
priority: high
parent: zb-9jra34
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
closed_at: 2026-11-12T13:23:04Z
---

Found by the v0.1.0 review: `zban create "x" -s done --body "- [ ] y"` creates a done issue with an unchecked item, because only `update` applies the closing rules (`REQ-CLOSE-CHECKLIST`, `DES-LIFECYCLE`).

Maintainer decision: `create` refuses `done` with unchecked task items (`unchecked_items`), the same as `update`. `create` gets no `--force`; to record a closed issue with open items, create it open and close it with `update --force`. Record this in `DES-LIFECYCLE` first.

- [x] Mara updated (`DES-LIFECYCLE`, `VER-LIFECYCLE`)
- [x] CLI and MCP `create` refuse it, with a test through the binary

## Summary of Changes

`create` now applies the `done` rule of `DES-LIFECYCLE`: creating an issue as `done` while its body has unchecked task items fails with `unchecked_items` and writes nothing, through the CLI and MCP alike. `create` has no force; the error says to create the issue open and then force setting done.

- Mara: `DES-LIFECYCLE` gained the "Creating closed" rule, `VER-LIFECYCLE` checks it, and the closing guidance in `docs/issues.mara.md` shows the workaround.
- Core: `src/lifecycle.rs` splits the unchecked-items check out of `refuse_unfinished` into `refuse_unchecked`, which `update` and the new `refuse_unfinished_creation` share; `operations::create` calls it. The adapters are unchanged.
- Tests: `tests/lifecycle_cli.rs` `creating_done_with_an_unchecked_item_fails_and_cannot_be_forced` runs the binary (refused and nothing written, `create --force` is a usage error, checked or fenced items and `cancelled` are allowed, create open then `update --force` works). `tests/mcp.rs` `invalid_arguments_give_the_same_error_codes` adds the same refusal and `force` on `create_issue`, with equal CLI and MCP documents. The new CLI test fails without the fix.

Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (153 passed), `mara schema validate`, `mara project validate` (valid, evaluation complete, no diagnostics), and the issue's repro `zban create "x" -s done --body "- [ ] y"` now exits 1 with `unchecked_items` and creates no file.
