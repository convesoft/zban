---
title: Command line with JSON output
status: done
type: feature
priority: high
parent: zb-9jra34
blocked_by:
  - zb-2b3mf7
  - zb-ramrc1
created_at: 2026-11-11T21:00:06Z
closed_at: 2026-11-12T09:33:52Z
---

Implements `DES-CLI` for `init`, `create`, `list`, `show`, `update` and `delete`, and satisfies `REQ-NO-GIT` and `REQ-DAMAGED-DATA`. Dependencies follow `ADR-DEPENDENCIES`.
The `check`, `prime` and `mcp` commands come with their own issues, and the handoff
options and fields with zb-qkj8tj.

## Done when

- [x] `VER-CLI` passes for the commands above
- [x] `VER-NO-GIT` passes

## Summary of Changes

- New core module `operations` (`DES-SHARED-CORE`): `init`, `create`, `list`, `show`, `update` and `delete` take requests holding values as given (status words, IDs in any case), so the MCP server can reuse the same validation and error codes. Results are `IssueView` (list and mutations), `IssueDetail` (show) and `Issues { issues, warnings }`, which serialize to the JSON of `DES-CLI` through `serde`/`serde_json` (named in `ADR-DEPENDENCIES`). Core types serialize as their words.
- `list` implements every filter of `REQ-LIST-FILTERS` over `Graph::issues` order. `show` and `list` warn about each skipped file: unreadable files with their first error code from `DES-CHECK`, and both files of a duplicated ID with `duplicate_id`.
- `update` applies metadata, relation and body edits to one copy of the issue, checks relations with `Graph::check`, and writes through `Project::save`, so lifecycle rules see the issue as the request leaves it and a refused or failed request writes nothing. An unchanged issue is not rewritten. A `related` link stored on the other issue is removed there, after the issue itself is written. Adding and removing one value in a request, an empty replacement text, and two options reading standard input fail with `usage`.
- New error codes `usage`, `text_not_found` and `text_ambiguous`. Malformed IDs fail with `not_found` for the issue acted on and `invalid_reference` for a relation value.
- The binary is now `src/main.rs` → `src/cli.rs` (arguments and dispatch), `src/cli/output.rs` (the shared output layer: `Reply`, `--json` versus human output, the error object, exit statuses 0/1/2, clap errors as JSON `usage` errors) and `src/cli/human.rs`. `check` plugs in as a `Command` variant returning a `Reply`, with `Reply::failed` for exit status 1.
- `Project::create_with` and `Project::delete` delegate to crate-private variants that take an already loaded `Graph`, so each operation loads the project once.
- Knowledge: `DES-CLI` now documents choices the spec left open: `path` is absolute, reverse relations are lists of IDs, `delete` returns the issue as it was, `init` returns `{path, prefix}`, and body edits apply as `--body`, replacements, `--append`. The command-line page and the README say which commands work.
- Tests: `tests/cli.rs` (18 tests for `VER-CLI` through the binary: JSON documents of every command, error objects and exit statuses, usage errors, a table producing every error code, list filters alone and combined, `-C`, outside a project, replacements, append, all-or-nothing updates, standard input, relations, broken files) and `tests/no_git.rs` (2 tests for `VER-NO-GIT`: a full create, update, close, reopen and delete cycle outside Git leaves files outside `.zban/` unchanged, and no source file starts a process). `conflict` is produced by `tests/issue_files.rs`; `invalid_handoff` comes with zb-qkj8tj. Filed zb-x59t84 to add `check` to the no-Git cycle and the JSON test once zb-yj8xta lands.

Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (96 passed: 32 unit, 18 `cli`, 5 `ids`, 11 `issue_files`, 12 `lifecycle`, 2 `no_git`, 6 `project_layout`, 10 `relations`), `mara schema validate` and `mara project validate` (`valid: true`, evaluation complete, 0 errors, 0 warnings).
