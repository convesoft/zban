# Using zban with agents

zban teaches agents its workflow itself, so a project needs very little setup.

> zban is not implemented yet. This page describes the agreed v1 interface.

## Setup

**With MCP** (recommended where supported): configure the [MCP server](mcp.mara.md).
The server's instructions carry the agent guide, and clients such as Claude Code
keep them in context across compaction.

**Without MCP** (CLI-only agents, CI jobs, clients that ignore server
instructions): make the agent read `zban prime`. Add this line to your
`AGENTS.md` or `CLAUDE.md`:

```markdown
**IMPORTANT**: before you do anything else, run `zban prime` and follow its output.
```

Or, in Claude Code, inject it automatically with hooks in `.claude/settings.json`:

```json
{
  "hooks": {
    "SessionStart": [{ "hooks": [{ "type": "command", "command": "zban prime" }] }],
    "PreCompact": [{ "hooks": [{ "type": "command", "command": "zban prime" }] }]
  }
}
```

Project-specific rules belong in your own `AGENTS.md`, for example "include the
issue ID in commit messages as `Refs: zb-xxxxxx`". zban does not configure them.

## Things to ask your agent

- "Are there tasks we should be tracking? Create issues for them."
- "What should we work on next?"
- "Start on zb-n938cw."
- "Break the export feature into an epic with tasks and dependencies."

## Requirement

:::mara requirement REQ-AGENT-GUIDE
:mid: 01M764FJ3Z4GXPTR8FHAF3XX7E
:title: Teach agents the zban workflow
:status: accepted
:satisfies: NEED-AGENT-ONBOARDING

zban provides an agent workflow guide through `zban prime` and through the MCP server instructions. Both come from one source text and name the operations the way the surface does: commands for the CLI, tools for MCP. The vocabulary in the guide comes from the binary itself, so it cannot disagree with the binary.
:::

## Design

:::mara design DES-AGENT-GUIDE
:mid: 01M764FJ3ZPFMKF31AGPP8X9PA
:title: Agent guide content
:status: accepted
:satisfies: REQ-AGENT-GUIDE

The guide is one template compiled into the binary and kept under about 150 lines, because it is paid for in every agent context. Its sections are:

1. **Track work in zban.** Use issues instead of ad-hoc todo lists.
2. **Before starting.** Look for an existing issue, otherwise create one; set it `in-progress`.
3. **While working.** Keep the body's checklist current. File newly discovered work as separate issues with `discovered_from` rather than widening the current one.
4. **Finishing.** Add `## Summary of Changes` and set `done`. If unchecked items remain, finish them or move them into follow-up issues; force only when the user agrees. To abandon work, add `## Reasons for Cancelling` and set `cancelled`.
5. **Committing.** Commit issue files together with the code they describe.
6. **Finding work.** Use the ready list and offer the top candidates; show an issue before starting it.
7. **Reference.** Types, statuses, priorities, relations, the readiness rule, and the operations with their key options.

Project-specific conventions are deliberately left out; projects add them in their own agent instructions.
:::

## Scenario

:::mara scenario SCN-AGENT-SESSION
:mid: 01M764FJ3ZRJ3Q9GY8PMXWG7PF
:title: An agent works through an issue
:status: accepted

A developer starts a coding-agent session in a repository that uses zban. The agent receives the zban guide through MCP instructions or a `zban prime` hook. Asked what to do next, it lists ready work and proposes the highest-priority issue. With the developer's agreement it sets the issue `in-progress`, reads its body and the documents it references, and does the work while checking off task items. On noticing an unrelated bug, it files a bug with `discovered_from` pointing at the current issue and carries on. At the end it appends a summary, sets the issue `done`, which archives it, and commits the code together with the changed issue files. A later session, or a different agent, can see what was done and pick up the bug.
:::
