---
title: Align the short options of zban list with DES-CLI
status: done
type: bug
priority: low
parent: zb-9jra34
discovered_from: zb-aapz52
created_at: 2026-11-12T11:14:31Z
closed_at: 2026-11-12T11:41:32Z
---

Found in the `VAL-SCOPE-REVIEW` for v0.1.0. `zban list` accepts `-s`, `-t` and `-p` as short forms of `--status`, `--type` and `--priority` (`src/cli.rs`), but `DES-CLI` gives `list` only the long forms. `create` and `update` have these short forms in `DES-CLI` too, so the implementation and the knowledge disagree only for `list`. The agent guide and the MCP tools are not affected.

- [x] Decide whether `list` keeps the short forms, and record the answer in `DES-CLI` if it does
- [x] Make `src/cli.rs` and `docs/cli.mara.md` agree, with a `VER-CLI` test for the result

## Summary of Changes

Maintainer decision: `list` keeps `-s`, `-t` and `-p`, consistent with `create` and `update`. `DES-CLI` now gives `list` the short forms and states that `-s`, `-t` and `-p` are short for `--status`, `--type` and `--priority` in `create`, `list` and `update`. The user guidance in `docs/cli.mara.md` says the same, with an example. `VER-CLI` now names the check. `src/cli.rs` already accepted the short forms, so the code is unchanged. The new test `list_short_options_select_what_the_long_options_select` in `tests/cli.rs` runs the real binary and checks that the short and long forms select the same issues, alone, repeated and combined. Removing `-s` from `list` makes the test fail.

Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all pass), and `mara schema validate` plus `mara project validate` (`valid: true`, evaluation complete, no diagnostics).
