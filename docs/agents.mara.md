# Using zban with agents

zban teaches agents its workflow itself, so a project needs very little setup.

## Setup

**With MCP** (recommended where supported): configure the [MCP server](mcp.mara.md).
The server's instructions carry the agent guide, and clients such as Claude Code
keep them in context across compaction. Started outside a zban project, the
server sends only a short note instead of the full guide.

**In your agent instructions**: add this line to your `AGENTS.md` or `CLAUDE.md`,
so that agents without the guide read it from `zban prime`:

```markdown
**IMPORTANT**: this project tracks its work with zban. If your instructions already
contain the zban workflow guide (it starts with `# zban workflow`), for example from
the zban MCP server, follow it and use the zban tools. Otherwise, before anything
else, run `zban prime` and follow its output.
```

An agent with the MCP server finds the guide in its instructions and skips
`zban prime`, so it does not read the guide twice. CLI-only agents, CI jobs and
clients that ignore server instructions run `zban prime`.

**Without MCP**, Claude Code can instead inject the guide with hooks in
`.claude/settings.json`:

```json
{
  "hooks": {
    "SessionStart": [{ "hooks": [{ "type": "command", "command": "zban prime" }] }],
    "PreCompact": [{ "hooks": [{ "type": "command", "command": "zban prime" }] }]
  }
}
```

`zban prime` prints nothing outside a zban project, so these hooks can also go in
your user-level settings. A hook adds the guide even when the MCP server has sent
it, so do not combine the hooks with the server.

> **Known limitation:** zban decides whether to deliver the guide from the
> directory where the session starts. If you start your agent in a folder above
> the project, such as a folder holding several repositories, or the top of a
> monorepo whose `.zban/` lives in a subfolder, the agent gets no guide from
> `zban prime`, and only the short note over MCP. Start the session inside the
> project, or ask the agent to run `zban prime -C <project>`.

Project-specific rules belong in your own `AGENTS.md`, for example "include the
issue ID in commit messages as `Refs: zb-xxxxxx`". zban does not configure them.

## Things to ask your agent

- "Are there tasks we should be tracking? Create issues for them."
- "What should we work on next?"
- "Start on zb-n938cw."
- "Break the export feature into an epic with tasks and dependencies."
- "Write a handoff for zb-n938cw."
- "Continue where we left off on zb-n938cw."

## Handoffs

A handoff is a short note on an issue that lets a new session continue
unfinished work without rereading the old conversation. Ask your agent to write
one while it still has the conversation, for example before you clear a long
session and start fresh instead of relying on compaction. The agent writes it
with `update --handoff`, which replaces any earlier note, so the issue always
holds the current state; Git keeps the history.

```markdown
## Handoff (2026-11-12T18:40:00Z)

- At 5b93272 on feat/parser, working tree clean.
- Done: reader and canonical writer.
- Next: preserve unknown keys.
- Uncertain: flow-style lists written by hand.
- Rejected: round-tripping through a generic YAML value, which loses key order.
- Files: src/store.rs, tests/files.rs
```

zban stamps the heading with the time the note was written, and `list` and
`show` report its age. zban cannot see Git, so the agent records the commit and
branch itself. A new session reads a handoff only when you ask it to continue;
it then checks the note against the repository before acting on it, because a
stale note can mislead. When the issue is done, the agent replaces the handoff
with `## Summary of Changes` and keeps the approaches it rejected.

zban does not load handoffs into new sessions by itself: notes belong to
issues, and zban cannot tell which issue a new session is about.

## Requirements

:::mara requirement REQ-AGENT-GUIDE
:mid: 01M764FJ3Z4GXPTR8FHAF3XX7E
:title: Teach agents the zban workflow
:status: accepted
:satisfies: NEED-AGENT-ONBOARDING

zban provides an agent workflow guide through `zban prime` and through the MCP server instructions. Both deliver the full guide only when a project is found, so sessions outside zban projects carry almost none of it. Both come from one source text and name the operations the way the surface does: commands for the CLI, tools for MCP. An agent can tell from its own context whether it already has the guide, so a project's agent instructions can load it through `zban prime` only where it is missing. The vocabulary in the guide comes from the binary itself, so it cannot disagree with the binary.
:::

:::mara requirement REQ-HANDOFF
:mid: 01M777QQGMEKBF23H6FN17YY39
:title: Hand off unfinished work through the issue
:status: accepted
:satisfies: NEED-SESSION-CONTINUITY

An issue can hold one handoff note in its body. An update writes the note, replacing any previous one, or removes it, and zban records when the note was written. Showing an issue returns the note and its time, and listing reports the time for each issue that has one. zban never writes, removes or delivers a handoff note on its own; the guide tells agents to write and read notes when the developer asks.
:::

## Design

:::mara design DES-AGENT-GUIDE
:mid: 01M764FJ3ZPFMKF31AGPP8X9PA
:title: Agent guide content
:status: accepted
:satisfies: REQ-AGENT-GUIDE
:mitigates: RISK-SESSION-ABOVE-PROJECT

