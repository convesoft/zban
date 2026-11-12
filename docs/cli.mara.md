# Command line

`zban` is a single binary. Every command prints human-readable output by default
and one JSON document with `--json`. Run `zban <command> --help` for full options.

## Common tasks

```sh
zban init                                   # create .zban/ here, with ID prefix zb
zban init --prefix web                      # ... or with your own prefix
zban create "Parse issue files" -t feature -p high --parent zb-9jra34
zban create "Crash on empty title" -t bug --discovered-from zb-n938cw --body - < notes.md
zban list --ready                           # what can be started now
zban list --status in-progress              # what is being worked on
zban list --parent zb-9jra34                # direct children of an epic
zban list --type bug --tag storage --search "frontmatter"
zban show zb-n938cw zb-2b3mf7               # full issues, including body and reverse relations
zban update zb-n938cw -s in-progress
zban update zb-n938cw --replace-old "- [ ] Preserve unknown keys" --replace-new "- [x] Preserve unknown keys"
zban update zb-n938cw --append "## Summary of Changes

Parser and writer implemented." -s done
zban update zb-n938cw --handoff - < handoff.md   # replace the handoff note
zban check                                  # report problems in .zban/
zban check --fix                            # move misplaced files, delete leftover temporary files
```

`create`, `list` and `update` take `-s`, `-t` and `-p` as short forms of
`--status`, `--type` and `--priority`, so `zban list -t bug -p high` lists the
open high-priority bugs.

Global options are `--json` and `-C, --directory DIR`. zban finds its project by
searching upward from the current directory to the nearest `.zban/`. With
`-C DIR` it behaves as if started in `DIR`, so any directory inside a project
works:

```sh
zban -C ../other-repo list --ready
```

`zban init` inside an existing project creates a nested project, and commands
run at or below it then use the nested project only. `init` prints a note naming
the enclosing project, and returns its `.zban/` as `enclosing` with `--json`.

## Errors

Exit status is 0 on success, 1 when an operation fails or `check` finds errors,
and 2 for invalid usage. With `--json`, a failure prints one object such as
`{"error": {"code": "unchecked_items", "message": "..."}}`. The codes are stable
and listed in [[DES-CLI]]; agents can branch on them, for
example retrying after `conflict`.

## Requirements

:::mara requirement REQ-JSON-OUTPUT
:mid: 01M764FJ3ZR331SYJPBK6THY9V
:title: Give machine-readable results for every command
:status: accepted
:satisfies: NEED-FIND-NEXT
:satisfies: NEED-TRACK-PROGRESS

Every command except `mcp` accepts `--json` and then writes exactly one JSON document to standard output, with documented and stable field names. A failure produces `{"error": {"code": "...", "message": "..."}}` with a documented, stable code and a non-zero exit status. Without `--json`, output is formatted for humans.
:::

:::mara requirement REQ-LIST-FILTERS
:mid: 01M764FJ3Z8MJPVMEH31KKANCY
:title: Find issues by their fields
:status: accepted
:satisfies: NEED-FIND-NEXT
:satisfies: NEED-PROJECT-MEMORY

Listing filters by status, type, priority, tag, parent and readiness, and searches title and body case-insensitively. The parent filter matches direct children only. Repeated values of one filter match any of them, and different filters must all match. Closed issues are excluded unless the request includes them or asks for a closed status. Showing accepts several IDs in one request.
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
| `list` | `-s, --status S`…, `-t, --type T`…, `-p, --priority P`…, `--tag T`…, `--parent ID` (direct children), `--ready`, `--search TEXT`, `--all` |
| `show <id>…` | — |
| `update <id>` | `-s`, `-t`, `-p`, `--title`, `--add-tag`/`--remove-tag`, `--parent ID`/`--no-parent`, `--add-blocked-by`/`--remove-blocked-by`, `--add-related`/`--remove-related`, `--discovered-from ID`/`--no-discovered-from`, `--body TEXT\|-`, `--append TEXT\|-`, `--replace-old TEXT --replace-new TEXT` (repeatable, applied in order), `--handoff TEXT\|-`/`--no-handoff` ([[DES-HANDOFF]]), `--force` |
| `delete <id>` | — |
| `check` | `--fix` ([[DES-CHECK]]) |
| `prime` | Prints the agent guide ([[DES-AGENT-GUIDE]]). Outside a project it prints nothing and exits 0, so global hooks are harmless. With `--json` it prints `{"guide": "..."}`, with `null` outside a project. |
| `mcp` | Serves MCP over stdio ([[DES-MCP-TOOLS]]). Takes no `--json`. |

