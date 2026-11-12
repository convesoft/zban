# Agent instructions

**IMPORTANT**: before you do anything else, run `zban prime` and follow its output.

Use the `zban` on your `PATH`, installed from a GitHub release (see
`README.md`) or with `cargo install --path . --locked` from this checkout, and
the zban MCP server configured in `.mcp.json`, which starts `zban mcp` from
`PATH`. In a worktree, pass `directory` on every MCP call or start the session
in that worktree. Use `cargo run -q -- <command>` only to try out an unreleased
change to zban itself that the task depends on, such as a new option or a
changed issue format; the installed `zban` reads and writes the same issue
format as `main`. The rules below add to the guide.

## Project

zban is a file-based issue tracker for humans and coding agents. Issues are Markdown
files in the repository, managed through a Rust CLI and an MCP server. The v1
specification is implemented and released as v0.1.0.

Staying small is an explicit goal (`GOAL-SMALL-TOOL`).

## Where things live

| Path | Owns |
| --- | --- |
| `docs/*.mara.md` | Goals, needs, terms, scenarios, requirements, design, decisions, risks and verification as Mara items, plus the user guide |
| `.mara/` | Mara schema, rules and policies. Do not change these as a side effect of other work. |
| `.zban/` | This repository's backlog, in zban's own issue format |
| `src/` | The implementation |

## Knowledge versus backlog

- Durable knowledge lives only in Mara: what zban must do and why, how it is
  designed, and how that is checked.
- Issues in `.zban/` say what to do next and cite Mara IDs such as `DES-READINESS` or
  `VER-CLI`. Never copy requirement or design text into an issue. An issue body
  holds only task-specific steps, notes and the result summary.
- If work reveals a new or changed product decision, update the Mara items first
  (in the same change), then cite them. Do not leave decisions only in an issue,
  a commit message or a PR.
- Read a cited item with the Mara MCP `get` tool or `mara get <ID>`, and follow
  `related` for its context.

## Working on an issue

The guide covers finding, starting, updating and closing issues. In this
repository, also:

- Read the Mara items an issue cites before writing code.
- Set an issue `done` only when its `Done when` checks actually pass, and name
  the checks you ran in `## Summary of Changes`.
- Create issues only with `zban create`, which generates an ID that is unused in
  `.zban/` and `.zban/archive/`. Never choose an ID yourself.
- Keep to one issue per pull request: file newly discovered work as a new issue
  with `--discovered-from` rather than widening the current one.

## Scope

`ADR-SMALL-SCOPE` lists what v1 excludes: claims, cross-worktree state, Git
automation, agent execution, UIs, query languages, templates, custom fields and
more. Do not add such capabilities, even when similar tools have them. If
something seems necessary, propose a draft need and decision in Mara and ask the
user before implementing it.

## Mara

- Use the Mara skill and MCP tools, or the `mara` CLI, with this directory as the
  project root. Inspect `schema_get` guidance before choosing a flavour or relation.
- After changing knowledge, run schema and project validation and require
  `valid: true` with complete evaluation. The engineering policies require every
  accepted requirement to have an accepted basis and an accepted verification,
  and every accepted design to have an accepted basis.
- `accepted` means agreed, not implemented or tested. Cite actual test runs as
  evidence of completion.
- Link implementing code with comment markers on the declaration, for example
  `// @mara implements DES-READINESS` (see `.mara/README.md`).

## Documentation layout

Each topic page in `docs/` serves users and engineering at once. It opens with
user-facing guidance (examples and tables) and then has `## Requirements`,
`## Design` and, where useful, `## Scenario` sections holding the Mara items. The
guidance explains and the items are normative; keep them consistent and avoid
restating item text at length. Goals, needs and terms belong in `product.mara.md`,
decisions and risks in `decisions.mara.md`, and checks in `verification.mara.md`.
Add a page only for a new user-visible topic, and list it in `docs/index.mara.md`.
Remove a page's "not implemented yet" note once the feature works.

## Rust

- Keep the core library and thin CLI/MCP adapters split described in `DES-SHARED-CORE`.
  zban starts no external processes (`REQ-NO-GIT`).
- Use the crates named in `ADR-DEPENDENCIES`. Adding another runtime dependency
  needs a recorded decision first.
- Before reporting work as done, run `cargo fmt --all -- --check`,
  `cargo clippy --all-targets -- -D warnings` and `cargo test`. Report which
  checks ran.
- Write tests for behaviour named by the cited `VER-` items. Prefer exercising the
  real binary or MCP protocol over mocks for those checks.

## Commits

Use Conventional Commits without a scope (`<type>: <imperative summary>`), and add a
`Refs: zb-xxxxxx` footer naming the issue(s) involved. Mark breaking changes
with `!`.

Pull requests are squash-merged, one issue per pull request. The title follows
the commit format above and becomes the commit subject on `main`. The
description becomes the commit body: fill in `.github/pull_request_template.md`,
tick only the checks you actually ran, link follow-up issues under known
limitations, and complete the `Refs:` footer. Write each paragraph of the
description on a single line: GitHub wraps the squash commit body itself, and
manual line breaks leave it ragged.
