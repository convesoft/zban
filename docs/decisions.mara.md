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

Issue trackers for agents tend to grow into platforms: query engines, terminal and web UIs, agent chat, worktree and terminal management, claims, sessions and handoffs. Each addition looks reasonable, but together they make the tool hard to reason about. zban v1 contains issue files, relations, readiness, lifecycle and archive, the check command, the CLI, the MCP server and the agent guide.

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

A configurable vocabulary forces every tool and agent to read configuration first, and it makes the agent guide differ per project. zban fixes all three sets. `draft` keeps unrefined ideas out of ready work. `cancelled` keeps abandoned work apart from delivered work. There is no milestone type; group a release with an epic or a tag. Priorities are five named levels because names read better than numbers in prompts.
:::

:::mara decision ADR-RANDOM-IDS
:mid: 01M764FJ3ZADMY1PJZ654H5PHK
:title: Use short random IDs with a project prefix
:status: accepted
:justifies: DES-ISSUE-ID

Sequential numbers collide when parallel branches create issues, and they need a shared counter. UUIDs never collide but are noisy, and the short forms people cite are only prefixes, which can become ambiguous. Four random base36 characters, about 1.7 million values, leave a noticeable chance of collisions across branches in a long-lived project. zban uses its prefix plus six random Crockford base32 characters, about a billion values per prefix, and `zban check` detects any duplicate. IDs carry no order; `created_at` provides it.
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

Issue files are merged by Git, so the format avoids needless conflicts and drift. Each relation is stored once, on one issue. Storing both directions, such as `blocking` and `blocked_by`, lets the two disagree. There is no `updated_at`, which would conflict on every concurrent edit; Git history already records changes. Lists hold one item per line, the key order is fixed, and the ID exists only in the filename.
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

Two branches could generate the same random ID. The files have different slugs, so Git merges both silently and the ID becomes ambiguous. With 2^30 values per prefix, a thousand issues created on each of two branches collide with a probability of about 0.1%.
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
