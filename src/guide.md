# zban workflow

This project tracks its work in zban issues: Markdown files in `.zban/`, moved
to `.zban/archive/` when closed. Change them only with
{{cli}}`zban` commands{{end}}{{mcp}}the zban tools{{end}}. The project's own agent instructions take precedence.

{{cli}}
Commands print text for people. Add `--json` to get one JSON document with
every field; a failure then prints `{"error": {"code": ..., "message": ...}}`
with a stable code. Options that take text read standard input when given `-`,
which suits multi-line bodies: `{{create title="Title" body=TEXT}} <<'EOF'`.
{{end}}
{{mcp}}
In a worktree, or outside the directory the server started in, pass
`directory`, your working directory's absolute path, on every call, or tools
may act on another checkout.
{{end}}

## Track work in zban

- Record work that outlives your current step in issues, not todo lists, plan
  files or TODO comments.
- One outcome per issue. Group larger work under an epic with
  `{{param parent}}`, and order steps with `{{param blocked_by}}`.
- Write bodies for a reader without this conversation: the goal, the context
  and `- [ ]` task items.

## Before starting

1. Find an existing issue: `{{list search=TEXT}}`, or
   `{{show ids=ID}}` if the user names one.
2. Otherwise create one: `{{create title="Title" type=task body=TEXT}}`.
3. Set it in progress: `{{update id=ID status=in-progress}}`.

## While working

- Tick each task item once done, and add items as you find them:
  `{{update id=ID replace_old="- [ ] Step" replace_new="- [x] Step"}}`.
- File unrelated work you discover as a separate issue, then carry on:
  `{{create title="Title" type=bug discovered_from=ID}}`.
- On a `conflict` error, show the issue again and retry.
- Fix an issue file that zban skips or reports `unreadable`, such as one with
  merge conflict markers, by hand until `{{check}}` reports no errors.

## Finishing

- Close with a summary of what changed and the checks you ran:
  `{{update id=ID append="## Summary of Changes ..." status=done}}`.
- Unchecked task items block `done` (`unchecked_items`), and open children
  block `done` and `cancelled` (`open_children`): finish, close or move them
  to follow-up issues. Use `{{param force}}` only if the user agrees.
- To abandon work, append `## Reasons for Cancelling` and set `cancelled`.

## Committing

Commit issue files, archive moves included, with the code they describe.

## Handoffs

A handoff note lets a new session continue unfinished work.

- Write one only when the user asks, while you have the conversation:
  `{{update id=ID handoff=TEXT}}`. It replaces any earlier note.
- Record the commit, branch and whether the tree is clean; what is done, next,
  and blocked or uncertain; rejected approaches; and files touched. Use plain
  bullets: no task items, diffs, or `#` or `##` headings.
- Resume only when asked: find the issue with `{{list status=in-progress}}`,
  asking the user if several match, and check its note against the repository
  first.
- When closing, add `{{param no_handoff}}` and keep the rejected approaches in
  the summary.

## Finding work

- `{{list ready}}` lists issues that can start now, best first. Offer the top
  ones to the user rather than starting one unasked.
- Show an issue before starting it, and read what its body refers to.

## Reference

Types: {{types}}. An epic groups other issues.
Statuses: {{statuses}}. New issues are `todo`; `done` and `cancelled` are closed.
Priorities, highest first: {{priorities}}.

Relations: `parent`, `blocked_by` (issues to close first), `related` and
`discovered_from` (the issue in progress when this one was found). `{{show}}`
shows both directions.

An issue is ready when it is `todo`, not an epic, all its blockers are closed,
and no ancestor is closed or has an open blocker.
The reasons in `not_ready` are {{not_ready}}.

{{cli}}
Commands and their options. `...` marks a repeatable option, IDs ignore case,
and every command takes `-C DIR` to act as if started in `DIR`.

- `{{synopsis list}}`:
  open issues unless `{{param all}}` or `{{param status}}` asks for closed ones.
  Repeated values of one option match any of them; `{{param parent}}` matches
  direct children only.
- `{{synopsis show}}`: full issues, with the body and the reverse relations.
- `{{synopsis create}}`
- `{{synopsis update}}`:
  body edits apply in the order `{{param body}}`, replacements,
  `{{param append}}`, the handoff, and the whole update is applied or nothing
  is. Each `{{param replace_old}}` text must occur exactly once
  (`text_not_found`, `text_ambiguous`).
- `{{synopsis delete}}`: only for an issue no other issue refers to. To drop
  work, cancel the issue instead.
- `{{synopsis check}}`: reports problems in `.zban/` with stable codes and
  exits with status 1 when any is an error. `{{param fix}}` only moves
  misplaced files and deletes leftover temporary files.
{{end}}
