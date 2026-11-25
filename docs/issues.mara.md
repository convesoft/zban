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

These sets are fixed for every project ([ADR-FIXED-VOCABULARY](decisions.mara.md#01M764FJ3ZYNNKFADFY3G06GRD)).

| Type | Use for |
| --- | --- |
| `epic` | A larger outcome that groups features, tasks and smaller epics, such as phases. Not worked on directly. |
| `feature` | A user-facing capability or enhancement |
| `bug` | Something that is broken |
| `task` | A concrete piece of work: a chore, or a step of a feature |

| Status | Meaning |
| --- | --- |
| `draft` | Captured but not refined, or parked on purpose. Neither it nor anything under it is offered as ready work. |
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
closed and nothing above them in the parent chain is closed, blocked or a
`draft`. Keep planned epics, backlog and work only a human can do as `draft`,
and move them out of `draft` when their work may start. A parent's status
`in-progress` does not by itself hold its children back. Epics are never ready,
because their children carry the work. Every listing uses the same order:
priority, then age, then ID. The precise rule is [DES-READINESS](#01M764FJ3ZXCS66TCPPJ3HF04V).

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

zban commands that write at the same moment take turns. Each holds a short
lock on `.zban/` while it checks and replaces a file, so when two agents update
one issue together, neither change is lost: at worst one of them gets `conflict`
and retries. The lock lasts only for that step and leaves no file behind.

Commands that read issues share the lock while they list `.zban/` and read the
issue files, so they never miss an issue that another zban command is replacing,
closing or reopening at that moment. Reads do not wait for each other, only for
a write's short step.

## Broken files

A file zban cannot read, for example one left with Git conflict markers after a
merge, does not stop work on the other issues. `list` and `show` skip it with a
warning, and changes to other issues still work. Fix the file by hand, then run
`zban check`, which lists every problem with a stable code ([DES-CHECK](#01M779N2S6013TMF0ZXVSNVZBN)):

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

---

<a id="01M764FJ3ZF9DQZ05TX19YZKPY"></a>

### Store each issue as one Markdown file

| Type | Requirement |
| --- | --- |
| ID | REQ-ONE-FILE |
| Status | Accepted |
| Satisfies | [NEED-READABLE-FILES](product.mara.md#01M764FJ3Z2306WVCH858EK24P) |

Each issue is stored as exactly one UTF-8 Markdown file with a YAML frontmatter header under the project's `.zban/` directory. zban needs no other state, such as a database, index or cache, to operate on issues.

---

<a id="01M764FJ3Z0CDR7D6WGHMJYWXK"></a>

### Preserve content an operation does not change

| Type | Requirement |
| --- | --- |
| ID | REQ-PRESERVE-CONTENT |
| Status | Accepted |
| Satisfies | [NEED-READABLE-FILES](product.mara.md#01M764FJ3Z2306WVCH858EK24P) |
| Satisfies | [NEED-INTEGRITY](product.mara.md#01M764FJ3Z8BV1YNJN9TA5QQDA) |

When zban rewrites an issue, the body bytes stay unchanged unless the operation edits the body, and frontmatter keys zban does not know are kept. Each file write replaces the file atomically, so an interrupted write never leaves a partially written issue.

---

<a id="01M76XFGSJMTW8EQ1TN4W3KV1F"></a>

### Never overwrite a concurrent change

| Type | Requirement |
| --- | --- |
| ID | REQ-NO-LOST-UPDATES |
| Status | Accepted |
| Satisfies | [NEED-INTEGRITY](product.mara.md#01M764FJ3Z8BV1YNJN9TA5QQDA) |

When an issue file changes between zban reading it and replacing it, for example through another zban process, an editor or a branch switch, zban does not replace it and the operation fails with the error code `conflict`. Of several zban processes that change one issue at the same time, each either writes its change without dropping another's or fails with `conflict`. Creating an issue never replaces an existing file. On a file system without hard links, this holds against other zban processes, but not against another program that creates a file under the new issue's name at the same moment.

---

<a id="01M779N58NATQJ611QCSYK1PWB"></a>

### Keep working when some issue data is broken

| Type | Requirement |
| --- | --- |
| ID | REQ-DAMAGED-DATA |
| Status | Accepted |
| Satisfies | [NEED-INTEGRITY](product.mara.md#01M764FJ3Z8BV1YNJN9TA5QQDA) |
| Satisfies | [NEED-BRANCH-SAFE](product.mara.md#01M764FJ3Z4B0WDVBR7JXWBR5B) |

An issue file that cannot be parsed, or invalid data already in the project, such as a reference to a missing issue after a merge, does not stop work on the other issues. Listing and showing skip unreadable files and report each one as a warning. A mutation validates only what the request changes, and fails only when it must read or write an unreadable file. An ID that names more than one file is reported as ambiguous rather than resolved by guessing. `zban check` reports every such problem.

---

<a id="01M764FJ3ZQ5N4A9NJC8NNK6SA"></a>

### Create issue IDs without coordination

| Type | Requirement |
| --- | --- |
| ID | REQ-MERGE-SAFE-IDS |
| Status | Accepted |
| Satisfies | [NEED-BRANCH-SAFE](product.mara.md#01M764FJ3Z4B0WDVBR7JXWBR5B) |

zban assigns issue IDs without consulting other branches, checkouts or services. Independently created issues practically never share an ID, a new ID never repeats one already present in the local project, and any duplicates after a merge are reported by [REQ-CHECK](#01M764FJ3ZBK44WBWNMZ4ZK98W).

---

<a id="01M764FJ3ZMN288AGT0HD4V1DW"></a>

### Leave version control to the user

| Type | Requirement |
| --- | --- |
| ID | REQ-NO-GIT |
| Status | Accepted |
| Satisfies | [NEED-BRANCH-SAFE](product.mara.md#01M764FJ3Z4B0WDVBR7JXWBR5B) |

zban never runs Git or another version-control command, and it writes only inside the project's `.zban/` directory. Issue changes reach other branches only through the user's ordinary commits and merges. zban works the same in a directory that is not a Git repository.

---

<a id="01M764FJ3ZJGRCCPD3K9XH12B0"></a>

### Use a fixed status, type and priority vocabulary

| Type | Requirement |
| --- | --- |
| ID | REQ-FIXED-VOCABULARY |
| Status | Accepted |
| Satisfies | [NEED-ORGANIZE](product.mara.md#01M764FJ3ZG12XTG21J7XKEQFC) |

Status is one of `draft`, `todo`, `in-progress`, `done`, `cancelled`. Type is one of `epic`, `feature`, `bug`, `task`. Priority is one of `critical`, `high`, `normal`, `low`, `deferred`. zban rejects other values on write and reports them when checking. Projects cannot change these sets.

---

<a id="01M764FJ3ZZZJS6M31EWC6TJMG"></a>

### Relate issues to each other

| Type | Requirement |
| --- | --- |
| ID | REQ-RELATIONS |
| Status | Accepted |
| Satisfies | [NEED-ORGANIZE](product.mara.md#01M764FJ3ZG12XTG21J7XKEQFC) |

An issue can name at most one parent of any type, any number of blocking issues, any number of related issues, and at most one issue it was discovered from. zban rejects references to unknown issues, references to the issue itself, cycles through `parent` and `blocked_by` edges taken together, and a `blocked_by` that names one of the issue's own ancestors, because each of these can never resolve without forcing a close. Reading an issue shows both directions of each relation. Deleting an issue that another issue references is refused.

---

<a id="01M764FJ3ZGP807YGG3R6CBPNN"></a>

### Report ready work and why other work waits

| Type | Requirement |
| --- | --- |
| ID | REQ-READY |
| Status | Accepted |
| Satisfies | [NEED-FIND-NEXT](product.mara.md#01M764FJ3ZKQYBEXD73381XSPQ) |

zban lists the issues that can be started now in a deterministic priority order. For any open issue it explains why the issue is not ready, naming the responsible status, type, blocker or ancestor.

---

<a id="01M764FJ3Z4DX6RS09TWKY84QC"></a>

### Do not close unfinished work

| Type | Requirement |
| --- | --- |
| ID | REQ-CLOSE-CHECKLIST |
| Status | Accepted |
| Satisfies | [NEED-TRACK-PROGRESS](product.mara.md#01M764FJ3Z6S4M6YKX2NXB78TK) |

Setting an issue to `done` while its body contains unchecked Markdown task items is refused unless the request explicitly forces it. Setting an issue to `done` or `cancelled` while it has open children is refused unless forced, and zban never closes the children itself. Cancelling with unchecked task items is not restricted. Listings show checklist progress as checked and total item counts.

---

<a id="01M764FJ3ZKD4N35SRK3JR3Y3B"></a>

### Archive closed issues automatically

| Type | Requirement |
| --- | --- |
| ID | REQ-AUTO-ARCHIVE |
| Status | Accepted |
| Satisfies | [NEED-PROJECT-MEMORY](product.mara.md#01M764FJ3Z76TNN9PYF84SMRNB) |

Closing an issue as `done` or `cancelled` moves its file to `.zban/archive/`, and reopening it moves the file back to `.zban/`. Archived issues remain available to show, search and relations, and are excluded from default listings.

---

<a id="01M764FJ3ZBK44WBWNMZ4ZK98W"></a>

### Detect and repair inconsistent issue data

| Type | Requirement |
| --- | --- |
| ID | REQ-CHECK |
| Status | Accepted |
| Satisfies | [NEED-INTEGRITY](product.mara.md#01M764FJ3Z8BV1YNJN9TA5QQDA) |
| Mitigates | [RISK-ID-COLLISION](decisions.mara.md#01M764FJ3Z4YEGWP7Z0Y8VZE9N) |

`zban check` examines the whole project and reports each problem with its file and a stable code: malformed configuration, filenames that are not valid IDs, unparseable files, missing required fields, invalid values, unknown keys, duplicate IDs, references to unknown issues or to the issue itself, cycles and blockers on an ancestor as defined in [REQ-RELATIONS](#01M764FJ3ZZZJS6M31EWC6TJMG), a `related` link stored on both issues, open issues under a closed parent, files in the wrong directory for their status, and temporary files left behind by an interrupted write. Unknown keys, a `related` link stored on both issues, an open issue under a closed parent and a leftover temporary file are warnings. Every other finding is an error, and only errors make the check fail. `zban check --fix` repairs only file placement and deletes leftover temporary files. Every other finding is left for a human or agent to edit.

---

## Design

---

<a id="01M764FJ3ZKR92J7WBBKRNMK2B"></a>

### Project directory layout

| Type | Design |
| --- | --- |
| ID | DES-PROJECT-LAYOUT |
| Status | Accepted |
| Satisfies | [REQ-ONE-FILE](#01M764FJ3ZF9DQZ05TX19YZKPY) |
| Satisfies | [REQ-AUTO-ARCHIVE](#01M764FJ3ZKD4N35SRK3JR3Y3B) |

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

Unknown configuration keys are errors. `zban init` creates the directory and the configuration. When a step after creating the directory fails, such as writing the configuration, `init` removes the directory again if it is still empty, so that `init` can be run again; a `.zban/` that already existed fails with `project_exists` and is left alone. The prefix is `zb` unless `--prefix` gives another one; a value that breaks the prefix rule above is rejected. zban finds the project by walking up from the working directory to the nearest `.zban/config.toml`, as far as the filesystem root. The global `-C, --directory DIR` option makes zban behave as if started in `DIR`: the search starts there, and `init` creates `.zban/` there.

`init` works inside an existing project too and creates a nested project there. Commands run at or below the nested root then find the nested project and no longer see the enclosing one, so `init` names the enclosing project: the nearest directory above the new root that holds `.zban/config.toml` ([DES-CLI](cli.mara.md#01M764FJ3Z6N3M80E0PMQZXSD4)).

Every `*.md` file directly inside `.zban/` or `.zban/archive/` is an issue. Other files and subdirectories are ignored, except that `zban check` reports the temporary files of interrupted writes ([DES-CHECK](#01M779N2S6013TMF0ZXVSNVZBN)). Open issues (`draft`, `todo`, `in-progress`) belong in `.zban/`, and closed issues (`done`, `cancelled`) belong in `.zban/archive/`.

---

<a id="01M764FJ3ZP2NBQPN65QHVH1XY"></a>

### Issue identifiers and filenames

| Type | Design |
| --- | --- |
| ID | DES-ISSUE-ID |
| Status | Accepted |
| Satisfies | [REQ-MERGE-SAFE-IDS](#01M764FJ3ZQ5N4A9NJC8NNK6SA) |

An ID is `<prefix>-` followed by six characters from the lowercase Crockford base32 alphabet `0123456789abcdefghjkmnpqrstvwxyz`, drawn from operating-system randomness. That gives 2^30 (about 1.07 billion) IDs per prefix. When a generated ID already exists in `.zban/` or `.zban/archive/`, zban draws again.

The filename is `<id>.md` and nothing else. The ID is read from the filename and is not repeated in the frontmatter, so it has a single source, and an issue's path follows from its ID and status alone. A Markdown file in `.zban/` or `.zban/archive/` whose name is not a valid ID is reported by `zban check`. Input IDs are matched case-insensitively.

---

<a id="01M764FJ3ZZGJNFXW9VV1NNVAD"></a>

### Issue file format

| Type | Design |
| --- | --- |
| ID | DES-ISSUE-FILE |
| Status | Accepted |
| Satisfies | [REQ-ONE-FILE](#01M764FJ3ZF9DQZ05TX19YZKPY) |
| Satisfies | [REQ-PRESERVE-CONTENT](#01M764FJ3Z0CDR7D6WGHMJYWXK) |
| Satisfies | [REQ-FIXED-VOCABULARY](#01M764FJ3ZJGRCCPD3K9XH12B0) |
| Satisfies | [REQ-RELATIONS](#01M764FJ3ZZZJS6M31EWC6TJMG) |
| Mitigates | [RISK-MERGE-CONFLICTS](decisions.mara.md#01M764FJ3ZVFTZJKT1TXJQ3SM9) |

A file starts with a line `---`, then a YAML mapping, then a closing `---` line. One blank line directly after the closing line separates header and body and is not part of the body. Everything after it is the body: free-form Markdown that zban treats as opaque text. The only exceptions are task-item counting in [DES-LIFECYCLE](#01M764FJ3ZPNXY2WFB1Y624PFS) and the handoff section in [DES-HANDOFF](agents.mara.md#01M777QQGMQNB89NDDMPGRTTFA).

| Key | Required | Value |
| --- | --- | --- |
| `title` | yes | Non-empty single-line string |
| `status` | yes | A status from [REQ-FIXED-VOCABULARY](#01M764FJ3ZJGRCCPD3K9XH12B0) |
| `type` | yes | A type from [REQ-FIXED-VOCABULARY](#01M764FJ3ZJGRCCPD3K9XH12B0) |
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

The header must be plain YAML: no anchors, aliases, custom tags, merge keys or duplicate keys. Hand-written headers may use plain, single-quoted or double-quoted scalars, block or flow lists, and comments. When zban writes a header, it uses the key order above, writes a string as a plain scalar when a YAML 1.2 parser reads it back as the same string and double-quoted otherwise, writes lists in block style with one item per line indented by two spaces, and places unknown keys, preserved verbatim, after the known ones. Comments are not kept when zban rewrites a header. Empty lists and absent optional values are omitted. There is no `updated_at`; Git history records changes ([ADR-MERGE-FRIENDLY-FORMAT](decisions.mara.md#01M764FJ3Z8SW2MT7EAP58RF8R)).

When zban writes a file, it writes the closing line, one blank line and the body, or ends the file after the closing line when the body is empty. Text given as a new body, appended text and a handoff note are stored ending with exactly one LF. Appending to a non-empty body joins the text after exactly one blank line.

---

<a id="01M764FJ3ZXCS66TCPPJ3HF04V"></a>

### Readiness and ordering

| Type | Design |
| --- | --- |
| ID | DES-READINESS |
| Status | Accepted |
| Satisfies | [REQ-READY](#01M764FJ3ZGP807YGG3R6CBPNN) |

An issue is ready when all of these hold:

1. its status is `todo`;
2. its type is not `epic`;
3. every `blocked_by` target is closed;
4. no ancestor in its `parent` chain is closed;
5. no ancestor has a `blocked_by` target that is still open;
6. no ancestor has the status `draft`.

Only closed, blocked and `draft` ancestors hold an issue back ([ADR-DRAFT-HOLDS-DESCENDANTS](decisions.mara.md#01M81NANBTX4F8FH8A3NVBPCTA)). An ancestor's type, or its status `in-progress`, does not. An unknown or unreadable issue ([REQ-DAMAGED-DATA](#01M779N58NATQJ611QCSYK1PWB)) counts as open: as a blocker it holds its dependents back, and as a parent it ends the ancestor chain. Ready issues, like every listing, are ordered by priority (`critical` first), then `created_at` (oldest first), then ID. For an open issue that is not ready, zban reports every applicable reason: `status`, `epic`, `blocked_by:<id>`, `ancestor_closed:<id>`, `ancestor_blocked:<id>` and `ancestor_draft:<id>`. It reports them in this order, and the ancestors of one `ancestor_` reason nearest first. In `blocked_by:<id>` the ID is the open blocker. In the three `ancestor_` reasons it is the ancestor, one reason per affected ancestor; showing that ancestor tells what blocks it.

---

<a id="01M764FJ3ZPNXY2WFB1Y624PFS"></a>

### Status changes and archiving

| Type | Design |
| --- | --- |
| ID | DES-LIFECYCLE |
| Status | Accepted |
| Satisfies | [REQ-CLOSE-CHECKLIST](#01M764FJ3Z4DX6RS09TWKY84QC) |
| Satisfies | [REQ-AUTO-ARCHIVE](#01M764FJ3ZKD4N35SRK3JR3Y3B) |
| Satisfies | [REQ-PRESERVE-CONTENT](#01M764FJ3Z0CDR7D6WGHMJYWXK) |

Any status can change to any other status; zban enforces no workflow beyond these rules. New issues default to status `todo`, type `task` and priority `normal`.

- **Done with open tasks.** A task item is a list line starting with `- [ ]`, `* [ ]` or `+ [ ]` (unchecked) or the same with `[x]` or `[X]` (checked), at any indentation and outside fenced code blocks. Changing the status to `done` while an unchecked item exists fails unless forced.
- **Fenced code blocks.** A fence line holds a run of at least three backticks or tildes, with no backtick after a backtick run, indented by at most three columns more than its margin: the content of the list item holding the line, or the line start outside list items. A tab advances to the next multiple of four columns. A fence line opens a block, which ends at a fence line of the same character, at least as long and with nothing after it, or at the end of the body. A list item starts at a line that, within the same indentation limit, begins with `-`, `*`, `+`, or one to nine digits and `.` or `)`, followed by a space, a tab or the line end; the text after the marker is read like a line of its own, so it can start a nested item or a fence. The item's content starts after the marker and the whitespace that follows it, or one column after the marker when that whitespace is wider than four columns or ends the line. The item holds the following lines that are blank or indented at least to its content, and any other line ends it. Lines inside a block neither start nor end list items, so a closing fence has the margin of the opening one.
- **Close with open children.** Changing the status to `done` or `cancelled` while an issue whose `parent` is this one is open fails unless forced. The error names the open children.
- **Checked state.** Both rules apply to the issue as the request leaves it, so one update can check the last task item, append a summary and set `done`.
- **Creating closed.** Creating an issue as `done` while it has an unchecked task item fails the same way, and `create` cannot be forced. To record such an issue, create it open, then force setting `done` with an update. A new issue has no children, so creating it closed never fails for open children.
- **Closing.** Entering `done` or `cancelled` sets `closed_at`. Leaving them removes it. Closing never changes children or dependents ([ADR-NO-CASCADE](decisions.mara.md#01M76XFGSJ81ZR60CGYEMESP5K)).
- **Placement.** After a status change crosses between open and closed, zban first rewrites the file atomically in place (a temporary file renamed over the original), then renames it into the other directory. If the second step is interrupted, the file has the right content in the wrong place, which `zban check --fix` repairs.

---

<a id="01M779N2S6013TMF0ZXVSNVZBN"></a>

### Check findings and repair

| Type | Design |
| --- | --- |
| ID | DES-CHECK |
| Status | Accepted |
| Satisfies | [REQ-CHECK](#01M764FJ3ZBK44WBWNMZ4ZK98W) |

`zban check` reports findings ordered by path, then code. In JSON each finding is `{severity, code, path, message}`; human output prints one line per finding, or `No problems found.` when there are none, so a clean run is visibly distinct from no output. The line appears only in human output and only when there are no findings at all, so a check that finds only warnings prints the warnings alone; it changes neither the JSON output nor the exit status.

| Code | Severity | Finding |
| --- | --- | --- |
| `invalid_config` | error | `config.toml` is unparseable, has an unknown key or `format_version`, or an invalid prefix. When it is reported, check reports nothing else. |
| `invalid_filename` | error | A Markdown file whose name is not a valid ID ([DES-ISSUE-ID](#01M764FJ3ZP2NBQPN65QHVH1XY)) |
| `unparseable` | error | No frontmatter, invalid YAML, or YAML outside the plain subset of [DES-ISSUE-FILE](#01M764FJ3ZZGJNFXW9VV1NNVAD), such as Git conflict markers |
| `missing_field` | error | A required key is absent, including `closed_at` on a closed issue |
| `invalid_value` | error | A value breaks [DES-ISSUE-FILE](#01M764FJ3ZZGJNFXW9VV1NNVAD), including `closed_at` on an open issue |
| `unknown_key` | warning | A frontmatter key zban does not know |
| `duplicate_id` | error | One ID names a file in both `.zban/` and `.zban/archive/` |
| `invalid_reference` | error | A relation names an unknown issue or the issue itself. A reference to an existing but unreadable file is not reported again. |
| `cycle` | error | The issue is on a cycle through `parent` and `blocked_by`, or is blocked by one of its own ancestors ([REQ-RELATIONS](#01M764FJ3ZZZJS6M31EWC6TJMG)) |
| `related_both_sides` | warning | The same `related` link is stored on both issues |
| `open_under_closed` | warning | An open issue whose parent is closed |
| `misplaced` | error | A file in the wrong directory for its status |
| `temporary_file` | warning | A file named `.zban-*.tmp` directly inside `.zban/` or `.zban/archive/`. zban writes through such files ([REQ-PRESERVE-CONTENT](#01M764FJ3Z0CDR7D6WGHMJYWXK)), and an interrupted write leaves one behind. |

`--fix` moves each misplaced file into the directory its status requires, unless a file already exists at the target path; that finding then remains. It also deletes each leftover temporary file. The fixed findings are not reported. Because zban writes its temporary files before it takes the lock of [DES-SHARED-CORE](cli.mara.md#01M764FJ3ZNZPGXYJ4CS2QFHPK), `--fix` can delete the temporary file of a write that is still running; that write then fails with `io` and leaves its issue unchanged.

---

## Scenario

---

<a id="01M764FJ3Z89Y0KG9KKRR216PZ"></a>

### Two agents change issues on separate branches

| Type | Scenario |
| --- | --- |
| ID | SCN-PARALLEL-BRANCHES |
| Status | Accepted |

Two agents work in separate worktrees of one repository. Each creates new issues and updates an issue the other does not touch. One agent closes its issue, which moves the file into the archive. Both branches merge through ordinary pull requests without conflicts, because new issues have distinct filenames and the edited issues are different files. A conflict appears only when both branches change the same lines of the same issue, and it is resolved like any text conflict. After the merges, `zban check` reports no problems. Neither agent saw the other's status changes before the merge.
