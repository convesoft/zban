---
title: Rename MCP tag parameter to tags on create_issue and list_issues
status: done
type: task
tags:
  - mcp
parent: zb-v90b0k
created_at: 2026-11-22T10:56:29Z
closed_at: 2026-11-22T13:34:32Z
---

Issue frontmatter and JSON output name the field `tags`, while MCP `create_issue` and `list_issues` currently accept `tag` (mirroring repeatable CLI `--tag`). Across T3 Code sessions, agents repeatedly guessed `tags` on `create_issue` and hit `unknown field 'tags'`.

Make a clean breaking rename in MCP (no alias): rename `tag` to `tags` on `create_issue` and `list_issues` (keeping CLI `--tag` singular for repeatable flags).

## Tasks

- [x] Update `DES-MCP-TOOLS`, `docs/mcp.mara.md`, `VER-SURFACE-PARITY` and `VER-AGENT-GUIDE` (if the guide/parity test checks parameter names) to document `tags` in MCP
- [x] Rename `tag` to `tags` in `CreateArgs` and `ListArgs` in `src/mcp.rs` without adding a `tag` alias
- [x] Update MCP and guide parity tests (`VER-SURFACE-PARITY`, `VER-AGENT-GUIDE`)

## Done when

- [x] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [x] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [x] `zban check` reports no errors

## Summary of Changes

- MCP `create_issue` and `list_issues` take `tags` instead of `tag`, with no alias: a request that sends `tag` fails with `usage` (unknown field), like any unknown parameter. The CLI keeps the repeatable `--tag`; `update_issue` keeps `add_tag` and `remove_tag`.
- `DES-MCP-TOOLS` names the one renamed option and why; the user guide in `docs/mcp.mara.md` says so and shows a `list_issues {"tags": [...]}` example; `VER-SURFACE-PARITY` now requires `tags` and the refusal of `tag`.
- `src/guide.rs`: a parameter can name its command-line option where it differs from the MCP name (`many("tags", "T").option("tag")`), so the CLI guide still renders `--tag T...`. Both guide renderings and their snapshots are unchanged, so the MCP guide stays at its previous size.
- Tests: the MCP parity flow uses `tags`; the new `create_and_list_take_tags_and_refuse_tag` checks the schemas through a real stdio client and that `tag` is refused without writing; the guide parity test in `tests/prime.rs` maps `tags` to `--tag`; a unit test renders `list tags=T` on both surfaces.

Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all pass), `mara schema validate` (valid), `mara project validate --format json` (`valid: true`, `evaluation_complete: true`), `zban check` (no problems).

## Decisions made

- `VER-AGENT-GUIDE` needed no new wording. Its parity test, `guide_lists_every_option_of_each_command` in `tests/prime.rs`, does check parameter names (every MCP parameter must match a listed CLI option) and was updated to map `tags` to `--tag`; the item's criteria still hold as written, and neither rendering of the guide names the parameter in an MCP example.
- The exception is recorded in `DES-MCP-TOOLS` rather than in a separate decision record, since it is one renamed parameter.


## Review follow-up

- The MCP parity flow in `tests/mcp.rs` now lists with `tags` and no `parent` over five open issues (the fixture's untagged `A`, `B` and epic `E`, the created child of `E` tagged `storage`, and an untagged created issue). It asserts that the result, equal to the CLI's `--tag` JSON, holds only the tagged issue, so it would fail if MCP ignored `tags`. Test-only; no change under `src/`.
- The `VER-AGENT-GUIDE` decision above now says that the guide parity test checks parameter names and was updated.
- Checks rerun: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `mara schema validate`, `mara project validate --format json` (`valid: true`, `evaluation_complete: true`) and `zban check`.
