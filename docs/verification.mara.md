# Verification

How we establish that zban meets its requirements and goals. Each definition
says what to check and what to expect. A definition is not a result: results
come from actual test runs, cited when an issue is closed.

## Requirement checks

:::mara verification VER-ISSUE-FILES
:mid: 01M764FJ3ZP3SETQ9WX71Z5RYM
:title: Round-trip issue files
:status: accepted
:methods: test
:verifies: REQ-ONE-FILE
:verifies: REQ-PRESERVE-CONTENT
:verifies: REQ-FIXED-VOCABULARY
:verifies: REQ-NO-LOST-UPDATES

Load fixture projects, including a copy of this repository's `.zban/`, and write each issue back without changes. Files already in canonical form are byte-identical. Metadata-only updates leave body bytes and unknown keys intact. A hand-written header with comments, quoted scalars and flow lists loads, and rewriting it drops the comments and writes the canonical form. Titles that YAML would read as another type, or that contain `: ` or ` #`, survive a write and a read unchanged. The blank line after the header is not part of the body, and an issue with an empty body ends after the closing line. Values outside the fixed vocabulary are rejected by create and update and reported by check. A write interrupted before its rename leaves the original file unchanged and no partial issue file. When a file changes between the read and the write of an update, the update fails with `conflict` and the other change survives. Many processes of the real binary update one issue at the same time, each retrying after `conflict`; none fails with another error, and every change an update reported as written is in the final file exactly once. Processes that run `zban show` and `zban list --all` find the issue every time and warn about no file, both while another process keeps renaming a copy of the issue over it under the lock and while another process keeps closing and reopening it as `zban update` does, rewriting it in place and then moving it into the other directory, each step under the lock. An update, a delete and a move by `zban check --fix` wait while another process holds the lock on `.zban/`, and so do `show` and `list`, which do not wait while another process holds it shared. A failure to take the lock lets the write or read go ahead only when it means that locking is unsupported. Create never replaces an existing file. A test makes linking fail as on a file system without hard links, since CI has none: with `EPERM` on Linux or as unsupported, `init` and `create` rename the new file into place instead, a create skips an ID taken in `.zban/` or `.zban/archive/`, and no temporary file is left; any other linking error fails with `io` and creates no file. When writing the configuration of `init` fails, by linking or by the write itself, `init` leaves no `.zban/`, and running it again succeeds.
:::

:::mara verification VER-ID-GENERATION
:mid: 01M764FJ3ZJ4XPRVDCB25S8X0A
:title: Generate merge-safe IDs
:status: accepted
:methods: test
:methods: analysis
:verifies: REQ-MERGE-SAFE-IDS

Generated IDs match `<prefix>-[0-9a-hjkmnp-tv-z]{6}`. With an injected generator that returns an existing ID first, create retries and succeeds with a fresh ID, checking both `.zban/` and `.zban/archive/`. `zban init` without `--prefix` writes prefix `zb`, and an invalid `--prefix` is rejected without creating `.zban/`. The collision analysis in [[RISK-ID-COLLISION]] still holds for the implemented alphabet and length.
:::

:::mara verification VER-NO-GIT
:mid: 01M764FJ3ZRQW0D630KWF72XY9
:title: Work without version control
:status: accepted
:methods: test
:methods: inspection
:verifies: REQ-NO-GIT

Run every command in a temporary directory that is not a Git repository and confirm they all succeed. Snapshot the files outside `.zban/` before and after a full create, update, close, reopen and check cycle, and confirm nothing changed. Inspect the source and confirm that zban starts no external processes.
:::

:::mara verification VER-RELATIONS-READY
:mid: 01M764FJ3Z2PGEHD2FMGDY4N6G
:title: Compute relations and readiness
:status: accepted
:methods: test
:verifies: REQ-RELATIONS
:verifies: REQ-READY

