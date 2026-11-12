# zban workflow

This project tracks its work in zban. Each issue is a Markdown file named after
its ID, with a short YAML header and a free-form body. Open issues live in
`.zban/` and closed ones in `.zban/archive/`. Change issues with {{cli}}`zban`
commands{{end}}{{mcp}}the zban tools{{end}}, which validate every change, rather than by editing the files. Where
this project's own agent instructions add to or differ from this guide, follow
them.

{{cli}}
Commands print text for people. Add `--json` to get one JSON document with
every field; a failure then prints `{"error": {"code": ..., "message": ...}}`
with a stable code. Options that take text read standard input when given `-`,
which suits multi-line bodies: `{{create title="Title" body=TEXT}} <<'EOF'`.
{{end}}
{{mcp}}
Each tool returns one JSON document with every field. A failure is a tool
error carrying `{"error": {"code": ..., "message": ...}}` with a stable code.

Pass `directory`, the absolute path of your working directory, on every call
when you work in a worktree or outside the directory the server started in.
Without it, a tool acts on the project found where the server started, which
may be another checkout on another branch.
{{end}}

## Track work in zban

- Record work in zban issues instead of ad-hoc todo lists, plan files or TODO
  comments, whenever it should outlive your current step or session.
- Give each issue one outcome. Group larger work under an epic with
  `{{param parent}}`, and put steps in order with `{{param blocked_by}}`.
- Write each body for a reader who has not seen this conversation: the goal,
  the context that matters, and a checklist of `- [ ]` task items.

## Before starting

1. Look for an existing issue with `{{list search=TEXT}}`, or with
   `{{show ids=ID}}` when the user names one.
2. If none fits, create one: `{{create title="Title" type=task body=TEXT}}`.
3. Set it in progress before working on it: `{{update id=ID status=in-progress}}`.

## While working

- Keep the body's checklist current. Tick each item as soon as it is done with
  `{{update id=ID replace_old="- [ ] Step" replace_new="- [x] Step"}}`, and
  add items as you find them.
- File newly discovered work, such as an unrelated bug, as a separate issue
  with `{{create title="Title" type=bug discovered_from=ID}}` rather than
  widening the current one. Then carry on.
- A `conflict` error means the file changed after zban read it, for example
  because another agent saved it. Run `{{show ids=ID}}` again and retry.
- A warning about a skipped file, or an `unreadable` error, means an issue file
  is broken, for example by merge conflict markers. Fix it by hand, then run
  `{{check}}` until it reports no errors.

## Finishing

- Append a summary and close the issue in one update:
  `{{update id=ID append="## Summary of Changes ..." status=done}}`.
  Say what changed and which checks you ran. Closing records `closed_at` and
  moves the file into `.zban/archive/`.
- zban refuses `done` while unchecked task items remain (`unchecked_items`),
  and `done` or `cancelled` while children are open (`open_children`). Finish
  the remaining items or children, close them, or move them into follow-up
  issues. Add `{{param force}}` only when the user agrees.
- To abandon work, append `## Reasons for Cancelling` with the reason and set
  `{{update id=ID status=cancelled}}`.

## Committing

Commit changed issue files, including moves into and out of `.zban/archive/`,
together with the code they describe. zban never runs Git.

## Handoffs

A handoff is a note on an issue that lets a new session continue unfinished
work without the old conversation.

- Write one only when the user asks, while you still have the conversation:
  `{{update id=ID handoff=TEXT}}`. It replaces any earlier note and records
  the time, so overwrite the note rather than appending to it.
- Record the commit, the branch and whether the working tree is clean; what
  is done; what is next; what is blocked or uncertain; approaches tried and
  rejected; and the files touched. Use plain bullets, not task items, no
  diffs, and no `#` or `##` headings (`invalid_handoff`).
- Resume only when asked to continue: find the matching issue with
  `{{list status=in-progress}}`, asking the user if several could match,
  show it, and check its note against the repository before acting on it.
- On `done`, replace the handoff with the summary, keeping the rejected
  approaches:
  `{{update id=ID no_handoff append="## Summary of Changes ..." status=done}}`.

## Finding work

- `{{list ready}}` lists the issues that can start now, best first. Offer
  the top candidates to the user rather than starting one unasked.
- Show an issue before starting it, with `{{show ids=ID}}`, and read what its body
  refers to.
- For each open issue that is not ready, `{{list}}{{cli}} --json{{end}}` says why in
  `not_ready`.

## Reference

Types: {{types}}. An epic groups other issues; its children carry the work.
Statuses: {{statuses}}. New issues are `todo`; `done` and `cancelled` are closed.
Priorities, highest first: {{priorities}}. The default is `normal`.

Relations are stored once, on one issue, and `{{show}}` lists both directions:

- `parent`: the issue this one belongs to, usually an epic.
- `blocked_by`: issues that must be closed before this one can start.
- `related`: an informational link that does not affect readiness.
- `discovered_from`: the issue being worked on when this one was found.

zban refuses relations that could never resolve: a cycle through `parent` and
`blocked_by`, and a blocker that is an ancestor of the issue.

An issue is ready when its status is `todo`, it is not an epic, all its
blockers are closed, and no ancestor is closed or has an open blocker. Listings
are ordered by priority, then age. The reasons in `not_ready` are {{not_ready}}.

{{cli}}
Commands and their main options. `...` marks a repeatable option, IDs ignore
case, and every command takes `-C DIR` to act as if started in `DIR`.
{{end}}
{{mcp}}
Tools and their parameters. `?` marks an optional parameter and `[...]` a list.
IDs ignore case, and every tool takes `directory`, as described above.
{{end}}

- `{{synopsis list}}`:
  open issues unless `{{param all}}` or `{{param status}}` asks for closed ones. {{cli}}Repeated values
  of one option{{end}}{{mcp}}The values of one
  list{{end}} match any of them; `{{param parent}}` matches direct children only.
- `{{synopsis show}}`: full issues, with the body and the reverse relations.
- `{{synopsis create}}`
- `{{synopsis update}}`:
  body edits apply in the order `{{param body}}`, replacements, `{{param append}}`,
  the handoff, and the whole update is applied or nothing is. Each `{{param replace_old}}` text{{mcp}}, replaced by
  the `{{param replace_new}}` text at the same position,{{end}} must occur
  exactly once (`text_not_found`, `text_ambiguous`).
- `{{synopsis delete}}`: only for an issue no other issue refers to. To drop work,
  cancel the issue instead.
- `{{synopsis check}}`: reports problems in `.zban/` with stable codes{{cli}} and
  exits with status 1 when any is an error{{end}}. `{{param fix}}` only moves misplaced files
  and deletes leftover temporary files.
