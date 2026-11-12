# Issues

An issue is one Markdown file with a short YAML header. This page explains the
model for users. The specification items below it are the normative contract.

## A quick look

```markdown
---
title: Parse issue files
status: in-progress
type: feature
priority: high
tags:
  - storage
parent: zb-9jra34
blocked_by:
  - zb-naex76
created_at: 2026-11-11T20:00:00Z
---

Load every issue in `.zban/` and `.zban/archive/`.

- [x] Read frontmatter
- [ ] Preserve unknown keys
```

The file is `.zban/zb-n938cw.md`. Its ID, `zb-n938cw`, is the filename, so you can
open any issue from its ID: `.zban/<id>.md`, or `.zban/archive/<id>.md` once it is
closed.

You can edit issues by hand. The header may contain YAML comments, but zban drops
them whenever it rewrites the header, so keep notes in the body.

## Types, statuses and priorities

These sets are fixed for every project ([[ADR-FIXED-VOCABULARY]]).

| Type | Use for |
| --- | --- |
| `epic` | A larger outcome that groups features, tasks and smaller epics, such as phases. Not worked on directly. |
| `feature` | A user-facing capability or enhancement |
| `bug` | Something that is broken |
| `task` | A concrete piece of work: a chore, or a step of a feature |

| Status | Meaning |
| --- | --- |
| `draft` | Captured but not refined. Never offered as ready work. |
| `todo` | Refined and waiting to be started |
| `in-progress` | Being worked on |
| `done` | Finished. Moved to the archive. |
| `cancelled` | Will not be done (duplicate, obsolete, rejected). Moved to the archive. |

Priorities, highest first: `critical`, `high`, `normal` (the default), `low`, `deferred`.

## Relations

- `parent`: the issue this one belongs to, usually an epic or a feature. zban
  does not restrict which types can be parents.
- `blocked_by`: issues that must be closed before this one can start.
- `related`: informational links with no effect on readiness.
- `discovered_from`: the issue that was being worked on when this one was found.

Each relation is written once, on one issue. zban shows the reverse direction
(children, blocks, related, discovered) when you read the other issue.

zban refuses relations that could never resolve: a loop through `parent` and
`blocked_by`, such as an epic blocked by its own child, and an issue blocked by
one of its own ancestors.

## Ready work

`zban list --ready` shows `todo` issues that can start now: their blockers are
closed and nothing above them in the parent chain is closed or blocked. A
`draft` parent does not hold its children back. Epics are never ready, because
their children carry the work. Every listing uses the same order: priority, then
age, then ID. The precise rule is [[DES-READINESS]].

Because readiness follows the whole parent chain, nested epics can split larger
work into phases. A `blocked_by` between sibling epics puts the phases in order:

```text
zb-9jra34  epic  Deliver zban v1
├─ zb-h4p1q7  epic  Phase 1: core
│   └─ tasks, ready now
└─ zb-m2x8r5  epic  Phase 2: integrations   (blocked_by: zb-h4p1q7)
    └─ tasks, ready once Phase 1 is closed
```

Phase 1 can close only after all its children have closed, so Phase 2's work
appears in the ready list exactly when Phase 1 is finished.

## Closing and the archive

Setting `done` or `cancelled` moves the file to `.zban/archive/` and records
`closed_at`. Reopening moves it back. Archived issues still show up in `show`,
in searches, in relations and in `list --all`.

zban refuses to close unfinished work unless you force it:

- `done` while the body still has unchecked `- [ ]` items;
- `done` or `cancelled` while the issue has open children.

`zban create -s done` also refuses a body with unchecked items, and `create`
has no `--force`: create the issue open, then close it with
`zban update ID -s done --force`.

zban never closes children for you. Close or cancel each child with its own
summary or reason first. A forced close leaves the children open, and
`zban check` warns about them.