The fixture contains: A blocking B; an independent C; an epic E; a child D whose parent is blocked; a task G under an epic that sits inside another epic blocked by an open epic J; a draft F; and issues of differing priority and age. The ready list contains exactly the expected issues in priority, age and ID order. Each non-ready issue reports its reasons. A `todo` child of a `draft` parent is ready. Closing A makes B ready. G reports `ancestor_blocked` and becomes ready once J closes. An issue blocked by an unreadable file is not ready, and an unreadable parent ends the ancestor chain. Adding a `related` link that already exists in the reverse direction changes no file, and removing it from either side deletes it from the issue that stores it. References to unknown issues and self references fail with `invalid_reference`. Cycles through `parent`, through `blocked_by`, and through both together, such as an epic blocked by its own child, fail with `cycle`, and so does a `blocked_by` on an ancestor. Deleting a referenced issue is refused. `show` lists children, blocks, related and discovered issues in the reverse direction.
:::

:::mara verification VER-LIFECYCLE
:mid: 01M764FJ3Z7VZZATFZS034NZXH
:title: Close, archive and reopen issues
:status: accepted
:methods: test
:verifies: REQ-CLOSE-CHECKLIST
:verifies: REQ-AUTO-ARCHIVE

Setting `done` with an unchecked task item fails and changes nothing. With force it succeeds. Creating an issue as `done` with an unchecked task item fails with the same code and creates no file, through the command line and MCP alike, and `create` accepts no force. A task item inside a fenced code block does not count, including a block nested in list items whose fence is indented to their content. A fence line indented by four columns outside a list item, or by four columns more than the content of its list item, opens no block, so the task items after it count. Setting `done` or `cancelled` on an issue with an open child fails, names the child and changes nothing. With force it succeeds and leaves the child unchanged. Closing sets `closed_at` and moves the file to `.zban/archive/`, and reopening clears `closed_at` and moves it back. Archived issues appear in `show`, in search, in relations and in `list --all`, but not in the default listing. Listings report checklist progress.
:::

:::mara verification VER-HANDOFF
:mid: 01M777QQGM8AY4TX1GGM86HG6F
:title: Write and read handoff notes
:status: accepted
:methods: test
:verifies: REQ-HANDOFF

Through the real binary: `update --handoff` adds a timestamped `## Handoff` section to a body without one, and a second update replaces it in place while every byte outside the section stays unchanged. Text read from standard input works. `--no-handoff` removes the section. Text containing a level 2 heading is rejected, a `## Handoff` heading inside a fenced code block is ignored while one after a fence line indented by four columns outside a list item is found, a body with two handoff sections makes both options fail without writing, and so does `--handoff` on a body that leaves a fenced code block open. `show` returns the text and time, a heading without a timestamp gives `written_at: null`, and `list` reports `handoff_at`. Closing an issue leaves its handoff untouched, and `zban prime` and the MCP instructions contain no handoff text.
:::

:::mara verification VER-CHECK
:mid: 01M764FJ3ZPNVFEX72X29BGRTT
:title: Report and repair inconsistencies
:status: accepted
:methods: test
:verifies: REQ-CHECK

Use a corpus of broken projects, one for each finding code in [[DES-CHECK]]. `zban check` reports each problem with that code, its severity and path. It exits non-zero on errors, and exits 0 when it finds only the warnings named in [[REQ-CHECK]]. `zban check --fix` moves only misplaced files and deletes only leftover temporary files, leaving every other file and finding untouched, and a second check reports neither. A misplaced file whose target path is taken stays in place and is still reported. The temporary file of a real interrupted write is reported, and files whose names only resemble one are not.
:::

:::mara verification VER-CLI
:mid: 01M764FJ3ZZBZYQWC7EQ4093FG
:title: Exercise the command line contract
:status: accepted
:methods: test
:verifies: REQ-JSON-OUTPUT
:verifies: REQ-LIST-FILTERS
:verifies: REQ-BODY-EDITS
:verifies: REQ-DAMAGED-DATA

Run every command with `--json` through the real binary. The output parses as one document with the fields documented in [[DES-CLI]]. Failures return an error object and exit status 1, and invalid usage exits with 2. Each error code in [[DES-CLI]] is produced by at least one test. Each list filter alone and in combination, search and `--all` select the expected issues, in the order defined by [[DES-READINESS]], and `--parent` selects direct children only. `list --parent` with a malformed or unknown ID fails with `not_found`, and with an unreadable or duplicated ID fails like `show`. The short options `-s`, `-t` and `-p` of `list` select the same issues as `--status`, `--type` and `--priority`. Commands run outside a project fail with `project_not_found`. `-C DIR`, given a project root or one of its subdirectories, behaves like starting in `DIR`. `init` inside a project returns that project's `.zban/` as `enclosing` and names it in human output, and `init` outside any project returns `enclosing: null` and prints no note. Replacing text that is absent or occurs more than once fails without writing. `--append` joins its text after exactly one blank line. An update that combines replacements, an append and metadata changes is all or nothing, and one update can check the last task item and set `done`. Body input from standard input works for create and update.