In `create`, `list` and `update`, `-s`, `-t` and `-p` are short for `--status`, `--type` and `--priority`. `list --parent` resolves its ID as `show` does, so an ID that is malformed or names no issue fails with `not_found`, and one naming two files or an unreadable file fails with `ambiguous_id` or `unreadable`. Every command accepts the global option `-C, --directory DIR` ([[DES-PROJECT-LAYOUT]]), and every command except `mcp` accepts `--json`. At most one option per request reads standard input. An update applies its body edits in the order `--body`, the replacements, `--append`, then the handoff.

Exit status is 0 on success, 1 when an operation fails or `check` finds errors, and 2 for invalid usage. Every command except `init`, `prime` and `mcp` fails with `project_not_found` when no project is found; `mcp` starts anyway and fails each tool call instead ([[DES-MCP-TOOLS]]). The error codes are:

| Code | When |
| --- | --- |
| `usage` | Unknown, missing or conflicting arguments, more than one input from standard input, or a relative MCP `directory`. Exit status 2. |
| `project_not_found` | No project is found ([[DES-PROJECT-LAYOUT]]) |
| `project_exists` | `init` where `.zban/` already exists |
| `invalid_config` | The configuration is malformed or has an unknown `format_version` |
| `not_found` | A named issue does not exist |
| `ambiguous_id` | A named ID matches a file in both `.zban/` and `.zban/archive/` |
| `unreadable` | The operation needs an issue file that cannot be read ([[REQ-DAMAGED-DATA]]) |
| `invalid_value` | A value breaks [[DES-ISSUE-FILE]] or [[DES-PROJECT-LAYOUT]], such as an unknown status, a multi-line title or an invalid prefix |
| `invalid_reference` | A relation names an unknown issue or the issue itself |
| `cycle` | A relation would create a cycle or a blocker on an ancestor ([[REQ-RELATIONS]]) |
| `unchecked_items` | `done` with unchecked task items, without force |
| `open_children` | `done` or `cancelled` with open children, without force |
| `referenced` | Deleting an issue that another issue references |
| `text_not_found` | A replacement's old text does not occur in the body |
| `text_ambiguous` | A replacement's old text occurs more than once |
| `invalid_handoff` | Handoff text contains a level 1 or 2 heading or leaves a fenced code block open, the written note would not read back, or the body has several handoff sections ([[DES-HANDOFF]]) |
| `conflict` | The file changed between reading and writing ([[REQ-NO-LOST-UPDATES]]) |
| `io` | Any other file-system failure |

In JSON, an issue has `id`, `title`, `status`, `type`, `priority`, `tags`, `parent`, `blocked_by`, `related`, `discovered_from`, `created_at`, `closed_at`, `path` (the absolute path of the file), `checklist` (`{checked, total}`), `ready` and `not_ready` (the reasons from [[DES-READINESS]]). `show` adds `body`, `handoff` and the reverse relations `children`, `blocks`, `related_from` and `discovered` as lists of IDs. `list` adds `handoff_at`. `list` returns `{"issues": [...], "warnings": [...]}` without bodies, in the order defined by [[DES-READINESS]], and `show` returns `{"issues": [...], "warnings": [...]}` in the order requested. Each warning is `{code, path, message}` with a code from [[DES-CHECK]], for a file skipped as unreadable or an ID present twice; human output prints warnings to standard error. Mutations return the resulting issue, and `delete` returns the issue as it was. `init` returns `{path, prefix, enclosing}`, where `path` is the absolute path of the new `.zban/` and `enclosing` is the absolute path of the enclosing project's `.zban/` ([[DES-PROJECT-LAYOUT]]), or `null` when there is none; human output then adds a note naming the enclosing project. `check` returns `{"findings": [{severity, code, path, message}]}`, where `path` is the absolute path of the file.
:::

