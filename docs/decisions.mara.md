# Decisions and risks

The choices behind zban and the risks we knowingly carry.

## Decisions

:::mara decision ADR-FILES-IN-REPO
:mid: 01M764FJ3Z76SNWB69FYE8BQRG
:title: Store issues as plain files in the repository
:status: accepted
:justifies: REQ-ONE-FILE
:justifies: DES-SHARED-CORE

Agents work in a checkout. External trackers need network access and credentials, and they drift from the branch being worked on. A side database adds locking and recovery problems. zban stores issues only as Markdown files under `.zban/`, with no database, cache or daemon. As a consequence, issue state is local to each branch, and every command reads all issue files. That is fast enough up to several thousand issues.
:::

:::mara decision ADR-SMALL-SCOPE
:mid: 01M764FJ3ZAAC9D0V2C1WQY9CB
:title: Limit v1 to the issue loop
:status: accepted
:justifies: REQ-NO-GIT
:justifies: DES-CLI

Issue trackers for agents tend to grow into platforms: query engines, terminal and web UIs, agent chat, worktree and terminal management, claims, named sessions and handoff machinery. Each addition looks reasonable, but together they make the tool hard to reason about. zban v1 contains issue files, relations, readiness, lifecycle and archive, handoff notes on issues, the check command, the CLI, the MCP server and the agent guide.

Excluded are claims and locking, state shared across worktrees, running Git, running agents or managing worktrees, terminal and web UIs, a query language, templates, custom fields, configurable vocabulary, manual ordering, assignees and time tracking. Adding a capability requires a recorded need and a decision first, not just precedent in another tool.
:::

:::mara decision ADR-NO-CLAIMS
:mid: 01M764FJ3ZDTFAZ4GG9A8BJ03S
:title: Rely on the in-progress status instead of claims
:status: accepted
:justifies: DES-READINESS
:accepts: RISK-DUPLICATE-WORK

Agents in different worktrees cannot see each other's status changes until their branches merge. Lock files shared through the Git common directory can coordinate them, but at great cost in complexity. zban v1 has no claims and no assignee field. `in-progress` is a durable status in the file, and whoever runs parallel agents (a human or an orchestrator) decides who works on what. If claims prove necessary later, they will be an additional ephemeral layer that leaves the file format and the `in-progress` status unchanged.
:::

:::mara decision ADR-FIXED-VOCABULARY
:mid: 01M764FJ3ZYNNKFADFY3G06GRD
:title: Hardcode statuses, types and priorities
:status: accepted
:justifies: REQ-FIXED-VOCABULARY

A configurable vocabulary forces every tool and agent to read configuration first, and it makes the agent guide differ per project. zban fixes all three sets. `draft` keeps unrefined ideas out of ready work. `cancelled` keeps abandoned work apart from delivered work. There is no milestone type. Nested epics, ordered with `blocked_by`, cover phases inside larger work, and a tag groups a release that cuts across epics. Priorities are five named levels because names read better than numbers in prompts.
:::

:::mara decision ADR-RANDOM-IDS
:mid: 01M764FJ3ZADMY1PJZ654H5PHK
:title: Use short random IDs with a project prefix
:status: accepted
:justifies: DES-ISSUE-ID

Sequential numbers collide when parallel branches create issues, and they need a shared counter. UUIDs never collide but are noisy, and the short forms people cite are only prefixes, which can become ambiguous. Four random base36 characters, about 1.7 million values, leave a noticeable chance of collisions across branches in a long-lived project. zban uses its prefix plus six random Crockford base32 characters, about a billion values per prefix, and `zban check` detects any duplicate. IDs carry no order; `created_at` provides it.

The filename is the bare ID, without a slug of the title. A slug goes stale when the title changes, or causes rename churn if it follows the title. Without one, an agent opens an issue straight from its ID, two issues that share an ID land on the same path so Git reports the collision when merging, and there are no slugging rules to specify. Listing titles is left to `zban list`.
:::

:::mara decision ADR-AUTO-ARCHIVE
:mid: 01M764FJ3ZWDY1N7FRADERZ3XJ
:title: Archive when closing rather than on command
:status: accepted
:justifies: REQ-AUTO-ARCHIVE
:justifies: DES-LIFECYCLE

