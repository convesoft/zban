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

Load fixture projects, including a copy of this repository's `.zban/`, and write each issue back without changes. Files already in canonical form are byte-identical. Metadata-only updates leave body bytes and unknown keys intact. Values outside the fixed vocabulary are rejected by create and update and reported by check. A write interrupted before its rename leaves the original file unchanged and no partial issue file. When a file changes between the read and the write of an update, the update fails with `conflict` and the other change survives. Create never replaces an existing file.
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

The fixture contains: A blocking B; an independent C; an epic E; a child D whose parent is blocked; a draft F; and issues of differing priority and age. The ready list contains exactly the expected issues in priority, age and ID order. Each non-ready issue reports its reasons. A `todo` child of a `draft` parent is ready. Closing A makes B ready. Adding a `related` link that already exists in the reverse direction changes no file, and removing it from either side deletes it from the issue that stores it. References to unknown issues, self references, and cycles through `parent` or `blocked_by` are rejected. Deleting a referenced issue is refused. `show` lists children, blocks, related and discovered issues in the reverse direction.
:::

:::mara verification VER-LIFECYCLE
:mid: 01M764FJ3Z7VZZATFZS034NZXH
:title: Close, archive and reopen issues
:status: accepted
:methods: test
:verifies: REQ-CLOSE-CHECKLIST
:verifies: REQ-AUTO-ARCHIVE

Setting `done` with an unchecked task item fails and changes nothing. With force it succeeds. A task item inside a fenced code block does not count. Setting `done` or `cancelled` on an issue with an open child fails, names the child and changes nothing. With force it succeeds and leaves the child unchanged. Closing sets `closed_at` and moves the file to `.zban/archive/`, and reopening clears `closed_at` and moves it back. Archived issues appear in `show`, in search, in relations and in `list --all`, but not in the default listing. Listings report checklist progress.
:::

:::mara verification VER-CHECK
:mid: 01M764FJ3ZPNVFEX72X29BGRTT
:title: Report and repair inconsistencies
:status: accepted
:methods: test
:verifies: REQ-CHECK

Use a corpus of broken projects, one for each finding kind listed in [[REQ-CHECK]]. `zban check` reports each problem with its code, severity and path. It exits non-zero on errors, and exits 0 when it finds only the warnings named in [[REQ-CHECK]]. `zban check --fix` moves only misplaced files, leaves every other finding untouched, and a second check reports no placement findings.
:::

:::mara verification VER-CLI
:mid: 01M764FJ3ZZBZYQWC7EQ4093FG
:title: Exercise the command line contract
:status: accepted
:methods: test
:verifies: REQ-JSON-OUTPUT
:verifies: REQ-LIST-FILTERS
:verifies: REQ-BODY-EDITS

Run every command with `--json` through the real binary. The output parses as one document with the fields documented in [[DES-CLI]]. Failures return an error object and exit status 1, and invalid usage exits with 2. Each list filter alone and in combination, search and `--all` select the expected issues, in the order defined by [[DES-READINESS]]. Commands run outside a project fail with `project_not_found`. Replacing text that is absent or occurs more than once fails without writing. An update that combines replacements, an append and metadata changes is all or nothing. Body input from standard input works for create and update.
:::

:::mara verification VER-SURFACE-PARITY
:mid: 01M764FJ3Z01GJN91JB409C3GW
:title: Match CLI and MCP behaviour
:status: accepted
:methods: test
:verifies: REQ-SURFACE-PARITY
:verifies: REQ-MCP-STDIO

A real MCP client talks to `zban mcp` over stdio and runs a create, block, ready, update, close and check flow. For the same requests against identical fixtures, the tool results equal the CLI's JSON output, and invalid arguments produce the same error codes. The tool list matches [[DES-MCP-TOOLS]]. Started outside a project, the server serves its instructions and every tool fails with `project_not_found`.
:::

:::mara verification VER-AGENT-GUIDE
:mid: 01M764FJ3ZZ9YTCWGRGCH00BBV
:title: Keep the agent guide consistent
:status: accepted
:methods: test
:methods: inspection
:verifies: REQ-AGENT-GUIDE

Snapshot tests cover `zban prime` and the MCP server instructions, both rendered from the same template. The vocabulary lists in the guide equal the enumerations in the code, and every command or tool the guide names exists. `zban prime` outside a project prints nothing and exits 0. Inspection confirms the guide covers the sections in [[DES-AGENT-GUIDE]] and stays under about 150 lines.
:::

## Outcome check

:::mara validation VAL-DOGFOOD
:mid: 01M764FJ3ZND8RM7NMZNT08MP6
:title: Track zban's own development with zban
:status: accepted
:methods: demonstration
:validates: GOAL-AGENT-WORKFLOW
:validates: GOAL-REPO-ISSUES
:exercises: SCN-AGENT-SESSION
:exercises: SCN-PARALLEL-BRANCHES

After the CLI works, zban's own backlog is managed only through zban. Over at least ten issues worked on by agents, including some in parallel worktrees, review the history. Agents found ready work and kept checklists and summaries without prompting beyond the guide. Discovered work was filed as issues. No issue needed a manual frontmatter repair other than merge conflicts. `zban check` stayed clean on `main`. Record shortfalls as issues or as changes to this knowledge.
:::
