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
:title: Ship one Rust binary through GitHub Releases, dual-licensed
:status: accepted
:justifies: DES-SHARED-CORE

zban is written in Rust and ships as one self-contained binary. It is distributed only through GitHub Releases: each release attaches prebuilt binaries and a SHA-256 checksum file, and its tag can be built from source with `cargo install --git https://github.com/convesoft/zban --tag vX.Y.Z`. zban is not published to crates.io, and its manifest sets `publish = false` so that nobody publishes it by accident. zban is an application, not a library other crates depend on, and one channel keeps a release to pushing a tag, with no registry account, publish token or crate name to maintain. Package-manager wrappers, such as an npm package, are not part of v1; adding one needs its own decision. The license is `MIT OR Apache-2.0`.

v1 supports Linux and macOS on x86_64 and aarch64. Windows is deferred until there is a need for it, because it needs its own handling of atomic file replacement, CRLF line endings and paths.

Release binaries are built for `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-apple-darwin` and `aarch64-apple-darwin`. On Linux, musl is chosen over glibc because the binary is then statically linked: one file runs on every distribution, whatever its glibc version, including musl-based ones such as Alpine. A glibc binary would instead have to be built against the oldest glibc it should support. musl's slower memory allocator does not matter to a short-lived command that reads small files.

A release is cut by pushing a tag `vX.Y.Z` that names the version in `Cargo.toml` and points at a commit on `main`. The release workflow refuses any other tag, and it runs the test suite on the tagged commit, on each platform whose binary it can run, before it creates the release. A release then ships only code that has landed on `main` and passes its tests.
:::

:::mara decision ADR-DEPENDENCIES
:mid: 01M779N7PPXZEFMF6M942DMS6H
:title: Build on established crates
:status: accepted
:justifies: DES-SHARED-CORE
:justifies: DES-MCP-TOOLS
:justifies: DES-ISSUE-FILE

zban needs argument parsing, JSON, a YAML parser for issue headers, an MCP server and graph checks over relations. Writing these by hand keeps the dependency tree small, but spends effort on parser and protocol details that maintained crates already handle, and hand-written protocol code tends to lag behind MCP revisions. zban builds on established crates: `clap` for arguments, `serde` and `serde_json` for JSON, `saphyr-parser` for YAML, whose parse events expose enough detail to reject anchors, aliases, tags and duplicate keys, the official MCP Rust SDK `rmcp` for the server, `petgraph` for cycle and reachability checks over `parent` and `blocked_by`, and `getrandom` for IDs.

The YAML crate parses header values only. A thin text layer splits the header into top-level entries, so unknown keys are kept verbatim and known keys are written in the canonical form of [[DES-ISSUE-FILE]] without a generic YAML serializer. The two keys of `config.toml` are read by a small parser in zban rather than a TOML crate. `rmcp` needs the `tokio` runtime; only `zban mcp` starts it, and the core library stays synchronous. The costs are a larger binary and dependency tree. Adding a runtime dependency beyond these needs a recorded decision.

The tests have one dependency of their own, `tempfile`, which is not built into the binary. Each test builds a fresh fixture project in a `TempDir`: a directory with a random name under the system's temporary directory, owned by the test and deleted when it ends. No two tests share a directory, even under a runner that starts one process per test, and no fixture lies inside this repository, whose own `.zban/` a test would otherwise find.
:::

:::mara decision ADR-NO-CASCADE
:mid: 01M76XFGSJ81ZR60CGYEMESP5K
:title: Refuse to close unfinished work rather than cascade
:status: accepted
:justifies: REQ-CLOSE-CHECKLIST
:justifies: DES-LIFECYCLE

Closing an issue while its checklist or children are unfinished hides that work. Open children of a closed parent silently drop out of the ready list ([[DES-READINESS]]). Cascading the close to the children would avoid that, but one command would then rewrite many files, could stop part-way, and would give every child the parent's reason, although some may be finished, moved elsewhere or still wanted. zban refuses such a close unless it is forced, and it never closes another issue implicitly. A forced close leaves the open children in place, and `zban check` warns about them. Closing `done` and `cancelled` is treated the same way; only unchecked task items are specific to `done`.
:::

:::mara decision ADR-WRITE-LOCK
:mid: 01M77YZD0BXR4H9NYMR7CFMW6R
:title: Lock the project directory only around each write and read
:status: accepted
:justifies: DES-SHARED-CORE

Comparing a file's bytes just before renaming over it leaves a window between the comparison and the rename. The v0.1.0 review saw two `zban update` runs on one issue both pass the comparison, and the later rename dropped the earlier change while both reported success. zban therefore holds an exclusive advisory lock on the `.zban/` directory from each such comparison until the rename or deletion that follows, and while it moves a file between `.zban/` and the archive. The standard library's `File::lock` provides it on Linux and macOS without another dependency; it needs Rust 1.89, which the manifest declares as `rust-version`.