By convention, a finished issue gets a `## Summary of Changes` section and a
cancelled one gets `## Reasons for Cancelling`. An unfinished issue can carry a
`## Handoff` note for the next session; see [Handoffs](agents.mara.md#handoffs).

## Simultaneous changes

If an issue file changes between zban reading it and writing it, for example
because another agent or an editor saved it, zban stops with a `conflict` error
instead of overwriting the other change. Show the issue again and retry.

## Broken files

A file zban cannot read, for example one left with Git conflict markers after a
merge, does not stop work on the other issues. `list` and `show` skip it with a
warning, and changes to other issues still work. Fix the file by hand, then run
`zban check`, which lists every problem with a stable code ([[DES-CHECK]]):

```text
$ zban check
.zban/zb-k3m9x2.md: error: invalid YAML on line 3: mapping values are not allowed in this context [unparseable]
.zban/zb-n938cw.md: warning: unknown key `assignee` [unknown_key]
```

`zban check` exits with status 1 when it finds an error. Warnings alone, such as
an open issue under a closed parent, exit 0. `zban check --fix` moves each file
that is in the wrong directory for its status, for example after an interrupted
close, deletes the `.zban-*.tmp` files that interrupted writes left behind, and
leaves every other problem for you to fix.

## Requirements

:::mara requirement REQ-ONE-FILE
:mid: 01M764FJ3ZF9DQZ05TX19YZKPY
:title: Store each issue as one Markdown file
:status: accepted
:satisfies: NEED-READABLE-FILES

Each issue is stored as exactly one UTF-8 Markdown file with a YAML frontmatter header under the project's `.zban/` directory. zban needs no other state, such as a database, index or cache, to operate on issues.
:::

:::mara requirement REQ-PRESERVE-CONTENT
:mid: 01M764FJ3Z0CDR7D6WGHMJYWXK
:title: Preserve content an operation does not change
:status: accepted
:satisfies: NEED-READABLE-FILES
:satisfies: NEED-INTEGRITY

When zban rewrites an issue, the body bytes stay unchanged unless the operation edits the body, and frontmatter keys zban does not know are kept. Each file write replaces the file atomically, so an interrupted write never leaves a partially written issue.
:::

:::mara requirement REQ-NO-LOST-UPDATES
:mid: 01M76XFGSJMTW8EQ1TN4W3KV1F
:title: Never overwrite a concurrent change
:status: accepted
:satisfies: NEED-INTEGRITY

When an issue file changes between zban reading it and replacing it, for example through another zban process, an editor or a branch switch, zban does not replace it and the operation fails with the error code `conflict`. Creating an issue never replaces an existing file. zban takes no locks.
:::

:::mara requirement REQ-DAMAGED-DATA
:mid: 01M779N58NATQJ611QCSYK1PWB
:title: Keep working when some issue data is broken
:status: accepted
:satisfies: NEED-INTEGRITY
:satisfies: NEED-BRANCH-SAFE

An issue file that cannot be parsed, or invalid data already in the project, such as a reference to a missing issue after a merge, does not stop work on the other issues. Listing and showing skip unreadable files and report each one as a warning. A mutation validates only what the request changes, and fails only when it must read or write an unreadable file. An ID that names more than one file is reported as ambiguous rather than resolved by guessing. `zban check` reports every such problem.
:::

:::mara requirement REQ-MERGE-SAFE-IDS
:mid: 01M764FJ3ZQ5N4A9NJC8NNK6SA
:title: Create issue IDs without coordination
:status: accepted
:satisfies: NEED-BRANCH-SAFE

zban assigns issue IDs without consulting other branches, checkouts or services. Independently created issues practically never share an ID, a new ID never repeats one already present in the local project, and any duplicates after a merge are reported by [[REQ-CHECK]].
:::

:::mara requirement REQ-NO-GIT
:mid: 01M764FJ3ZMN288AGT0HD4V1DW
:title: Leave version control to the user
:status: accepted
:satisfies: NEED-BRANCH-SAFE

zban never runs Git or another version-control command, and it writes only inside the project's `.zban/` directory. Issue changes reach other branches only through the user's ordinary commits and merges. zban works the same in a directory that is not a Git repository.
:::

:::mara requirement REQ-FIXED-VOCABULARY
:mid: 01M764FJ3ZJGRCCPD3K9XH12B0
:title: Use a fixed status, type and priority vocabulary
:status: accepted
:satisfies: NEED-ORGANIZE

Status is one of `draft`, `todo`, `in-progress`, `done`, `cancelled`. Type is one of `epic`, `feature`, `bug`, `task`. Priority is one of `critical`, `high`, `normal`, `low`, `deferred`. zban rejects other values on write and reports them when checking. Projects cannot change these sets.
:::

:::mara requirement REQ-RELATIONS
:mid: 01M764FJ3ZZZJS6M31EWC6TJMG
:title: Relate issues to each other
:status: accepted
:satisfies: NEED-ORGANIZE

An issue can name at most one parent of any type, any number of blocking issues, any number of related issues, and at most one issue it was discovered from. zban rejects references to unknown issues, references to the issue itself, cycles through `parent` and `blocked_by` edges taken together, and a `blocked_by` that names one of the issue's own ancestors, because each of these can never resolve without forcing a close. Reading an issue shows both directions of each relation. Deleting an issue that another issue references is refused.
:::

:::mara requirement REQ-READY
:mid: 01M764FJ3ZGP807YGG3R6CBPNN
:title: Report ready work and why other work waits
:status: accepted
:satisfies: NEED-FIND-NEXT

zban lists the issues that can be started now in a deterministic priority order. For any open issue it explains why the issue is not ready, naming the responsible status, type, blocker or ancestor.
:::

:::mara requirement REQ-CLOSE-CHECKLIST
:mid: 01M764FJ3Z4DX6RS09TWKY84QC
:title: Do not close unfinished work
:status: accepted
:satisfies: NEED-TRACK-PROGRESS

Setting an issue to `done` while its body contains unchecked Markdown task items is refused unless the request explicitly forces it. Setting an issue to `done` or `cancelled` while it has open children is refused unless forced, and zban never closes the children itself. Cancelling with unchecked task items is not restricted. Listings show checklist progress as checked and total item counts.
:::

:::mara requirement REQ-AUTO-ARCHIVE
:mid: 01M764FJ3ZKD4N35SRK3JR3Y3B
:title: Archive closed issues automatically
:status: accepted
:satisfies: NEED-PROJECT-MEMORY

Closing an issue as `done` or `cancelled` moves its file to `.zban/archive/`, and reopening it moves the file back to `.zban/`. Archived issues remain available to show, search and relations, and are excluded from default listings.
:::

:::mara requirement REQ-CHECK
:mid: 01M764FJ3ZBK44WBWNMZ4ZK98W
:title: Detect and repair inconsistent issue data
:status: accepted
:satisfies: NEED-INTEGRITY
:mitigates: RISK-ID-COLLISION

`zban check` examines the whole project and reports each problem with its file and a stable code: malformed configuration, filenames that are not valid IDs, unparseable files, missing required fields, invalid values, unknown keys, duplicate IDs, references to unknown issues or to the issue itself, cycles and blockers on an ancestor as defined in [[REQ-RELATIONS]], a `related` link stored on both issues, open issues under a closed parent, files in the wrong directory for their status, and temporary files left behind by an interrupted write. Unknown keys, a `related` link stored on both issues, an open issue under a closed parent and a leftover temporary file are warnings. Every other finding is an error, and only errors make the check fail. `zban check --fix` repairs only file placement and deletes leftover temporary files. Every other finding is left for a human or agent to edit.
:::

## Design

:::mara design DES-PROJECT-LAYOUT
:mid: 01M764FJ3ZKR92J7WBBKRNMK2B
:title: Project directory layout
:status: accepted
:satisfies: REQ-ONE-FILE
:satisfies: REQ-AUTO-ARCHIVE

```text
.zban/
  config.toml
  zb-k3m9x2.md      open issues
  archive/
    zb-7hq2pd.md    closed issues
```

`config.toml` has exactly two keys:

```toml
format_version = 1   # format of the .zban/ directory; zban refuses formats it does not know
prefix = "zb"        # ID prefix: 1-8 lowercase ASCII letters or digits, starting with a letter
```

Unknown configuration keys are errors. `zban init` creates the directory and the configuration. The prefix is `zb` unless `--prefix` gives another one; a value that breaks the prefix rule above is rejected. zban finds the project by walking up from the working directory to the nearest `.zban/config.toml`, as far as the filesystem root. The global `-C, --directory DIR` option makes zban behave as if started in `DIR`: the search starts there, and `init` creates `.zban/` there.

Every `*.md` file directly inside `.zban/` or `.zban/archive/` is an issue. Other files and subdirectories are ignored, except that `zban check` reports the temporary files of interrupted writes ([[DES-CHECK]]). Open issues (`draft`, `todo`, `in-progress`) belong in `.zban/`, and closed issues (`done`, `cancelled`) belong in `.zban/archive/`.
:::

:::mara design DES-ISSUE-ID
:mid: 01M764FJ3ZP2NBQPN65QHVH1XY
:title: Issue identifiers and filenames
:status: accepted
:satisfies: REQ-MERGE-SAFE-IDS

An ID is `<prefix>-` followed by six characters from the lowercase Crockford base32 alphabet `0123456789abcdefghjkmnpqrstvwxyz`, drawn from operating-system randomness. That gives 2^30 (about 1.07 billion) IDs per prefix. When a generated ID already exists in `.zban/` or `.zban/archive/`, zban draws again.

The filename is `<id>.md` and nothing else. The ID is read from the filename and is not repeated in the frontmatter, so it has a single source, and an issue's path follows from its ID and status alone. A Markdown file in `.zban/` or `.zban/archive/` whose name is not a valid ID is reported by `zban check`. Input IDs are matched case-insensitively.
:::

:::mara design DES-ISSUE-FILE
:mid: 01M764FJ3ZZGJNFXW9VV1NNVAD
:title: Issue file format
:status: accepted
:satisfies: REQ-ONE-FILE
:satisfies: REQ-PRESERVE-CONTENT
:satisfies: REQ-FIXED-VOCABULARY
:satisfies: REQ-RELATIONS
:mitigates: RISK-MERGE-CONFLICTS

A file starts with a line `---`, then a YAML mapping, then a closing `---` line. One blank line directly after the closing line separates header and body and is not part of the body. Everything after it is the body: free-form Markdown that zban treats as opaque text. The only exceptions are task-item counting in [[DES-LIFECYCLE]] and the handoff section in [[DES-HANDOFF]].

| Key | Required | Value |
| --- | --- | --- |
| `title` | yes | Non-empty single-line string |
| `status` | yes | A status from [[REQ-FIXED-VOCABULARY]] |
| `type` | yes | A type from [[REQ-FIXED-VOCABULARY]] |
| `priority` | no | A priority; omitted means `normal`, and zban omits `normal` when writing |
| `tags` | no | List of distinct lowercase words: letters, digits and single hyphens, starting with a letter |
| `parent` | no | One issue ID |
| `blocked_by` | no | List of distinct issue IDs |
| `related` | no | List of distinct issue IDs. The link is symmetric and stored on one of the two issues. |
| `discovered_from` | no | One issue ID |
| `created_at` | yes | Timestamp, set on creation |
| `closed_at` | when closed | Timestamp. Present exactly while the status is `done` or `cancelled`. |

Timestamps are RFC 3339 in UTC, written to whole seconds with a `Z` suffix, such as `2026-11-11T20:00:00Z`. Files use LF line endings.

Adding a `related` link that already exists in either direction changes nothing. Removing a link edits whichever issue stores it, which may be the other issue.

The header must be plain YAML: no anchors, aliases, custom tags, merge keys or duplicate keys. Hand-written headers may use plain, single-quoted or double-quoted scalars, block or flow lists, and comments. When zban writes a header, it uses the key order above, writes a string as a plain scalar when a YAML 1.2 parser reads it back as the same string and double-quoted otherwise, writes lists in block style with one item per line indented by two spaces, and places unknown keys, preserved verbatim, after the known ones. Comments are not kept when zban rewrites a header. Empty lists and absent optional values are omitted. There is no `updated_at`; Git history records changes ([[ADR-MERGE-FRIENDLY-FORMAT]]).

When zban writes a file, it writes the closing line, one blank line and the body, or ends the file after the closing line when the body is empty. Text given as a new body, appended text and a handoff note are stored ending with exactly one LF. Appending to a non-empty body joins the text after exactly one blank line.
:::

:::mara design DES-READINESS
:mid: 01M764FJ3ZXCS66TCPPJ3HF04V
:title: Readiness and ordering
:status: accepted
:satisfies: REQ-READY

An issue is ready when all of these hold:

1. its status is `todo`;
2. its type is not `epic`;
3. every `blocked_by` target is closed;
4. no ancestor in its `parent` chain is closed;
5. no ancestor has a `blocked_by` target that is still open.

Only closed or blocked ancestors hold an issue back. An ancestor's type, or its status `draft` or `in-progress`, does not. An unknown or unreadable issue ([[REQ-DAMAGED-DATA]]) counts as open: as a blocker it holds its dependents back, and as a parent it ends the ancestor chain. Ready issues, like every listing, are ordered by priority (`critical` first), then `created_at` (oldest first), then ID. For an open issue that is not ready, zban reports every applicable reason: `status`, `epic`, `blocked_by:<id>`, `ancestor_closed:<id>` and `ancestor_blocked:<id>`. In `blocked_by:<id>` the ID is the open blocker. In `ancestor_closed:<id>` and `ancestor_blocked:<id>` it is the ancestor, one reason per affected ancestor; showing that ancestor tells what blocks it.
:::

:::mara design DES-LIFECYCLE
:mid: 01M764FJ3ZPNXY2WFB1Y624PFS
:title: Status changes and archiving
:status: accepted
:satisfies: REQ-CLOSE-CHECKLIST
:satisfies: REQ-AUTO-ARCHIVE
:satisfies: REQ-PRESERVE-CONTENT

Any status can change to any other status; zban enforces no workflow beyond these rules. New issues default to status `todo`, type `task` and priority `normal`.

- **Done with open tasks.** A task item is a list line starting with `- [ ]`, `* [ ]` or `+ [ ]` (unchecked) or the same with `[x]` or `[X]` (checked), at any indentation and outside fenced code blocks. Changing the status to `done` while an unchecked item exists fails unless forced.
- **Fenced code blocks.** A fence line holds a run of at least three backticks or tildes, with no backtick after a backtick run, indented by at most three columns more than its margin: the content of the list item holding the line, or the line start outside list items. A tab advances to the next multiple of four columns. A fence line opens a block, which ends at a fence line of the same character, at least as long and with nothing after it, or at the end of the body. A list item starts at a line that, within the same indentation limit, begins with `-`, `*`, `+`, or one to nine digits and `.` or `)`, followed by a space, a tab or the line end; the text after the marker is read like a line of its own, so it can start a nested item or a fence. The item's content starts after the marker and the whitespace that follows it, or one column after the marker when that whitespace is wider than four columns or ends the line. The item holds the following lines that are blank or indented at least to its content, and any other line ends it. Lines inside a block neither start nor end list items, so a closing fence has the margin of the opening one.
- **Close with open children.** Changing the status to `done` or `cancelled` while an issue whose `parent` is this one is open fails unless forced. The error names the open children.
- **Checked state.** Both rules apply to the issue as the request leaves it, so one update can check the last task item, append a summary and set `done`.
- **Creating closed.** Creating an issue as `done` while it has an unchecked task item fails the same way, and `create` cannot be forced. To record such an issue, create it open, then force setting `done` with an update. A new issue has no children, so creating it closed never fails for open children.
- **Closing.** Entering `done` or `cancelled` sets `closed_at`. Leaving them removes it. Closing never changes children or dependents ([[ADR-NO-CASCADE]]).
- **Placement.** After a status change crosses between open and closed, zban first rewrites the file atomically in place (a temporary file renamed over the original), then renames it into the other directory. If the second step is interrupted, the file has the right content in the wrong place, which `zban check --fix` repairs.
:::

:::mara design DES-CHECK
:mid: 01M779N2S6013TMF0ZXVSNVZBN
:title: Check findings and repair
:status: accepted
:satisfies: REQ-CHECK

`zban check` reports findings ordered by path, then code. In JSON each finding is `{severity, code, path, message}`; human output prints one line per finding.

| Code | Severity | Finding |
| --- | --- | --- |
| `invalid_config` | error | `config.toml` is unparseable, has an unknown key or `format_version`, or an invalid prefix. When it is reported, check reports nothing else. |
| `invalid_filename` | error | A Markdown file whose name is not a valid ID ([[DES-ISSUE-ID]]) |
| `unparseable` | error | No frontmatter, invalid YAML, or YAML outside the plain subset of [[DES-ISSUE-FILE]], such as Git conflict markers |
| `missing_field` | error | A required key is absent, including `closed_at` on a closed issue |
| `invalid_value` | error | A value breaks [[DES-ISSUE-FILE]], including `closed_at` on an open issue |
| `unknown_key` | warning | A frontmatter key zban does not know |
| `duplicate_id` | error | One ID names a file in both `.zban/` and `.zban/archive/` |
| `invalid_reference` | error | A relation names an unknown issue or the issue itself. A reference to an existing but unreadable file is not reported again. |
| `cycle` | error | The issue is on a cycle through `parent` and `blocked_by`, or is blocked by one of its own ancestors ([[REQ-RELATIONS]]) |
| `related_both_sides` | warning | The same `related` link is stored on both issues |
| `open_under_closed` | warning | An open issue whose parent is closed |
| `misplaced` | error | A file in the wrong directory for its status |
| `temporary_file` | warning | A file named `.zban-*.tmp` directly inside `.zban/` or `.zban/archive/`. zban writes through such files ([[REQ-PRESERVE-CONTENT]]), and an interrupted write leaves one behind. |

`--fix` moves each misplaced file into the directory its status requires, unless a file already exists at the target path; that finding then remains. It also deletes each leftover temporary file. The fixed findings are not reported. Because zban takes no locks ([[REQ-NO-LOST-UPDATES]]), `--fix` can delete the temporary file of a write that is still running; that write then fails with `io` and leaves its issue unchanged.
:::

## Scenario

:::mara scenario SCN-PARALLEL-BRANCHES
:mid: 01M764FJ3Z89Y0KG9KKRR216PZ
:title: Two agents change issues on separate branches
:status: accepted

Two agents work in separate worktrees of one repository. Each creates new issues and updates an issue the other does not touch. One agent closes its issue, which moves the file into the archive. Both branches merge through ordinary pull requests without conflicts, because new issues have distinct filenames and the edited issues are different files. A conflict appears only when both branches change the same lines of the same issue, and it is resolved like any text conflict. After the merges, `zban check` reports no problems. Neither agent saw the other's status changes before the merge.
:::
