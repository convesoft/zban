---
title: Note an enclosing project on init
status: done
type: feature
priority: high
parent: zb-9jra34
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
closed_at: 2026-11-12T13:24:14Z
---

Found by the v0.1.0 review: `zban init` inside an existing project silently creates a nested project, and commands run below it then see an empty backlog (`DES-PROJECT-LAYOUT`).

Maintainer decision: allow nesting, but `init` prints a note naming the enclosing project (and returns it in JSON). Record it in `DES-PROJECT-LAYOUT` and `DES-CLI` first.

- [x] Mara updated
- [x] CLI behaviour and JSON field, with tests

## Summary of Changes

- `DES-PROJECT-LAYOUT` says `init` works inside an existing project and names the enclosing project, the nearest directory above the new root with `.zban/config.toml`. `DES-CLI` adds the `enclosing` field to the `init` result and the note in human output. `VER-CLI` checks both, and the CLI page explains nesting to users.
- `init` returns `{path, prefix, enclosing}`. `enclosing` is the absolute path of the enclosing project's `.zban/`, like `path`, or `null` when there is none. Human output adds a `note:` line on standard output naming it. Nesting is still allowed and nothing else changes. `init` has no MCP tool, so only the CLI changed.
- `Project::enclosing_dir` shares the upward search with project discovery (`nearest_root` in `src/store.rs`).
- Tests: `init_names_the_enclosing_project` (new, through the binary: `null` and no note outside a project, the note names the enclosing `.zban/`, `enclosing` is the nearest project, and the nested project hides the enclosing project's issues), and `every_command_prints_one_json_document` now expects the `enclosing` key.
- Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all pass), and `mara schema validate` and `mara project validate`, both `valid: true` with complete evaluation over every page.
