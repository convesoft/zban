# Agent instructions

## Project

zban is a file-based issue tracker for humans and coding agents. Issues are Markdown
files in the repository, managed through a Rust CLI and an MCP server. The v1
specification is agreed; implementation is starting.

zban replaces two references: [beans](https://github.com/hmans/beans), whose small
core it follows, and an earlier attempt (`convesoft/work`) that drowned in
coordination machinery. Staying small is an explicit goal (`GOAL-SMALL-TOOL`).

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

Until the zban CLI works (issue `zb-as3s58`), edit issue files by hand, exactly
as `DES-ISSUE-FILE`, `DES-ISSUE-ID` and `DES-LIFECYCLE` specify.

1. **Pick.** Choose a ready issue as defined in `DES-READINESS`, or the one the
   user names. Set `status: in-progress`.
2. **Read.** Read the cited Mara items before writing code.
3. **Work.** Keep the body's checklist current.
4. **Finish.** Append `## Summary of Changes`, including the checks you actually
   ran. Set `status: done`, add `closed_at` (RFC 3339 UTC), and `git mv` the file
   into `.zban/archive/`. Mark an issue done only when its `Done when` checks
   pass.
5. **Record new work.** File newly discovered work as a new issue with
   `discovered_from`, rather than widening the current one. Generate an ID with:
   `python3 -c "import secrets;a='0123456789abcdefghjkmnpqrstvwxyz';print('zb-'+''.join(secrets.choice(a) for _ in range(6)))"`
   and confirm it is unused under `.zban/`.
6. **Commit.** Commit issue files together with the code they describe.

## Scope

`ADR-SMALL-SCOPE` lists what v1 excludes: claims, cross-worktree state, Git
automation, agent execution, UIs, query languages, templates, custom fields and
more. Do not add such capabilities, even when a neighbouring tool has them. If
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
- Before reporting work as done, run `cargo fmt --all -- --check`,
  `cargo clippy --all-targets -- -D warnings` and `cargo test`. Report which
  checks ran.
- Write tests for behaviour named by the cited `VER-` items. Prefer exercising the
  real binary or MCP protocol over mocks for those checks.

## Commits

Use Conventional Commits (`<type>(<scope>): <imperative summary>`), and add a
`Refs: zb-xxxxxx` footer naming the issue(s) involved. Mark breaking changes
with `!`.
