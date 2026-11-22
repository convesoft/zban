---
title: Add --not-tag / not_tags negative tag filter to list
status: done
type: feature
tags:
  - cli
  - mcp
parent: zb-v90b0k
related:
  - zb-wbm14r
  - zb-banzx5
created_at: 2026-11-22T10:56:42Z
closed_at: 2026-11-22T14:02:04Z
---

Even with `draft` cascading (zb-banzx5), orthogonal tags on active `todo` work (such as `maintainer` or `slow`) benefit from a direct exclusion filter on `list` / `list_issues`.

Add a repeatable negative tag filter:

- CLI: `--not-tag T` (repeatable) on `zban list`
- MCP: `not_tags` (list of strings) on `list_issues`, aligned with the `tags` naming in zb-wbm14r
- Semantics: excludes any issue carrying any of the given tags, composing with `--tag` and all other filters (`REQ-LIST-FILTERS`).

## Tasks

- [x] Update `REQ-LIST-FILTERS`, `DES-CLI`, `DES-MCP-TOOLS`, `DES-AGENT-GUIDE`, `VER-CLI`, `VER-SURFACE-PARITY` and `VER-AGENT-GUIDE`
- [x] Implement `--not-tag` in core `ListRequest`, CLI `list`, MCP `list_issues` (`not_tags`), and the CLI guide synopsis (keeping MCP instructions within the character budget in `DES-AGENT-GUIDE`)
- [x] Add CLI and MCP parity tests covering single and repeated `--not-tag`/`not_tags` values, composition with `--tag`/`tags` and `--ready`, and tag validation

## Done when

- [x] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [x] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [x] `zban check` reports no errors

## Summary of Changes

- `zban list --not-tag T` (repeatable) and `list_issues {"not_tags": [...]}` leave out every issue carrying any of the given tags, also when it matches `--tag`/`tags`, and compose with every other filter (`ListRequest::not_tags`, `src/operations.rs`). The CLI guide's `list` synopsis gains `[--not-tag T]...` with one clause on its meaning; the MCP rendering is unchanged at 3,787 characters, since `DES-AGENT-GUIDE` leaves parameters to the tool schemas.
- Values of `--tag` and `--not-tag` (`tags`, `not_tags`) must be valid tags, or `list` fails with `invalid_value` (new `check_tag` in `src/issue.rs`, shared with `Issue::validate`).
- Mara: `REQ-LIST-FILTERS`, `DES-CLI`, `DES-MCP-TOOLS`, `DES-AGENT-GUIDE`, `VER-CLI`, `VER-SURFACE-PARITY` and `VER-AGENT-GUIDE` updated, plus the user guide examples in `docs/cli.mara.md` and `docs/mcp.mara.md`.
- Tests: `list_not_tag_leaves_out_issues_carrying_an_excluded_tag` (`tests/cli.rs`) and `not_tags_leaves_out_what_not_tag_leaves_out` (`tests/mcp.rs`, real stdio client compared with the CLI's JSON) cover one value, repeated values, `--tag`/`tags` with an issue carrying both a wanted and an excluded tag, `--ready`, `--parent`, `--type`, `--all` and invalid tag values; both fail when the exclusion is disabled. `create_and_list_take_tags_and_not_tags_and_refuse_the_singular` checks the `not_tags` schema and refuses `not_tag`; the guide parity test maps `not_tags` to `--not-tag`.

Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all pass), `mara schema validate` (valid), `mara project validate --format json` (`valid: true`, `evaluation_complete: true`), `zban check` (no problems, built and installed binary).

## Decisions made

- A tag value in `--tag` or `--not-tag` that is not a valid tag fails with `invalid_value`, as an unknown status, type or priority does: no readable issue can carry such a tag, so the filter would silently select nothing, or exclude nothing. This also applies to `--tag`, which before matched nothing; recorded in `REQ-LIST-FILTERS` and `DES-CLI`.
- A valid tag that no issue carries is accepted and excludes nothing (`VER-CLI` test case).
- The exclusion wins over `--tag`: an issue carrying both a wanted and an excluded tag is left out; recorded in `REQ-LIST-FILTERS` and `DES-CLI`.
- The MCP rendering of the guide does not mention `not_tags`; the `list_issues` schema describes it. Recorded in `DES-AGENT-GUIDE`.

Follow-up: zb-32nknc (`update --remove-tag` accepts an invalid tag).

## Review round 1

Review of 1cc2ee2 found no P0 to P2 issues; four P3 points fixed in tests and Mara wording, with no change under `src/`:

- `tests/cli.rs`: the `--not-tag` test adds H, a ready task tagged `cli`, so ready work holds an issue each exclusion keeps and one it drops (`--ready` gives A and H, `--ready --not-tag cli` gives A, `--ready --not-tag storage` gives H). It adds rows for `--status` and `--search` with `--not-tag`.
- `tests/mcp.rs`: the parity test gives two new issues the parent E and runs its listings as a table, with new rows for `parent`, `type`, `priority` and `all` (after closing one issue) combined with `not_tags`. It also compares both projects' `.zban/` before and after the refused `tags`/`not_tags` calls.
- `VER-CLI` names the accepted unused tag ("a valid tag that no issue carries is accepted and excludes nothing") and the added combinations; `VER-SURFACE-PARITY` names the added combinations and that a refused tag changes no file.
- Filed zb-rby6kf: `DES-ISSUE-FILE` describes tags more loosely than the rule zban enforces.

Checks rerun after merging `origin/main`: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all pass), `mara schema validate` (valid), `mara project validate --format json` (`valid: true`, `evaluation_complete: true`), `zban check` (no problems). With the exclusion disabled, both `--not-tag` tests fail.
