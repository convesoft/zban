---
title: Fail list --parent on an unknown issue
status: done
type: bug
priority: high
parent: zb-9jra34
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
closed_at: 2026-11-12T13:22:45Z
---

Found by the v0.1.0 review: `zban list --parent` with an unknown or malformed ID returns an empty list with exit 0, while `show` and `update` fail with `not_found`. Also, `DES-CLI` says every command except `init` and `prime` fails with `project_not_found`, but `mcp` starts anyway, as `DES-MCP-TOOLS` says.

Maintainer decision: `list --parent` fails with `not_found` (or the usual ID error) for an unknown or malformed ID, on both surfaces. Record it in `DES-CLI` first, and correct the `mcp` exception there.

- [x] Mara updated (`DES-CLI`, `VER-CLI`)
- [x] CLI and MCP behave so, with tests

## Summary of Changes

- `DES-CLI` now says `list --parent` resolves its ID as `show` does: a malformed or unknown ID fails with `not_found`, and an ID naming two files or an unreadable file fails with `ambiguous_id` or `unreadable`. It also names `mcp` as an exception to `project_not_found`, pointing to `DES-MCP-TOOLS`. `VER-CLI` checks the new behaviour.
- The core `list` operation resolves the parent through `named` and `Graph::resolve`, the same path `show` takes, so the CLI and MCP get the same result.
- Tests: `list_parent_fails_on_an_issue_show_cannot_find` in `tests/cli.rs` runs the real binary and compares each code with `show`. Two `list_issues` cases in the MCP parity test check that both surfaces give `not_found`. The list filter test now lists the children of a closed, archived parent (none) instead of expecting `nonsense` to list nothing. Both new tests fail on the old code.
- Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all pass), `mara schema validate` and `mara project validate` (valid, evaluation complete), and `zban check` (no findings).
