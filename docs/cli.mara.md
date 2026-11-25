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
zban list --ready --not-tag maintainer      # ready work, without issues tagged maintainer
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
and listed in [DES-CLI](#01M764FJ3Z6N3M80E0PMQZXSD4); agents can branch on them, for
example retrying after `conflict`.

## Requirements

---

<a id="01M764FJ3ZR331SYJPBK6THY9V"></a>

### Give machine-readable results for every command

| Type | Requirement |
| --- | --- |
| ID | REQ-JSON-OUTPUT |
| Status | Accepted |
| Satisfies | [NEED-FIND-NEXT](product.mara.md#01M764FJ3ZKQYBEXD73381XSPQ) |
| Satisfies | [NEED-TRACK-PROGRESS](product.mara.md#01M764FJ3Z6S4M6YKX2NXB78TK) |

Every command except `mcp` accepts `--json` and then writes exactly one JSON document to standard output, with documented and stable field names. A failure produces `{"error": {"code": "...", "message": "..."}}` with a documented, stable code and a non-zero exit status. Without `--json`, output is formatted for humans.

---

<a id="01M764FJ3Z8MJPVMEH31KKANCY"></a>

### Find issues by their fields

| Type | Requirement |
| --- | --- |
| ID | REQ-LIST-FILTERS |
| Status | Accepted |
| Satisfies | [NEED-FIND-NEXT](product.mara.md#01M764FJ3ZKQYBEXD73381XSPQ) |
| Satisfies | [NEED-PROJECT-MEMORY](product.mara.md#01M764FJ3Z76TNN9PYF84SMRNB) |

Listing filters by status, type, priority, tag, parent and readiness, excludes issues by tag, and searches title and body case-insensitively. The parent filter matches direct children only. Repeated values of one filter match any of them, and different filters must all match. The tag exclusion leaves out every issue that carries any of its tags, even one that the other filters, the tag filter included, select. A status, type, priority or tag value that no issue could carry is refused rather than matching nothing. Closed issues are excluded unless the request includes them or asks for a closed status. Showing accepts several IDs in one request.

---

<a id="01M764FJ3ZZC411PJMKE9WERPY"></a>

### Edit issue bodies precisely

| Type | Requirement |
| --- | --- |
| ID | REQ-BODY-EDITS |
| Status | Accepted |
| Satisfies | [NEED-TRACK-PROGRESS](product.mara.md#01M764FJ3Z6S4M6YKX2NXB78TK) |

An update can replace the whole body, append text to it, and replace exact text that occurs exactly once, failing when the text is absent or ambiguous. Body edits and metadata changes in one request are applied all or nothing. Multi-line input can be read from standard input.

---

<a id="01M764FJ3ZGF33SBBZTSCZ1FTM"></a>

### Offer the same operations through CLI and MCP

| Type | Requirement |
| --- | --- |
| ID | REQ-SURFACE-PARITY |
| Status | Accepted |
| Satisfies | [NEED-FIND-NEXT](product.mara.md#01M764FJ3ZKQYBEXD73381XSPQ) |
| Satisfies | [NEED-TRACK-PROGRESS](product.mara.md#01M764FJ3Z6S4M6YKX2NXB78TK) |

The command line and the MCP server expose the same operations with the same validation, results and error codes. Neither invokes the other.

---

## Design

---

<a id="01M764FJ3Z6N3M80E0PMQZXSD4"></a>

### Command set

| Type | Design |
| --- | --- |
| ID | DES-CLI |
| Status | Accepted |
| Satisfies | [REQ-JSON-OUTPUT](#01M764FJ3ZR331SYJPBK6THY9V) |
| Satisfies | [REQ-LIST-FILTERS](#01M764FJ3Z8MJPVMEH31KKANCY) |
| Satisfies | [REQ-BODY-EDITS](#01M764FJ3ZZC411PJMKE9WERPY) |

| Command | Arguments and options |
| --- | --- |
| `init` | `[--prefix P]` |
| `create <title>` | `-t TYPE`, `-s STATUS`, `-p PRIORITY`, `--tag T`…, `--parent ID`, `--blocked-by ID`…, `--related ID`…, `--discovered-from ID`, `--body TEXT\|-` |
| `list` | `-s, --status S`…, `-t, --type T`…, `-p, --priority P`…, `--tag T`…, `--not-tag T`…, `--parent ID` (direct children), `--ready`, `--search TEXT`, `--all` |
| `show <id>…` | — |
| `update <id>` | `-s`, `-t`, `-p`, `--title`, `--add-tag`/`--remove-tag`, `--parent ID`/`--no-parent`, `--add-blocked-by`/`--remove-blocked-by`, `--add-related`/`--remove-related`, `--discovered-from ID`/`--no-discovered-from`, `--body TEXT\|-`, `--append TEXT\|-`, `--replace-old TEXT --replace-new TEXT` (repeatable, applied in order), `--handoff TEXT\|-`/`--no-handoff` ([DES-HANDOFF](agents.mara.md#01M777QQGMQNB89NDDMPGRTTFA)), `--force` |
| `delete <id>` | — |
| `check` | `--fix` ([DES-CHECK](issues.mara.md#01M779N2S6013TMF0ZXVSNVZBN)) |
| `prime` | Prints the agent guide ([DES-AGENT-GUIDE](agents.mara.md#01M764FJ3ZPFMKF31AGPP8X9PA)). Outside a project it prints nothing and exits 0, so global hooks are harmless. With `--json` it prints `{"guide": "..."}`, with `null` outside a project. |
| `mcp` | Serves MCP over stdio ([DES-MCP-TOOLS](mcp.mara.md#01M764FJ3Z6VP70QDFVWGMZFB8)). Takes no `--json`. |

In `create`, `list` and `update`, `-s`, `-t` and `-p` are short for `--status`, `--type` and `--priority`. `list --not-tag` leaves out every issue that carries any of its tags, also when the issue carries a tag of `--tag`, and composes with every other filter. The values of `list --tag` and `--not-tag` must be valid tags ([DES-ISSUE-FILE](issues.mara.md#01M764FJ3ZZGJNFXW9VV1NNVAD)), or the request fails with `invalid_value`, as an unknown status, type or priority does: no readable issue can carry an invalid tag, so such a filter would silently select nothing, or exclude nothing. `list --parent` resolves its ID as `show` does, so an ID that is malformed or names no issue fails with `not_found`, and one naming two files or an unreadable file fails with `ambiguous_id` or `unreadable`. Every command accepts the global option `-C, --directory DIR` ([DES-PROJECT-LAYOUT](issues.mara.md#01M764FJ3ZKR92J7WBBKRNMK2B)), and every command except `mcp` accepts `--json`. At most one option per request reads standard input. An update applies its body edits in the order `--body`, the replacements, `--append`, then the handoff.

Exit status is 0 on success, 1 when an operation fails or `check` finds errors, and 2 for invalid usage. Every command except `init`, `prime` and `mcp` fails with `project_not_found` when no project is found; `mcp` starts anyway and fails each tool call instead ([DES-MCP-TOOLS](mcp.mara.md#01M764FJ3Z6VP70QDFVWGMZFB8)). The error codes are:

| Code | When |
| --- | --- |
| `usage` | Unknown, missing or conflicting arguments, more than one input from standard input, or a relative MCP `directory`. Exit status 2. |
| `project_not_found` | No project is found ([DES-PROJECT-LAYOUT](issues.mara.md#01M764FJ3ZKR92J7WBBKRNMK2B)) |
| `project_exists` | `init` where `.zban/` already exists |
| `invalid_config` | The configuration is malformed or has an unknown `format_version` |
| `not_found` | A named issue does not exist. For an ID in this project's format, with its prefix, the message adds a fixed hint that the issue may exist on another branch or worktree; zban does not look there ([REQ-NO-GIT](issues.mara.md#01M764FJ3ZMN288AGT0HD4V1DW)). Any other value, including an ID with another project's prefix, gets a message showing this project's ID format instead. |
| `ambiguous_id` | A named ID matches a file in both `.zban/` and `.zban/archive/` |
| `unreadable` | The operation needs an issue file that cannot be read ([REQ-DAMAGED-DATA](issues.mara.md#01M779N58NATQJ611QCSYK1PWB)) |
| `invalid_value` | A value breaks [DES-ISSUE-FILE](issues.mara.md#01M764FJ3ZZGJNFXW9VV1NNVAD) or [DES-PROJECT-LAYOUT](issues.mara.md#01M764FJ3ZKR92J7WBBKRNMK2B), such as an unknown status, a multi-line title or an invalid prefix |
| `invalid_reference` | A relation names an unknown issue or the issue itself |
| `cycle` | A relation would create a cycle or a blocker on an ancestor ([REQ-RELATIONS](issues.mara.md#01M764FJ3ZZZJS6M31EWC6TJMG)) |
| `unchecked_items` | `done` with unchecked task items, without force |
| `open_children` | `done` or `cancelled` with open children, without force |
| `referenced` | Deleting an issue that another issue references |
| `text_not_found` | A replacement's old text does not occur in the body |
| `text_ambiguous` | A replacement's old text occurs more than once |
| `invalid_handoff` | Handoff text contains a level 1 or 2 heading or leaves a fenced code block open, the written note would not read back, or the body has several handoff sections ([DES-HANDOFF](agents.mara.md#01M777QQGMQNB89NDDMPGRTTFA)) |
| `conflict` | The file changed between reading and writing ([REQ-NO-LOST-UPDATES](issues.mara.md#01M76XFGSJMTW8EQ1TN4W3KV1F)) |
| `io` | Any other file-system failure |

In JSON, an issue has `id`, `title`, `status`, `type`, `priority`, `tags`, `parent`, `blocked_by`, `related`, `discovered_from`, `created_at`, `closed_at`, `path` (the absolute path of the file), `checklist` (`{checked, total}`), `child_counts` (`{closed, total}`, counting the direct children only, the same issues as `show`'s `children` and `list --all --parent`; a child counts as closed when its status is `done` or `cancelled`, whether or not its file is archived), `ready` and `not_ready` (the reasons from [DES-READINESS](issues.mara.md#01M764FJ3ZXCS66TCPPJ3HF04V)). A child whose file is unreadable or whose ID names two files is left out of `children` and of `child_counts` and is reported as a warning ([REQ-DAMAGED-DATA](issues.mara.md#01M779N58NATQJ611QCSYK1PWB)); until it is repaired, closing the parent can still fail with `open_children`, because that check counts such a child as open when a readable copy of it is open. `show` adds `body`, `handoff` and the reverse relations `children`, `blocks`, `related_from` and `discovered` as lists of IDs. `list` adds `handoff_at`. `list` returns `{"issues": [...], "warnings": [...]}` without bodies, in the order defined by [DES-READINESS](issues.mara.md#01M764FJ3ZXCS66TCPPJ3HF04V), and `show` returns `{"issues": [...], "warnings": [...]}` in the order requested. Each warning is `{code, path, message}` with a code from [DES-CHECK](issues.mara.md#01M779N2S6013TMF0ZXVSNVZBN), for a file skipped as unreadable or an ID present twice; human output prints warnings to standard error. Mutations return the resulting issue, and `delete` returns the issue as it was. Human output of `list` and `show` shows checklist progress when the issue has task items and child progress when it has children: on a `list` line `[checked/total]` comes first, then `children closed/total closed`, then the handoff age, and `show` adds the lines `checklist: checked/total checked` and `child_counts: closed/total closed`. `init` returns `{path, prefix, enclosing}`, where `path` is the absolute path of the new `.zban/` and `enclosing` is the absolute path of the enclosing project's `.zban/` ([DES-PROJECT-LAYOUT](issues.mara.md#01M764FJ3ZKR92J7WBBKRNMK2B)), or `null` when there is none; human output then adds a note naming the enclosing project. `check` returns `{"findings": [{severity, code, path, message}]}`, where `path` is the absolute path of the file.

---

<a id="01M764FJ3ZNZPGXYJ4CS2QFHPK"></a>

### One core library with thin adapters

| Type | Design |
| --- | --- |
| ID | DES-SHARED-CORE |
| Status | Accepted |
| Satisfies | [REQ-SURFACE-PARITY](#01M764FJ3ZGF33SBBZTSCZ1FTM) |
| Satisfies | [REQ-PRESERVE-CONTENT](issues.mara.md#01M764FJ3Z0CDR7D6WGHMJYWXK) |
| Satisfies | [REQ-NO-LOST-UPDATES](issues.mara.md#01M76XFGSJMTW8EQ1TN4W3KV1F) |
| Satisfies | [REQ-DAMAGED-DATA](issues.mara.md#01M779N58NATQJ611QCSYK1PWB) |

The crate has a library holding the issue model, file store and operations (`init`, `create`, `list`, `show`, `update`, `delete`, `check` and `guide`), plus the `zban` binary. The CLI and MCP adapters only convert arguments into operation requests and operation results into output. Every operation reads the files fresh. There is no cache and no background process, so hand edits and branch switches are visible immediately, including to a running MCP server. Mutations validate the whole request against the loaded project before writing anything.

Loading never stops at a bad file. A file that cannot be parsed, or whose known keys break [DES-ISSUE-FILE](issues.mara.md#01M764FJ3ZZGJNFXW9VV1NNVAD), is kept as unreadable: it takes no part in listings, and readiness treats it as described in [DES-READINESS](issues.mara.md#01M764FJ3ZXCS66TCPPJ3HF04V). A mutation validates the values the request sets and the relations they form, not problems elsewhere in the project. It fails with `unreadable` when the issue it changes, or an issue its new relations name, is unreadable. Showing fails the same way for a named issue that is unreadable. Both fail with `ambiguous_id` when a named ID has two files.

Each mutation keeps the bytes of every issue file it read. Just before replacing or deleting a file, it reads the file again and fails with `conflict` if the bytes differ. Every step that replaces, moves or deletes an issue file holds an exclusive advisory lock on the `.zban/` directory, taken with `File::lock` on a handle of the directory, from that comparison until the rename or deletion is done ([ADR-WRITE-LOCK](decisions.mara.md#01M77YZD0BXR4H9NYMR7CFMW6R)). Loading the project holds the same lock, taken shared with `File::lock_shared`, from listing `.zban/` and `.zban/archive/` until it has read the bytes of every issue file listed, and parses them after releasing it. A load therefore never overlaps such a step of another zban process: it finds each issue file at the path and with the bytes that the last such step left, and loads do not wait for each other. `check` lists the temporary files under the shared lock too. A step that holds the exclusive lock never loads the project or lists the directories, because locks on two handles conflict even within one process. When taking the lock fails because the file system has no advisory locks (`Unsupported`, or the raw errors `ENOSYS`, `ENOTSUP` and `EOPNOTSUPP`), the step or load runs without it; any other failure to take the lock fails with `io`. Reading the project and writing the temporary file happen before the exclusive lock is taken, and no lock file is created. Two zban processes therefore never both pass the comparison for one file. Editors and Git do not take the lock, so a change they save in the instant between the comparison and the rename can still be lost, and a load that overlaps a rename or move of theirs can still leave a file out. `init` and `create` hard-link the temporary file to its new name, which fails rather than replacing an existing file, so there a create never replaces one. When linking fails because the file system has no hard links, with `EPERM` on Linux or as unsupported in the sense used for the lock above, the step instead holds the exclusive lock while it checks that the name is unused, for an issue in both `.zban/` and `.zban/archive/`, and renames the temporary file to it ([ADR-CREATE-WITHOUT-HARD-LINKS](decisions.mara.md#01M785TJ0TQG6K5K6WJS3G8GBE)); any other failure to link fails with `io`. A create then never replaces a file that exists at the check or that another zban process creates, but an editor, Git, or a zban process on a file system that has no advisory locks either can still create the name between the check and the rename.