The guide is one template compiled into the binary and kept under about 150 lines, because it is paid for in every agent context. The MCP rendering also stays within 3,800 characters, a margin under the 4,096 at which Claude Code truncates server instructions. It leaves out what the tool schemas carry, such as each tool's parameters, and both renderings share tight wording rather than diverging text. Its sections are:

1. **Track work in zban.** Use issues instead of ad-hoc todo lists.
2. **Before starting.** Look for an existing issue, otherwise create one; set it `in-progress`.
3. **While working.** Keep the body's checklist current. File newly discovered work as separate issues with `discovered_from` rather than widening the current one. On a `conflict` error, show the issue again and retry.
4. **Finishing.** Add `## Summary of Changes` and set `done`. If unchecked items or open children remain, finish them, close them or move them into follow-up issues; force only when the user agrees. To abandon work, add `## Reasons for Cancelling` and set `cancelled`.
5. **Committing.** Commit issue files together with the code they describe.
6. **Handoffs.** Writing and resuming handoff notes, as in [[DES-HANDOFF]].
7. **Finding work.** Use the ready list and offer the top candidates; show an issue before starting it.
8. **Reference.** Types, statuses, priorities, relations and the readiness rule. The command-line rendering also lists every command with its options; the MCP rendering leaves the tools' parameters to their schemas.

The MCP rendering also tells agents to pass `directory` on every call when they work in a worktree or outside the directory the server started in.

Delivery depends on the project found where the surface starts: from the working directory or `-C` for `prime`, and from the server's `-C` or working directory at startup for MCP. With a project, both deliver the full guide; a project whose configuration is invalid still counts, because the guide does not depend on it. Without one, `prime` prints nothing, and the MCP server sends a short note of a few lines instead: zban tracks issues in `.zban/` directories, no project was found where the server started, tools take `directory`, and `zban prime -C <dir>` prints the full guide.

Both renderings open with the heading `# zban workflow`, and the line that the setup guidance recommends for a project's agent instructions names it: an agent whose instructions already hold the guide, for example from the MCP server, follows it and uses the tools, and any other agent runs `zban prime`. An agent with both the server and the line therefore reads the guide once, with the operations of the surface it has.

Project-specific conventions are deliberately left out; projects add them in their own agent instructions.
:::

:::mara design DES-HANDOFF
:mid: 01M777QQGMQNB89NDDMPGRTTFA
:title: Handoff section
:status: accepted
:satisfies: REQ-HANDOFF

The handoff is the body section whose heading line is `## Handoff` or `## Handoff (<timestamp>)`, with a timestamp as in [[DES-ISSUE-FILE]]. It runs to the next level 1 or 2 heading or to the end of the body. Headings inside fenced code blocks, as [[DES-LIFECYCLE]] defines them, are ignored: a fence opens only when indented by at most three columns more than the content of the list item holding it, or than the line start outside list items.

- **Writing.** `update --handoff TEXT|-` writes `## Handoff (<now>)`, a blank line and the text. It replaces an existing section in place, or appends the section after a blank line. Text that contains a level 1 or 2 heading outside a fenced code block is rejected, because it would end the section early, and so is text that leaves a fenced code block open, because it would carry the section past the next heading. `update --no-handoff` removes the section, together with the blank lines before it when it ends the body. Both fail when the body has more than one handoff section, which must then be fixed by hand. Otherwise, bytes outside the section are unchanged. Writing also fails, changing nothing, when the new body would not give back the note just written, for example because the body leaves a fenced code block open before it.
- **Reading.** In JSON, `show` adds `handoff` as `{written_at, text}`, or `null` without a section, and `list` adds `handoff_at`. A heading without a timestamp gives `written_at: null`. With several sections, reading uses the first. Human output shows the age of the note.
- **Guide.** The guide section on handoffs says: write a handoff only when the developer asks, while the conversation is still available, and overwrite rather than append. Record the commit, the branch and whether the working tree is clean; what is done; what is next; what is blocked or uncertain; approaches tried and rejected; and the files touched. Use plain bullets, not task items, and no diffs. Resume only when asked to continue: show the matching in-progress issue, asking the developer if several could match, and check the note against the repository before acting on it. On `done`, replace the handoff with `## Summary of Changes` and keep the rejected approaches.

`prime` and the MCP instructions never include handoff notes ([[ADR-HANDOFF-ON-ISSUE]]).
:::

## Scenario

:::mara scenario SCN-AGENT-SESSION
:mid: 01M764FJ3ZRJ3Q9GY8PMXWG7PF
:title: An agent works through an issue
:status: accepted

A developer starts a coding-agent session in a repository that uses zban. The agent receives the zban guide through MCP instructions or a `zban prime` hook. Asked what to do next, it lists ready work and proposes the highest-priority issue. With the developer's agreement it sets the issue `in-progress`, reads its body and the documents it references, and does the work while checking off task items. On noticing an unrelated bug, it files a bug with `discovered_from` pointing at the current issue and carries on. At the end it appends a summary, sets the issue `done`, which archives it, and commits the code together with the changed issue files. A later session, or a different agent, can see what was done and pick up the bug.
:::
