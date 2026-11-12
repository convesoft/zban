---
title: Align the short options of zban list with DES-CLI
status: todo
type: bug
priority: low
parent: zb-9jra34
discovered_from: zb-aapz52
created_at: 2026-11-12T11:14:31Z
---

Found in the `VAL-SCOPE-REVIEW` for v0.1.0. `zban list` accepts `-s`, `-t` and `-p` as short forms of `--status`, `--type` and `--priority` (`src/cli.rs`), but `DES-CLI` gives `list` only the long forms. `create` and `update` have these short forms in `DES-CLI` too, so the implementation and the knowledge disagree only for `list`. The agent guide and the MCP tools are not affected.

- [ ] Decide whether `list` keeps the short forms, and record the answer in `DES-CLI` if it does
- [ ] Make `src/cli.rs` and `docs/cli.mara.md` agree, with a `VER-CLI` test for the result