A fixture holds a file with Git conflict markers, an issue with an invalid status, an issue with a dangling `blocked_by` and an ID present in both `.zban/` and `.zban/archive/`. `list` and `show` of other issues succeed and report warnings for the broken files. Updating the issue with the dangling blocker, for example setting it `in-progress`, succeeds. Showing or updating an unreadable issue fails with `unreadable`, and showing the duplicated ID fails with `ambiguous_id`.
:::

:::mara verification VER-SURFACE-PARITY
:mid: 01M764FJ3Z01GJN91JB409C3GW
:title: Match CLI and MCP behaviour
:status: accepted
:methods: test
:verifies: REQ-SURFACE-PARITY
:verifies: REQ-MCP-STDIO

A real MCP client talks to `zban mcp` over stdio and runs a create, block, ready, update, close and check flow. For the same requests against identical fixtures, the tool results equal the CLI's JSON output, and invalid arguments produce the same error codes. The tool list matches [[DES-MCP-TOOLS]]. Started outside a project, the server sends the short note as its instructions and every tool without a `directory` fails with `project_not_found`. Calls whose `directory` points into another project or worktree read and write only that project, and a relative `directory` is rejected.
:::

:::mara verification VER-AGENT-GUIDE
:mid: 01M764FJ3ZZ9YTCWGRGCH00BBV
:title: Keep the agent guide consistent
:status: accepted
:methods: test
:methods: inspection
:verifies: REQ-AGENT-GUIDE

Snapshot tests cover `zban prime` and the MCP server instructions, both rendered from the same template. The vocabulary lists in the guide equal the enumerations in the code, and every command or tool the guide names exists. A test fails when the MCP rendering exceeds the character budget in [[DES-AGENT-GUIDE]]. `zban prime` outside a project prints nothing and exits 0, and `zban prime -C <project>` from there prints the full guide. `zban prime --json` prints `{"guide": ...}`, with `null` outside a project. A server started inside a project sends the full guide; one started outside sends the short note, which names `directory` and `zban prime -C`. Inspection confirms the guide covers the sections in [[DES-AGENT-GUIDE]] and stays under about 150 lines.
:::

## Outcome checks

:::mara validation VAL-DOGFOOD
:mid: 01M764FJ3ZND8RM7NMZNT08MP6
:title: Track zban's own development with zban
:status: accepted
:methods: demonstration
:validates: GOAL-AGENT-WORKFLOW
:validates: GOAL-REPO-ISSUES
:exercises: SCN-AGENT-SESSION
:exercises: SCN-PARALLEL-BRANCHES

After the CLI works, zban's own backlog is managed only through zban. Over at least ten issues worked on by agents, including some in parallel worktrees, review the history. Whoever chose the work, a single agent or whoever ran the parallel agents ([[ADR-NO-CLAIMS]]), found it with `zban list --ready`. Given only an issue to work on, agents kept checklists and summaries and closed their issues without further prompting. Discovered work was filed as issues. No issue needed a manual frontmatter repair other than merge conflicts. `zban check` stayed clean on `main`. Record shortfalls as issues or as changes to this knowledge.
:::

:::mara validation VAL-SCOPE-REVIEW
:mid: 01M774FDH7HRY5CQ0DNGHYY3FG
:title: Review zban's size before each release
:status: accepted
:methods: inspection
:validates: GOAL-SMALL-TOOL

Before each release, review the knowledge and the agent guide. Every capability traces to an accepted need. The commands in [[DES-CLI]] and the keys in [[DES-ISSUE-FILE]] changed only through recorded decisions. The agent guide stays under about 150 lines and covers every operation. In [[VAL-DOGFOOD]], agents needed nothing beyond the guide to predict what zban would do. Record shortfalls as issues or as changes to this knowledge.
:::