Archiving in bulk on an explicit command depends on someone running it, and agents may run it only when asked, so closed issues pile up among active ones. zban moves a file into `.zban/archive/` as soon as the issue closes, and back when it reopens. The status stays authoritative, and `zban check --fix` repairs a misplaced file. The costs: a close shows up as a rename plus an edit in diffs, and an edit on one branch combined with a close on another merges as a rename/modify, which Git normally resolves.
:::

:::mara decision ADR-MERGE-FRIENDLY-FORMAT
:mid: 01M764FJ3Z8SW2MT7EAP58RF8R
:title: Shape the file format for Git merges
:status: accepted
:justifies: DES-ISSUE-FILE
:justifies: REQ-RELATIONS

Issue files are merged by Git, so the format avoids needless conflicts and drift. Each relation is stored once, on one issue. Storing both directions, such as `blocking` and `blocked_by`, lets the two disagree. There is no `updated_at`, which would conflict on every concurrent edit; Git history already records changes. Lists hold one item per line, the key order is fixed, and the ID exists only in the filename. Relations hold bare IDs, not Markdown links: a link would need rewriting in every referring issue whenever its target is archived, so closing one issue would touch others and conflict with their edits on other branches. Editors find an issue by its ID because the ID is the filename.
:::

:::mara decision ADR-HANDOFF-ON-ISSUE
:mid: 01M777QQGMRG4N17FAT4EGF628
:title: Keep handoffs as notes on issues, written and read on request
:status: accepted
:justifies: REQ-HANDOFF
:justifies: DES-HANDOFF

Agents lose context between sessions, and a short note written while the conversation is still available carries it best. Notes generated automatically when a session ends come too late: the model is gone and session-end hooks get little time. Stop hooks that force a note after every turn are noisy. So agents write a note only when the developer asks.

The note lives in the issue body, overwritten in place, rather than in a separate handoff file or entity. The issue is already the record that outlives a session, a note there travels with the branch, and Git keeps earlier versions. A body section stays readable in Markdown previews and keeps the frontmatter small. zban stamps only the time, because it cannot read Git; the agent records the commit and branch.

zban does not deliver notes into new sessions. Notes belong to issues, and zban cannot tell which issue a session is about. Including every in-progress note in `prime` or the MCP instructions would cost every session and show other people's work after merges. Choosing by the current worktree or branch would need Git access or worktree tracking, which [[REQ-NO-GIT]] and [[ADR-SMALL-SCOPE]] exclude. The developer asks the agent to continue, and the agent shows the issue.
:::

:::mara decision ADR-CLI-AND-MCP
:mid: 01M764FJ3Z27M3C4E700KAKEE6
:title: Offer CLI and MCP over one core, plus prime
:status: accepted
:justifies: REQ-SURFACE-PARITY
:justifies: REQ-AGENT-GUIDE

MCP gives agents typed tools and server instructions in clients that support them. The CLI works everywhere: in shells, CI and agents without MCP. v1 ships both over one shared core. The agent guide is delivered as MCP server instructions, and through `zban prime` for agents that use only the CLI or whose clients ignore server instructions.
:::

:::mara decision ADR-RUST-BINARY
:mid: 01M764FJ3ZRYT7TB8RKNHPFM8K
:title: Ship one Rust binary, dual-licensed
:status: accepted
:justifies: DES-SHARED-CORE

zban is written in Rust and ships as one self-contained binary. It is distributed through `cargo install zban` and prebuilt binaries on GitHub Releases; package-manager wrappers are not part of v1. The license is `MIT OR Apache-2.0`.

v1 supports Linux and macOS on x86_64 and aarch64. Windows is deferred until there is a need for it, because it needs its own handling of atomic file replacement, CRLF line endings and paths.
:::

:::mara decision ADR-DEPENDENCIES
:mid: 01M779N7PPXZEFMF6M942DMS6H
:title: Build on established crates
:status: accepted
:justifies: DES-SHARED-CORE
:justifies: DES-MCP-TOOLS
:justifies: DES-ISSUE-FILE

zban needs argument parsing, JSON, a YAML parser for issue headers, an MCP server and graph checks over relations. Writing these by hand keeps the dependency tree small, but spends effort on parser and protocol details that maintained crates already handle, and hand-written protocol code tends to lag behind MCP revisions. zban builds on established crates: `clap` for arguments, `serde` and `serde_json` for JSON, a maintained YAML parser that exposes enough detail to reject anchors, aliases, tags and duplicate keys, the official MCP Rust SDK `rmcp` for the server, `petgraph` for cycle and reachability checks over `parent` and `blocked_by`, and `getrandom` for IDs.

