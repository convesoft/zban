---
title: Claude Code truncates the MCP guide at 4096 characters
status: done
type: bug
discovered_from: zb-wbacwg
created_at: 2026-11-12T19:40:41Z
closed_at: 2026-11-12T20:04:28Z
---

## Problem

Claude Code (2.1.322) truncates MCP server instructions at 4096 characters. Its debug log says `MCP server "zban": Server instructions truncated from 7398 to 4096 chars`, and the instructions end in `… [truncated]` partway through the Handoffs section. A Claude Code session with only the zban MCP server never sees the end of Handoffs, Finding work, or Reference, which holds the statuses, the readiness rule and the tool synopses. In one headless run the agent called `list_issues {"status": ["open"]}` and got `invalid_value`.

Before zb-wbacwg, this repository's `AGENTS.md` also made every session run `zban prime`, which delivered the whole guide (in command syntax). Now a session with the MCP server relies on the truncated instructions alone. Setups that follow the MCP recommendation in `docs/agents.mara.md` have always been affected.

Relevant items: `REQ-AGENT-GUIDE`, `DES-AGENT-GUIDE` (the guide is kept "under about 150 lines", with no character budget for MCP), `VER-AGENT-GUIDE`, `DES-MCP-TOOLS`. The Setup section of `docs/agents.mara.md` says clients such as Claude Code keep the server instructions in context.

## Decision needed

How the MCP rendering should fit a client limit. Options to weigh:

- Keep the MCP rendering under 4096 characters, for example by leaving out content that the tool schemas already carry, such as the parameter synopses.
- Order the MCP rendering so that the sections agents need most come first, and end it with a pointer to `zban prime` for the rest.
- Document the limit and recommend `zban prime` alongside MCP in Claude Code. This brings the duplication back.

## Tasks

- [x] Decide the approach and record it in Mara (`DES-AGENT-GUIDE`, `VER-AGENT-GUIDE`)
- [x] Implement it and add a test for the MCP instructions' size
- [x] Re-run the headless Claude Code check with only the zban MCP server

## Summary of Changes

- Took the maintainer's decision (first option) and recorded it in `DES-AGENT-GUIDE`: the MCP rendering stays within 3,800 characters and leaves each tool's parameters to its schema. The command-line rendering keeps the full reference. `VER-AGENT-GUIDE` now names the budget test.
- `src/guide.md`: the MCP rendering no longer has the tool synopses or the per-tool notes, which the schemas already carry. The shared wording is tighter in both renderings, and both still open with `# zban workflow`. MCP instructions went from 7,398 to 3,744 characters (83 lines); `zban prime` went from 7,075 to 5,338 characters (105 lines).
- `src/guide.rs`: `{{synopsis}}` names only the tool over MCP.
- `tests/prime.rs`: added `mcp_guide_fits_the_instructions_budget`. The removed MCP synopsis check is replaced by one that each command's options equal its tool's parameters. Both snapshots are updated.
- Dropped `discovered_from: zb-wbacwg` on this branch because zb-wbacwg is not on `main` yet (PR #44). Whichever of the two pull requests merges second keeps only `.zban/archive/zb-x2d2qt.md` and can restore the link.
- Checks run: `mara schema validate` and `mara project validate` (valid, evaluation complete), `zban check`, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`, after merging `origin/main` with #45.
- Headless check: Claude Code 2.1.322, `claude -p` in a scratch copy with `--strict-mcp-config --mcp-config` starting only the new `zban mcp`, no built-in tools, and `--debug-file`. The debug log has no "truncated" line; the released binary in the same setup logs `Server instructions truncated from 7398 to 4096 chars`. The agent called `list_issues {"status": ["in-progress"]}` and `{"ready": true}`, then created, started, ticked and closed an issue with `"in-progress"` and `"done"`, with no invalid statuses.
- CI on the pull request covers CI.