:::mara design DES-SHARED-CORE
:mid: 01M764FJ3ZNZPGXYJ4CS2QFHPK
:title: One core library with thin adapters
:status: accepted
:satisfies: REQ-SURFACE-PARITY
:satisfies: REQ-PRESERVE-CONTENT
:satisfies: REQ-NO-LOST-UPDATES
:satisfies: REQ-DAMAGED-DATA

The crate has a library holding the issue model, file store and operations (`init`, `create`, `list`, `show`, `update`, `delete`, `check` and `guide`), plus the `zban` binary. The CLI and MCP adapters only convert arguments into operation requests and operation results into output. Every operation reads the files fresh. There is no cache and no background process, so hand edits and branch switches are visible immediately, including to a running MCP server. Mutations validate the whole request against the loaded project before writing anything.

Loading never stops at a bad file. A file that cannot be parsed, or whose known keys break [[DES-ISSUE-FILE]], is kept as unreadable: it takes no part in listings, and readiness treats it as described in [[DES-READINESS]]. A mutation validates the values the request sets and the relations they form, not problems elsewhere in the project. It fails with `unreadable` when the issue it changes, or an issue its new relations name, is unreadable. Showing fails the same way for a named issue that is unreadable. Both fail with `ambiguous_id` when a named ID has two files.

Each mutation keeps the bytes of every issue file it read. Just before replacing or deleting a file, it reads the file again and fails with `conflict` if the bytes differ. Every step that replaces, moves or deletes an issue file holds an exclusive advisory lock on the `.zban/` directory, taken with `File::lock` on a handle of the directory, from that comparison until the rename or deletion is done ([[ADR-WRITE-LOCK]]). Loading the project holds the same lock, taken shared with `File::lock_shared`, from listing `.zban/` and `.zban/archive/` until it has read the bytes of every issue file listed, and parses them after releasing it. A load therefore never overlaps such a step of another zban process: it finds each issue file at the path and with the bytes that the last such step left, and loads do not wait for each other. `check` lists the temporary files under the shared lock too. A step that holds the exclusive lock never loads the project or lists the directories, because locks on two handles conflict even within one process. When taking the lock fails because the file system has no advisory locks (`Unsupported`, or the raw errors `ENOSYS`, `ENOTSUP` and `EOPNOTSUPP`), the step or load runs without it; any other failure to take the lock fails with `io`. Reading the project and writing the temporary file happen before the exclusive lock is taken, and no lock file is created. Two zban processes therefore never both pass the comparison for one file. Editors and Git do not take the lock, so a change they save in the instant between the comparison and the rename can still be lost, and a load that overlaps a rename or move of theirs can still leave a file out. `init` and `create` hard-link the temporary file to its new name, which fails rather than replacing an existing file, so there a create never replaces one. When linking fails because the file system has no hard links, with `EPERM` on Linux or as unsupported in the sense used for the lock above, the step instead holds the exclusive lock while it checks that the name is unused, for an issue in both `.zban/` and `.zban/archive/`, and renames the temporary file to it ([[ADR-CREATE-WITHOUT-HARD-LINKS]]); any other failure to link fails with `io`. A create then never replaces a file that exists at the check or that another zban process creates, but an editor, Git, or a zban process on a file system that has no advisory locks either can still create the name between the check and the rename.
:::
