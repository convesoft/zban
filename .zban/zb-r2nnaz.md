---
title: Review the decisions made in 1.0.0-rc.1
status: todo
type: task
tags:
  - maintainer
related:
  - zb-v90b0k
created_at: 2026-11-22T12:59:51Z
---

The orchestrator of milestone zb-v90b0k records here, one line each, the decisions made on the maintainer's behalf while the milestone ran: what was decided, which Mara items it rests on and how to undo it. Each decision is also written into the Mara item it belongs to.

This issue is for the maintainer to read before releasing `1.0.0-rc.1`. It is never dispatched to an agent.

## Decisions

Milestone:

- The milestone ships as `1.0.0-rc.1` under the npm dist-tag `next` instead of `0.2.0`: the maintainer decided this on 2026-11-22, because the v1 specification is implemented and no further enhancements are queued. It rests on `ADR-NPM-DISTRIBUTION` (the version comes from `Cargo.toml`, bumped with `pnpm nx release version <version>` in a release pull request). Undo: bump to `0.2.0` instead and rename zb-v90b0k back.

zb-a76tqw, success line of `zban check` (pull request #49):

- `No problems found.` is printed only when there are no findings at all; a check that finds only warnings prints the warnings alone. Rests on `DES-CHECK`, `VER-CHECK`. Undo: print the line whenever there are no errors, in `findings` in `src/cli/human.rs`.
- A `check --fix` that repairs every finding prints the line. Follows from `DES-CHECK`, where fixed findings are not reported. Undo: suppress the line after `--fix`.

zb-banzx5, `draft` ancestors hold back descendants (pull request #53):

- `ancestor_draft:<id>` reasons come after the reasons 0.1.0 already reported, one per `draft` ancestor, nearest first. Rests on `DES-READINESS`, `VER-RELATIONS-READY`. Undo: reorder in `Graph::not_ready` in `src/relations.rs`.
- An ancestor's status `in-progress` still holds nothing back by itself. Rests on `ADR-DRAFT-HOLDS-DESCENDANTS`. Undo: nothing to undo, this is the 0.1.0 behaviour.
- To stay inside the MCP character budget of `DES-AGENT-GUIDE` (3,787 of 3,800 characters used), the guide lost "An epic groups other issues." and the explanation in brackets after `discovered_from`; other sections say the same. Undo: restore the phrases in `src/guide.md` and cut elsewhere or raise the budget.
- The pull request title is marked breaking (`feat!`), because the change reverses the documented rule that a `draft` parent does not hold its children back. Decided by the orchestrator. It affects only the changelog.

zb-q3z2gz, docs bundled in `@convesoft/zban` (pull request #50):

- The whole `docs/` folder is copied, not a fixed list of pages. Rests on `ADR-NPM-DISTRIBUTION`. Undo: list the pages in `scripts/package-npm.mjs`.
- No separate pointer file; one sentence in `README.md` points to the bundled pages. Rests on `ADR-NPM-DISTRIBUTION`. Undo: add a pointer file to the package.
- Accepted limits: relative links that leave `docs/` do not resolve inside the package; the README in the platform packages and release archives links to a `docs/` folder they do not carry; the main package grows from about 21 kB to about 133 kB unpacked. Rests on `ADR-NPM-DISTRIBUTION`. Undo: rewrite the links when packing, or ship the pages in the archives too.
- A pull request that changes only the pages does not run the release workflow. Rests on `ADR-NPM-DISTRIBUTION`. Undo: add `docs/**` to the workflow's pull-request paths.
- The release workflow's check lists every entry of `docs/` that is not a directory, so a symlinked page, which `npm pack` leaves out, fails the release by name. Decided by the orchestrator from a review finding. Undo: go back to `find docs -type f` in `.github/workflows/release.yml`.

zb-y2kwfz, `child_counts` (pull request #54):

- The children counted are exactly those in `show`'s `children`, and a child counts as closed when it is `done` or `cancelled`, archived or not. Rests on `DES-CLI`, `VER-CLI`. Undo: change `Graph::new` in `src/relations.rs`.
- Human output is `children 2/7 closed` on a `list` line, after `[checked/total]` and before the handoff age, and `child_counts: 2/7 closed` in `show`; the agent guide does not mention the field. Rests on `DES-CLI`. Undo: change `src/cli/human.rs`.
- A child whose file is unreadable or whose ID names two files is left out of `child_counts`, as it is from `children`, while closing the parent still counts it as open. So in a damaged project the counts can read 1/1 closed while the close fails with `open_children`. The reviewer rated this a blocking mismatch; the orchestrator decided to keep the code and document the limit. Rests on `REQ-DAMAGED-DATA` (an ambiguous ID is not resolved by guessing) and `DES-CLI`. Undo: count such a child as open in `Graph::new`, so that `closed == total` always means the parent can close, at the cost that `total` no longer equals the length of `children`.

zb-wbm14r, MCP `tag` renamed to `tags` (pull request #55):

- Only `create_issue` and `list_issues` are renamed, as the issue says; `update_issue` keeps `add_tag` and `remove_tag`. Rests on `DES-MCP-TOOLS`. Undo: rename those two as well, which is another breaking change.
- `VER-AGENT-GUIDE` is unchanged and there is no separate decision record: the rename and its reason are in `DES-MCP-TOOLS`. Undo: add an ADR.
- The guide's parameter table can give a parameter a command-line option name that differs from its MCP name (`Param::option` in `src/guide.rs`). Rests on `DES-MCP-TOOLS`. Undo: none needed unless the CLI option is renamed to `--tags`.

zb-35d4qp, hint in `not_found` messages (pull request #56):

- Only an ID in this project's format, with its prefix, that names no issue gets the hint; any other value, including an ID with another project's prefix, keeps the message showing the ID format, and `invalid_reference` is unchanged. Rests on `DES-CLI`, `REQ-NO-GIT`. Undo: add the hint to `named` in `src/operations.rs` as well.

zb-esx9nv, `--not-tag` / `not_tags` (pull request #57):

- An issue that carries both a wanted and an excluded tag is left out: the exclusion wins over `--tag`/`tags` and every other filter. Rests on `REQ-LIST-FILTERS`, `DES-CLI`. Undo: change the filter in `list` in `src/operations.rs`.
- A valid tag that no issue carries is accepted and excludes nothing. Rests on `VER-CLI`. Undo: refuse unknown tags, which would need a list of known tags.
- The MCP rendering of the agent guide does not mention `not_tags` (it stays at 3,787 of 3,800 characters); the tool schema describes it, and the CLI guide's synopsis shows `[--not-tag T]...`. Rests on `DES-AGENT-GUIDE`. Undo: add it to `src/guide.md` and cut something else.
- Provisional, waiting on zb-q6q2fz: a value that is not a valid tag fails with `invalid_value` in `--tag`/`tags` as well as in `--not-tag`/`not_tags`, where 0.1.0 returned an empty list for `--tag`. The worker decided it and the orchestrator let it stand, because status, type and priority already refuse unknown values. Rests on `REQ-LIST-FILTERS`, `DES-CLI`. Undo: drop the `check_tag` call for `tags` in `list`, restore the sentence in both items and remove the `--tag` rows from the tests.

## Questions put to the maintainer

- zb-q6q2fz: should `list --tag` refuse invalid tag values, as `--not-tag` does (zb-esx9nv, pull request #57)? Provisional answer: yes.

## Done when

- [ ] The maintainer has read every line under `## Decisions` and reversed or accepted each one
