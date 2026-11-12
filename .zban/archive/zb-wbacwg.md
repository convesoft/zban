---
title: Recommend a zban prime line that does not duplicate the MCP guide
status: done
type: task
created_at: 2026-11-12T19:34:14Z
closed_at: 2026-11-12T19:41:11Z
---

## Problem

The Setup section of `docs/agents.mara.md` recommends an unconditional line for `AGENTS.md` or `CLAUDE.md`: "**IMPORTANT**: before you do anything else, run `zban prime` and follow its output." This repository's `AGENTS.md` uses it while `.mcp.json` also starts the zban MCP server. A Claude Code session then gets the full guide twice: once as MCP server instructions (tool syntax) and once from `zban prime` (command syntax). That wastes context and blurs tools and commands.

## Decision (maintainer)

Recommend a conditional line whose condition the agent can check by looking at its own context:

```markdown
**IMPORTANT**: this project tracks its work with zban. If your instructions already
contain the zban workflow guide (it starts with `# zban workflow`), for example from
the zban MCP server, follow it and use the zban tools. Otherwise, before anything
else, run `zban prime` and follow its output.
```

The wording may be polished, keeping the checkable `# zban workflow` marker, the MCP tools preferred when the guide came from MCP, and `zban prime` as the fallback.

Cites `REQ-AGENT-GUIDE`, `DES-AGENT-GUIDE`, `VER-AGENT-GUIDE`.

## Tasks

- [x] Check `REQ-AGENT-GUIDE`, `DES-AGENT-GUIDE`, `VER-AGENT-GUIDE` and `SCN-AGENT-SESSION` for wording that implies always running `zban prime` alongside MCP, and update it if so
- [x] Recommend the new line in the Setup section of `docs/agents.mara.md`, and update `README.md` and `docs/mcp.mara.md` where they repeat the old line
- [x] Use the new line in this repository's `AGENTS.md`, with the zban MCP server first and the command line as the fallback
- [x] Update `this_repository_loads_the_guide` in `tests/prime.rs` to compare the whole recommended block
- [x] Fresh-session check in a scratch copy with headless `claude`: with MCP, no `zban prime`; without MCP, `zban prime` before answering (each at least twice)
- [x] `mara schema validate`, `mara project validate`, `zban check`, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`
- [x] CI on the pull request

## Summary of Changes

- `docs/agents.mara.md` Setup recommends the conditional line from the decision, unchanged, for every project. Agents with the MCP server skip `zban prime`. The hooks are now described as an alternative only for setups without MCP, because a hook always adds the guide. `README.md` and `docs/mcp.mara.md` did not repeat the old line, so they are unchanged.
- Mara: `REQ-AGENT-GUIDE` now says an agent can tell whether it already has the guide. `DES-AGENT-GUIDE` records that both renderings open with `# zban workflow` and that the recommended line names this heading. `VER-AGENT-GUIDE` checks it. `SCN-AGENT-SESSION` and `ADR-CLI-AND-MCP` already said "MCP or `zban prime`" and needed no change.
- `AGENTS.md` uses the new line and puts the MCP server first, with the command line as the fallback. Its rule on creating issues now names `zban create` and the `create_issue` tool.
- `tests/prime.rs`: `this_repository_loads_the_guide` compares the whole fenced block from `docs/agents.mara.md` with a paragraph of `AGENTS.md`. Changing one word in `AGENTS.md` made it fail. The new test `setup_line_names_the_heading_the_guide_starts_with` checks that both renderings start with `# zban workflow`, that the short MCP note does not, and that the recommended line names the heading and `zban prime`.
- Fresh sessions: headless `claude -p "What is the next ready issue in this project? Answer briefly." --output-format stream-json --strict-mcp-config` in a scratch copy of the repository.
  - With `zban mcp` (Haiku twice, Sonnet once): no session ran Bash or `zban prime`. Each used `mcp__zban__list_issues` with `ready: true` and `directory`, and one also used `check_issues`.
  - With an empty MCP config (Haiku twice, Sonnet once): each session ran `zban prime` through Bash first, then `zban list --ready`.
  - Baseline with the old line and MCP (Haiku): the session ran `zban prime | head -80`, `zban --help` and `zban list` through Bash and never used the zban tools.
  - The wording did not need changing.
- Checks run: `mara schema validate`, `mara project validate` (valid, evaluation complete, 0 errors and 0 warnings), `zban check`, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all pass). CI on the pull request covers the last item.
- Found: Claude Code truncates the MCP instructions at 4096 characters (7398 sent). Tracked as zb-x2d2qt, which its own pull request adds and fixes before this one merges.
