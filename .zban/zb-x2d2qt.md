---
title: Claude Code truncates the MCP guide at 4096 characters
status: draft
type: bug
discovered_from: zb-wbacwg
created_at: 2026-11-12T19:40:41Z
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

- [ ] Decide the approach and record it in Mara (`DES-AGENT-GUIDE`, `VER-AGENT-GUIDE`)
- [ ] Implement it and add a test for the MCP instructions' size
