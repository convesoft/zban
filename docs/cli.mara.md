# Command line

`zban` is a single binary. Every command prints human-readable output by default
and one JSON document with `--json`. Run `zban <command> --help` for full options.

> zban is not implemented yet. This page describes the agreed v1 interface.

## Common tasks

```sh
zban init                                   # create .zban/ here, with ID prefix zb
zban init --prefix web                      # ... or with your own prefix
zban create "Parse issue files" -t feature -p high --parent zb-9jra34
zban create "Crash on empty title" -t bug --discovered-from zb-n938cw --body - < notes.md
zban list --ready                           # what can be started now
zban list --status in-progress              # what is being worked on
zban list --type bug --tag storage --search "frontmatter"
zban show zb-n938cw zb-2b3mf7               # full issues, including body and reverse relations
zban update zb-n938cw -s in-progress
zban update zb-n938cw --replace-old "- [ ] Preserve unknown keys" --replace-new "- [x] Preserve unknown keys"
zban update zb-n938cw --append "## Summary of Changes

Parser and writer implemented." -s done
zban check                                  # report problems in .zban/
zban check --fix                            # move misplaced files
```

Global options are `--json` and `--root PATH`. `PATH` is the directory that
contains `.zban/`; by default zban searches upward from the current directory.

## Requirements

:::mara requirement REQ-JSON-OUTPUT
:mid: 01M764FJ3ZR331SYJPBK6THY9V
:title: Give machine-readable results for every command
:status: accepted
:satisfies: NEED-FIND-NEXT
:satisfies: NEED-TRACK-PROGRESS

Every command accepts `--json` and then writes exactly one JSON document to standard output, with documented and stable field names. A failure produces `{"error": {"code": "...", "message": "..."}}` and a non-zero exit status. Without `--json`, output is formatted for humans.
:::

:::mara requirement REQ-LIST-FILTERS
:mid: 01M764FJ3Z8MJPVMEH31KKANCY
:title: Find issues by their fields
:status: accepted
:satisfies: NEED-FIND-NEXT
:satisfies: NEED-PROJECT-MEMORY

Listing filters by status, type, priority, tag, parent and readiness, and searches title and body case-insensitively. Repeated values of one filter match any of them, and different filters must all match. Closed issues are excluded unless the request includes them or asks for a closed status. Showing accepts several IDs in one request.
:::

:::mara requirement REQ-BODY-EDITS
:mid: 01M764FJ3ZZC411PJMKE9WERPY
:title: Edit issue bodies precisely
:status: accepted
:satisfies: NEED-TRACK-PROGRESS

An update can replace the whole body, append text to it, and replace exact text that occurs exactly once, failing when the text is absent or ambiguous. Body edits and metadata changes in one request are applied all or nothing. Multi-line input can be read from standard input.
:::

:::mara requirement REQ-SURFACE-PARITY
:mid: 01M764FJ3ZGF33SBBZTSCZ1FTM
:title: Offer the same operations through CLI and MCP
:status: accepted
:satisfies: NEED-FIND-NEXT
:satisfies: NEED-TRACK-PROGRESS

The command line and the MCP server expose the same operations with the same validation, results and error codes. Neither invokes the other.
:::

## Design

:::mara design DES-CLI
:mid: 01M764FJ3Z6N3M80E0PMQZXSD4
:title: Command set
:status: accepted
:satisfies: REQ-JSON-OUTPUT
:satisfies: REQ-LIST-FILTERS
:satisfies: REQ-BODY-EDITS

| Command | Arguments and options |
| --- | --- |
| `init` | `[--prefix P]` |
| `create <title>` | `-t TYPE`, `-s STATUS`, `-p PRIORITY`, `--tag T`…, `--parent ID`, `--blocked-by ID`…, `--related ID`…, `--discovered-from ID`, `--body TEXT\|-` |
| `list` | `--status S`…, `--type T`…, `--priority P`…, `--tag T`…, `--parent ID`, `--ready`, `--search TEXT`, `--all` |
| `show <id>…` | — |
| `update <id>` | `-s`, `-t`, `-p`, `--title`, `--add-tag`/`--remove-tag`, `--parent ID`/`--no-parent`, `--add-blocked-by`/`--remove-blocked-by`, `--add-related`/`--remove-related`, `--discovered-from ID`/`--no-discovered-from`, `--body TEXT\|-`, `--append TEXT\|-`, `--replace-old TEXT --replace-new TEXT` (repeatable, applied in order), `--force` |
| `delete <id>` | — |
| `check` | `--fix` |
| `prime` | Prints the agent guide ([[DES-AGENT-GUIDE]]). Outside a project it prints nothing and exits 0, so global hooks are harmless. |
| `mcp` | Serves MCP over stdio ([[DES-MCP-TOOLS]]) |

Exit status is 0 on success, 1 when an operation fails or `check` finds errors, and 2 for invalid usage. Every command except `init` and `prime` fails with the error code `project_not_found` when no project is found. A concurrent change fails with `conflict` ([[REQ-NO-LOST-UPDATES]]).

In JSON, an issue has `id`, `title`, `status`, `type`, `priority`, `tags`, `parent`, `blocked_by`, `related`, `discovered_from`, `created_at`, `closed_at`, `path`, `checklist` (`{checked, total}`), `ready` and `not_ready` (the reasons from [[DES-READINESS]]). `show` adds `body` and the reverse relations `children`, `blocks`, `related_from` and `discovered`. `list` returns `{"issues": [...]}` without bodies, in the order defined by [[DES-READINESS]]. Mutations return the resulting issue. `check` returns `{"findings": [{severity, code, path, message}]}`.
:::

:::mara design DES-SHARED-CORE
:mid: 01M764FJ3ZNZPGXYJ4CS2QFHPK
:title: One core library with thin adapters
:status: accepted
:satisfies: REQ-SURFACE-PARITY
:satisfies: REQ-PRESERVE-CONTENT
:satisfies: REQ-NO-LOST-UPDATES

The crate has a library holding the issue model, file store and operations (`init`, `create`, `list`, `show`, `update`, `delete`, `check` and `guide`), plus the `zban` binary. The CLI and MCP adapters only convert arguments into operation requests and operation results into output. Every operation reads the files fresh. There is no cache and no background process, so hand edits and branch switches are visible immediately, including to a running MCP server. Mutations validate the whole request against the loaded project before writing anything.

Each mutation keeps the bytes of every issue file it read. Just before replacing a file, it reads the file again and fails with `conflict` if the bytes differ. New issue files are created with exclusive creation, so a create never replaces an existing file. This closes the window for lost updates to the moment between that comparison and the rename, without locks.
:::
