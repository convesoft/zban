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
reasons are recorded in [ADR-SMALL-SCOPE](decisions.mara.md#01M764FJ3ZAAC9D0V2C1WQY9CB) and [ADR-NO-CLAIMS](decisions.mara.md#01M764FJ3ZDTFAZ4GG9A8BJ03S). A capability
outside this list needs a recorded need and decision before it is built.

## Goals

---

<a id="01M764FJ3Z3R4W2ZGMT5P434J0"></a>

### Keep issue tracking in the repository

| Type | Goal |
| --- | --- |
| ID | GOAL-REPO-ISSUES |
| Status | Accepted |

Issues live next to the code they describe and are versioned, branched, reviewed and merged with it. Tracking work needs no service, account or database. Anyone with a checkout has the backlog and its history.

---

<a id="01M764FJ3Z3VPB3JGPG1P94AXW"></a>

### Let coding agents manage their own work

| Type | Goal |
| --- | --- |
| ID | GOAL-AGENT-WORKFLOW |
| Status | Accepted |

Coding agents find what to work on, record progress and outcomes, and file follow-up work through the tracker instead of chat history or ad-hoc todo lists. Work then survives session ends, context compaction and handoffs between agents and humans.

---

<a id="01M764FJ3Z3P69KAH9C5ZFPZWD"></a>

### Stay understandable and predictable

| Type | Goal |
| --- | --- |
| ID | GOAL-SMALL-TOOL |
| Status | Accepted |

One person can understand the whole of zban, and an agent can predict what each operation does from the agent guide alone. This comes from a fixed data model, a short command set, a file format readable without zban, and no hidden state or background activity. Every added capability spends some of this, so it is added only for a demonstrated need.

---

## Needs

---

<a id="01M764FJ3Z2306WVCH858EK24P"></a>

### Read and edit issues without special tools

| Type | Need |
| --- | --- |
| ID | NEED-READABLE-FILES |
| Status | Accepted |
| Contributes to | [GOAL-REPO-ISSUES](#01M764FJ3Z3R4W2ZGMT5P434J0) |
| Contributes to | [GOAL-SMALL-TOOL](#01M764FJ3Z3P69KAH9C5ZFPZWD) |

Humans and agents can read, search, review and hand-edit issues as ordinary text files, with or without zban installed.

---

<a id="01M764FJ3Z4B0WDVBR7JXWBR5B"></a>

### Change issues on parallel branches

| Type | Need |
| --- | --- |
| ID | NEED-BRANCH-SAFE |
| Status | Accepted |
| Contributes to | [GOAL-REPO-ISSUES](#01M764FJ3Z3R4W2ZGMT5P434J0) |

People and agents working on separate branches or worktrees can create and update issues independently, and those changes merge through ordinary Git workflows.

---

<a id="01M764FJ3ZG12XTG21J7XKEQFC"></a>

### Structure and prioritize work

| Type | Need |
| --- | --- |
| ID | NEED-ORGANIZE |
| Status | Accepted |
| Contributes to | [GOAL-AGENT-WORKFLOW](#01M764FJ3Z3VPB3JGPG1P94AXW) |

Work can be classified, grouped into larger pieces, ordered by importance, and connected by dependencies and by where it was discovered.

---

<a id="01M764FJ3ZKQYBEXD73381XSPQ"></a>

### Find what to work on next

| Type | Need |
| --- | --- |
| ID | NEED-FIND-NEXT |
| Status | Accepted |
| Contributes to | [GOAL-AGENT-WORKFLOW](#01M764FJ3Z3VPB3JGPG1P94AXW) |

An agent or human can quickly see which issues can be started now, which are in progress, and why the others are waiting.

---

<a id="01M764FJ3Z6S4M6YKX2NXB78TK"></a>

### Record progress and outcomes

| Type | Need |
| --- | --- |
| ID | NEED-TRACK-PROGRESS |
| Status | Accepted |
| Contributes to | [GOAL-AGENT-WORKFLOW](#01M764FJ3Z3VPB3JGPG1P94AXW) |

Agents can create issues, record what is in progress, keep a task checklist current, and record what was done or why work was abandoned.

---

<a id="01M764FJ3Z8YQQEKPG1YMKZK0P"></a>

### Agents learn the workflow from the tool

| Type | Need |
| --- | --- |
| ID | NEED-AGENT-ONBOARDING |
| Status | Accepted |
| Contributes to | [GOAL-AGENT-WORKFLOW](#01M764FJ3Z3VPB3JGPG1P94AXW) |

An agent learns how to use zban, and the expected work discipline, from zban itself, whether it reaches zban through MCP or the command line, without the project writing its own instructions.

---

<a id="01M777QQGMVA28TW42QWJCGFX0"></a>

### Continue unfinished work in a new session

| Type | Need |
| --- | --- |
| ID | NEED-SESSION-CONTINUITY |
| Status | Accepted |
| Contributes to | [GOAL-AGENT-WORKFLOW](#01M764FJ3Z3VPB3JGPG1P94AXW) |

A developer can end an agent session with work unfinished and continue it in a new session, with the same or another agent, from the current state, the next steps and the approaches already rejected, without replaying the old conversation.

---

<a id="01M764FJ3Z76TNN9PYF84SMRNB"></a>

### Keep finished work as project memory

| Type | Need |
| --- | --- |
| ID | NEED-PROJECT-MEMORY |
| Status | Accepted |
| Contributes to | [GOAL-AGENT-WORKFLOW](#01M764FJ3Z3VPB3JGPG1P94AXW) |

Finished and abandoned issues stay available as context about past work, without crowding the list of active work.

---

<a id="01M764FJ3Z8BV1YNJN9TA5QQDA"></a>

### Trust the issue data

| Type | Need |
| --- | --- |
| ID | NEED-INTEGRITY |
| Status | Accepted |
| Contributes to | [GOAL-REPO-ISSUES](#01M764FJ3Z3R4W2ZGMT5P434J0) |

Users can detect and repair inconsistent issue data introduced by hand edits, interrupted writes or merges.

---

<a id="01M81JC6J5RQMFZ60SKJN5XGHQ"></a>

### Install and run zban through npm

| Type | Need |
| --- | --- |
| ID | NEED-NPM-DISTRIBUTION |
| Status | Accepted |
| Contributes to | [GOAL-AGENT-WORKFLOW](#01M764FJ3Z3VPB3JGPG1P94AXW) |

Developers and coding-agent setups can install and run zban through npm and `npx` on the supported platforms without manually downloading GitHub Release archives or compiling Rust from source.

---

## Vocabulary

---

<a id="01M764FJ3Z16YDB0AKKDJWA6K0"></a>

### Issue

| Type | Term |
| --- | --- |
| ID | TERM-ISSUE |
| Status | Accepted |
| Aliases | ticket |

One tracked unit of work, stored as one file in the project's `.zban/` directory. Every issue has a type: epic, feature, bug or task. The file format is defined in [DES-ISSUE-FILE](issues.mara.md#01M764FJ3ZZGJNFXW9VV1NNVAD).

---

<a id="01M764FJ3ZM923FNVJ8RAPBF9T"></a>

### Ready issue

| Type | Term |
| --- | --- |
| ID | TERM-READY |
| Status | Accepted |
| Aliases | actionable issue |

An issue that can be started now. Readiness is computed from status, type and relations whenever it is needed and is never stored; the rule is [DES-READINESS](issues.mara.md#01M764FJ3ZXCS66TCPPJ3HF04V).

---

<a id="01M764FJ3ZY93Z24QC6JRBVV5S"></a>

### Closed issue

| Type | Term |
| --- | --- |
| ID | TERM-CLOSED |
| Status | Accepted |

An issue with status `done` or `cancelled`. Closed issues are kept in `.zban/archive/` as project memory.