Reads take the same lock, shared, from listing `.zban/` and `.zban/archive/` until they have read the bytes of every issue file listed, and parse the files after releasing it. POSIX leaves it unspecified whether a directory listing returns an entry that is renamed while the listing runs, and on btrfs a listing that overlapped a rename over an issue file left that issue out: `show` and `update` failed with `not_found` for an issue that existed throughout, and a listing could also miss a referrer or an open child. Holding the lock only while listing was not enough: a close or reopen that moved a listed file into the other directory before it was read left the issue out, or kept it as unreadable, about once in 16,000 `show` runs while another process kept closing and reopening the issue. Under the lock, a read finds each file at the path and with the bytes that the last zban write left. Writes wait while a read lists the directories and reads the files, but not while it parses them, and reads never wait for each other. Looking for a listed file that has gone in the other directory was rejected, because the file can move back before that second look and it needs more code than holding the lock a little longer. Looking a named issue up by its path as well was rejected because it does not help listings, referrers or children, and listing again because no number of attempts guarantees a complete listing.

A lock file was rejected: a crash can leave it behind and it can be committed by accident, whereas the operating system releases a lock on the directory when the process ends, even when it is killed. Holding the lock for a whole command is not needed: the byte comparison already reports stale reads as `conflict`, and short locks keep other zban processes from waiting while a command parses the files and prepares its change.

The lock is not a claim: it covers one file operation or one read of the files, not an issue between commands, so the exclusion of claims and locking in [[ADR-SMALL-SCOPE]] stands. It coordinates zban processes that share one `.zban/` directory. Editors, Git and other worktrees do not take it. On a file system without advisory locks, taking the lock fails as unsupported, and zban then writes or reads without it, a write guarded by the byte comparison alone as before; any other failure to take the lock fails the command with `io`.
:::

:::mara decision ADR-CREATE-WITHOUT-HARD-LINKS
:mid: 01M785TJ0TQG6K5K6WJS3G8GBE
:title: Create issue files by renaming where hard links are unsupported
:status: accepted
:justifies: DES-SHARED-CORE
:justifies: REQ-NO-LOST-UPDATES

`init` and `create` write a temporary file and then hard-link it to its name, so that the write is atomic ([[REQ-PRESERVE-CONTENT]]) and a create fails rather than replacing an existing file. Some file systems have no hard links. On Linux, linking then fails with `EPERM`, which `link(2)` documents for a file system that "does not support the creation of hard links"; FAT and exFAT drives and many FUSE mounts are such file systems. On macOS, `linkat(2)` documents `ENOTSUP` for a file system that "does not support this call". With zban 0.1.0 on an `rclone mount` of a local directory, mounted by an ordinary user, `init` and every `create`, open or closed, failed with `io` and "Operation not permitted (os error 1)". `update`, closing, reopening and `delete`, which rename or remove files, worked, and so did advisory locks.

zban keeps the hard link. When linking fails with `EPERM` on Linux, or as unsupported (`Unsupported`, or the raw errors `ENOSYS`, `ENOTSUP` and `EOPNOTSUPP`, as for the lock of [[ADR-WRITE-LOCK]]), `init` and `create` instead take the exclusive lock, check that the name is unused, for an issue in both `.zban/` and the archive, and rename the temporary file to it, as closing and reopening already move a file. Any other failure to link fails with `io`. Writes stay atomic, and an interrupted create leaves only a temporary file. File systems with hard links keep the absolute guarantee that a create never replaces a file. On the others, no other zban process can take the name between the check and the rename, but a writer that does not take the lock can: an editor, Git, or zban on a file system that has no advisory locks either. It would have to create the same freshly drawn random ID, one of 2^30 values that neither directory held an instant before, in that instant. [[ADR-WRITE-LOCK]] accepts the same caveat for replacing and moving a file. Falling back on an `EPERM` with another cause is harmless: each such cause that can apply to a file zban has just created also makes the rename fail, so the command fails with `io` as before.

The rejected alternatives:

- **Keep hard links only.** Name the missing support in the `io` message and document that zban needs hard links. No promise changes, but zban cannot create issues on these file systems.
- **Create the issue file exclusively.** Open the issue's name with exclusive creation, then write and sync it. A create still never replaces a file, but an interrupted create leaves an empty or partial issue file, which breaks [[REQ-PRESERVE-CONTENT]]. `check` would report it as unreadable, and `check --fix` could not tell it from a damaged hand edit. A read during the write would see the partial file unless the write held the lock.
- **Rename without replacing.** `renameat2` with `RENAME_NOREPLACE` on Linux and `renamex_np` with `RENAME_EXCL` on macOS are atomic and never replace. The standard library offers neither, so they need a new dependency ([[ADR-DEPENDENCIES]]), and FUSE mounts support them only when their server does, so another fallback would still be needed.
- **Always check, then rename.** Dropping the hard link would remove one code path, but every file system would then carry the caveat above.
:::

## Risks

:::mara risk RISK-DUPLICATE-WORK
:mid: 01M764FJ3ZM69JRX3088WM98RP
:title: Parallel agents duplicate work
:status: accepted
:likelihood: possible
:consequence: moderate
:threatens: NEED-FIND-NEXT

Two agents on different branches may both see an issue as ready and start it, because `in-progress` on one branch is invisible on the other until merge. Two agents that find the same problem may likewise each file an issue for it, because an issue created on one branch is invisible on the other until merge. The work is duplicated and one result is discarded. [[ADR-NO-CLAIMS]] accepts this for v1, and [[ADR-SMALL-SCOPE]] excludes the state shared across worktrees that could prevent it.
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