The YAML crate parses header values only. A thin text layer splits the header into top-level entries, so unknown keys are kept verbatim and known keys are written in the canonical form of [[DES-ISSUE-FILE]] without a generic YAML serializer. `rmcp` needs the `tokio` runtime; only `zban mcp` starts it, and the core library stays synchronous. The costs are a larger binary and dependency tree. Adding a runtime dependency beyond these needs a recorded decision.
:::

:::mara decision ADR-NO-CASCADE
:mid: 01M76XFGSJ81ZR60CGYEMESP5K
:title: Refuse to close unfinished work rather than cascade
:status: accepted
:justifies: REQ-CLOSE-CHECKLIST
:justifies: DES-LIFECYCLE

Closing an issue while its checklist or children are unfinished hides that work. Open children of a closed parent silently drop out of the ready list ([[DES-READINESS]]). Cascading the close to the children would avoid that, but one command would then rewrite many files, could stop part-way, and would give every child the parent's reason, although some may be finished, moved elsewhere or still wanted. zban refuses such a close unless it is forced, and it never closes another issue implicitly. A forced close leaves the open children in place, and `zban check` warns about them. Closing `done` and `cancelled` is treated the same way; only unchecked task items are specific to `done`.
:::

## Risks

:::mara risk RISK-DUPLICATE-WORK
:mid: 01M764FJ3ZM69JRX3088WM98RP
:title: Parallel agents start the same issue
:status: accepted
:likelihood: possible
:consequence: moderate
:threatens: NEED-FIND-NEXT

Two agents on different branches may both see an issue as ready and start it, because `in-progress` on one branch is invisible on the other until merge. The work is duplicated and one result is discarded. [[ADR-NO-CLAIMS]] accepts this for v1.
:::

:::mara risk RISK-ID-COLLISION
:mid: 01M764FJ3Z4YEGWP7Z0Y8VZE9N
:title: Issues created independently share an ID
:status: accepted
:likelihood: rare
:consequence: minor
:threatens: REQ-MERGE-SAFE-IDS

Two branches could generate the same random ID. Both issues are then `<id>.md`, so the merge stops with an add/add conflict that has to be resolved by giving one issue a new ID. The collision merges silently only when one copy was already archived on its branch, so the paths differ; the ID is then ambiguous until `zban check` reports it. With 2^30 values per prefix, a thousand issues created on each of two branches collide with a probability of about 0.1%.
:::

:::mara risk RISK-SESSION-ABOVE-PROJECT
:mid: 01M772MFTG88ME51FKVXTNF540
:title: Agents started above a project miss the guide
:status: accepted
:likelihood: possible
:consequence: minor
:threatens: NEED-AGENT-ONBOARDING

zban decides whether to deliver the agent guide from the directory where a session starts. An agent started in a folder above the project, such as a folder of several repositories or the top of a monorepo with `.zban/` in a subfolder, gets nothing from `zban prime` and only a short note over MCP. It may then track work outside zban. Delivering the full guide everywhere would cost every unrelated session about 150 lines and give advice that does not apply there. The short MCP note still points to `zban prime -C`, and the user guide documents the limitation.
:::

:::mara risk RISK-MERGE-CONFLICTS
:mid: 01M764FJ3ZVFTZJKT1TXJQ3SM9
:title: Concurrent edits to one issue conflict on merge
:status: accepted
:likelihood: possible
:consequence: minor
:threatens: NEED-BRANCH-SAFE

Two branches that edit the same lines of the same issue conflict when merged, which interrupts an agent's flow. The file format keeps the conflicting surface small.
:::

:::mara risk RISK-SCOPE-CREEP
:mid: 01M764FJ3Z2R43ACV2SEMDKTXB
:title: zban grows beyond a small tool
:status: accepted
:likelihood: likely
:consequence: major
:threatens: GOAL-SMALL-TOOL

Each feature of comparable tools looks reasonable on its own, and agents readily propose more. Accumulated features would make zban hard to understand and maintain. [[ADR-SMALL-SCOPE]] and the repository's agent instructions require a recorded need and decision before scope grows.
:::
