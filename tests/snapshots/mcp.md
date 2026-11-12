# zban workflow

This project tracks its work in zban issues: Markdown files in `.zban/`, moved
to `.zban/archive/` when closed. Change them only with
the zban tools. The project's own agent instructions take precedence.

In a worktree, or outside the directory the server started in, pass
`directory`, your working directory's absolute path, on every call, or tools
may act on another checkout.

## Track work in zban

- Record work that outlives your current step in issues, not todo lists, plan
  files or TODO comments.
- One outcome per issue. Group larger work under an epic with
  `parent`, and order steps with `blocked_by`.
- Write bodies for a reader without this conversation: the goal, the context
  and `- [ ]` task items.

## Before starting

1. Find an existing issue: `list_issues {"search": TEXT}`, or
   `show_issues {"ids": [ID]}` if the user names one.
2. Otherwise create one: `create_issue {"title": "Title", "type": "task", "body": TEXT}`.
3. Set it in progress: `update_issue {"id": ID, "status": "in-progress"}`.

## While working

- Tick each task item once done, and add items as you find them:
  `update_issue {"id": ID, "replace_old": ["- [ ] Step"], "replace_new": ["- [x] Step"]}`.
- File unrelated work you discover as a separate issue, then carry on:
  `create_issue {"title": "Title", "type": "bug", "discovered_from": ID}`.
- On a `conflict` error, show the issue again and retry.
- Fix an issue file that zban skips or reports `unreadable`, such as one with
  merge conflict markers, by hand until `check_issues` reports no errors.

## Finishing

- Close with a summary of what changed and the checks you ran:
  `update_issue {"id": ID, "append": "## Summary of Changes ...", "status": "done"}`.
- Unchecked task items block `done` (`unchecked_items`), and open children
  block `done` and `cancelled` (`open_children`): finish, close or move them
  to follow-up issues. Use `force` only if the user agrees.
- To abandon work, append `## Reasons for Cancelling` and set `cancelled`.

## Committing

Commit issue files, archive moves included, with the code they describe.

## Handoffs

A handoff note lets a new session continue unfinished work.

- Write one only when the user asks, while you have the conversation:
  `update_issue {"id": ID, "handoff": TEXT}`. It replaces any earlier note.
- Record the commit, branch and whether the tree is clean; what is done, next,
  and blocked or uncertain; rejected approaches; and files touched. Use plain
  bullets: no task items, diffs, or `#` or `##` headings.
- Resume only when asked: find the issue with `list_issues {"status": ["in-progress"]}`,
  asking the user if several match, and check its note against the repository
  first.
- When closing, add `no_handoff` and keep the rejected approaches in
  the summary.

## Finding work

- `list_issues {"ready": true}` lists issues that can start now, best first. Offer the top
  ones to the user rather than starting one unasked.
- Show an issue before starting it, and read what its body refers to.

## Reference

Types: `epic`, `feature`, `bug`, `task`. An epic groups other issues.
Statuses: `draft`, `todo`, `in-progress`, `done`, `cancelled`. New issues are `todo`; `done` and `cancelled` are closed.
Priorities, highest first: `critical`, `high`, `normal`, `low`, `deferred`.

Relations: `parent`, `blocked_by` (issues to close first), `related` and
`discovered_from` (the issue in progress when this one was found). `show_issues`
shows both directions.

An issue is ready when it is `todo`, not an epic, all its blockers are closed,
and no ancestor is closed or has an open blocker.
The reasons in `not_ready` are `status`, `epic`, `blocked_by:<id>`, `ancestor_closed:<id>`, `ancestor_blocked:<id>`.

