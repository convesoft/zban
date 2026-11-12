# What zban is

zban is a file-based issue tracker for you and your coding agents. Issues are
Markdown files in your repository, managed through a CLI and an MCP server.

Coding agents lose track of work between sessions, after context compaction and
when work passes from one agent to another. Ad-hoc todo lists and chat history
don't survive any of that. zban gives agents and humans one shared, versioned
backlog that lives next to the code:

- **Plain files.** Each issue is one Markdown file in `.zban/`. You can read,
  edit, grep and review issues without zban installed.
- **Branches and merges like code.** zban never talks to a server and never
  runs Git. Issue changes travel with your commits.
- **Built for agents.** Every command has JSON output, the same operations are
  available as MCP tools, and zban teaches agents its workflow itself.
- **Small on purpose.** A fixed vocabulary, a short command set and no hidden
  state.

## What v1 includes

Issues with a fixed set of types, statuses and priorities; parent, blocker,
related and discovered-from relations; a computed list of ready work; automatic
archiving of closed issues; handoff notes for continuing work in a new session;
an integrity check; the CLI with JSON output; the MCP server; and the agent guide. See [Issues](issues.mara.md), [Command line](cli.mara.md),
[MCP server](mcp.mara.md) and [Using zban with agents](agents.mara.md).

## What zban deliberately does not do

zban does not run agents, manage worktrees, lock or claim issues across
worktrees, commit for you, serve a web or terminal UI, offer a query language,
templates or custom fields, or let projects redefine statuses and types. The
reasons are recorded in [[ADR-SMALL-SCOPE]] and [[ADR-NO-CLAIMS]]. A capability
outside this list needs a recorded need and decision before it is built.

## Goals

:::mara goal GOAL-REPO-ISSUES
:mid: 01M764FJ3Z3R4W2ZGMT5P434J0
:title: Keep issue tracking in the repository
:status: accepted

Issues live next to the code they describe and are versioned, branched, reviewed and merged with it. Tracking work needs no service, account or database. Anyone with a checkout has the backlog and its history.
:::

:::mara goal GOAL-AGENT-WORKFLOW
:mid: 01M764FJ3Z3VPB3JGPG1P94AXW
:title: Let coding agents manage their own work
:status: accepted

Coding agents find what to work on, record progress and outcomes, and file follow-up work through the tracker instead of chat history or ad-hoc todo lists. Work then survives session ends, context compaction and handoffs between agents and humans.
:::

:::mara goal GOAL-SMALL-TOOL
:mid: 01M764FJ3Z3P69KAH9C5ZFPZWD
:title: Stay understandable and predictable
:status: accepted

One person can understand the whole of zban, and an agent can predict what each operation does from the agent guide alone. This comes from a fixed data model, a short command set, a file format readable without zban, and no hidden state or background activity. Every added capability spends some of this, so it is added only for a demonstrated need.
:::

## Needs

:::mara need NEED-READABLE-FILES
:mid: 01M764FJ3Z2306WVCH858EK24P
:title: Read and edit issues without special tools
:status: accepted
:contributes_to: GOAL-REPO-ISSUES
:contributes_to: GOAL-SMALL-TOOL

Humans and agents can read, search, review and hand-edit issues as ordinary text files, with or without zban installed.
:::

:::mara need NEED-BRANCH-SAFE
:mid: 01M764FJ3Z4B0WDVBR7JXWBR5B
:title: Change issues on parallel branches
:status: accepted
:contributes_to: GOAL-REPO-ISSUES

People and agents working on separate branches or worktrees can create and update issues independently, and those changes merge through ordinary Git workflows.
:::

:::mara need NEED-ORGANIZE
:mid: 01M764FJ3ZG12XTG21J7XKEQFC
:title: Structure and prioritize work
:status: accepted
:contributes_to: GOAL-AGENT-WORKFLOW

Work can be classified, grouped into larger pieces, ordered by importance, and connected by dependencies and by where it was discovered.
:::

:::mara need NEED-FIND-NEXT
:mid: 01M764FJ3ZKQYBEXD73381XSPQ
:title: Find what to work on next
:status: accepted
:contributes_to: GOAL-AGENT-WORKFLOW

An agent or human can quickly see which issues can be started now, which are in progress, and why the others are waiting.
:::

:::mara need NEED-TRACK-PROGRESS
:mid: 01M764FJ3Z6S4M6YKX2NXB78TK
:title: Record progress and outcomes
:status: accepted
:contributes_to: GOAL-AGENT-WORKFLOW

Agents can create issues, record what is in progress, keep a task checklist current, and record what was done or why work was abandoned.
:::

:::mara need NEED-AGENT-ONBOARDING
:mid: 01M764FJ3Z8YQQEKPG1YMKZK0P
:title: Agents learn the workflow from the tool
:status: accepted
:contributes_to: GOAL-AGENT-WORKFLOW

An agent learns how to use zban, and the expected work discipline, from zban itself, whether it reaches zban through MCP or the command line, without the project writing its own instructions.
:::

:::mara need NEED-SESSION-CONTINUITY
:mid: 01M777QQGMVA28TW42QWJCGFX0
:title: Continue unfinished work in a new session
:status: accepted
:contributes_to: GOAL-AGENT-WORKFLOW

A developer can end an agent session with work unfinished and continue it in a new session, with the same or another agent, from the current state, the next steps and the approaches already rejected, without replaying the old conversation.
:::

:::mara need NEED-PROJECT-MEMORY
:mid: 01M764FJ3Z76TNN9PYF84SMRNB
:title: Keep finished work as project memory
:status: accepted
:contributes_to: GOAL-AGENT-WORKFLOW

Finished and abandoned issues stay available as context about past work, without crowding the list of active work.
:::

:::mara need NEED-INTEGRITY
:mid: 01M764FJ3Z8BV1YNJN9TA5QQDA
:title: Trust the issue data
:status: accepted
:contributes_to: GOAL-REPO-ISSUES

Users can detect and repair inconsistent issue data introduced by hand edits, interrupted writes or merges.
:::

## Vocabulary

:::mara term TERM-ISSUE
:mid: 01M764FJ3Z16YDB0AKKDJWA6K0
:title: Issue
:status: accepted
:aliases: ticket

One tracked unit of work, stored as one file in the project's `.zban/` directory. Every issue has a type: epic, feature, bug or task. The file format is defined in [[DES-ISSUE-FILE]].
:::

:::mara term TERM-READY
:mid: 01M764FJ3ZM923FNVJ8RAPBF9T
:title: Ready issue
:status: accepted
:aliases: actionable issue

An issue that can be started now. Readiness is computed from status, type and relations whenever it is needed and is never stored; the rule is [[DES-READINESS]].
:::

:::mara term TERM-CLOSED
:mid: 01M764FJ3ZY93Z24QC6JRBVV5S
:title: Closed issue
:status: accepted

An issue with status `done` or `cancelled`. Closed issues are kept in `.zban/archive/` as project memory.
:::
