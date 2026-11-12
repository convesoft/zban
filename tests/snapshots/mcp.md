# zban workflow

This project tracks its work in zban. Each issue is a Markdown file named after
its ID, with a short YAML header and a free-form body. Open issues live in
`.zban/` and closed ones in `.zban/archive/`. Change issues with the zban tools, which validate every change, rather than by editing the files. Where
this project's own agent instructions add to or differ from this guide, follow
them.

Each tool returns one JSON document with every field. A failure is a tool
error carrying `{"error": {"code": ..., "message": ...}}` with a stable code.

Pass `directory`, the absolute path of your working directory, on every call
when you work in a worktree or outside the directory the server started in.
Without it, a tool acts on the project found where the server started, which
may be another checkout on another branch.

## Track work in zban

- Record work in zban issues instead of ad-hoc todo lists, plan files or TODO
  comments, whenever it should outlive your current step or session.
- Give each issue one outcome. Group larger work under an epic with
  `parent`, and put steps in order with `blocked_by`.
- Write each body for a reader who has not seen this conversation: the goal,
  the context that matters, and a checklist of `- [ ]` task items.

## Before starting

1. Look for an existing issue with `list_issues {"search": TEXT}`, or with
   `show_issues {"ids": [ID]}` when the user names one.
2. If none fits, create one: `create_issue {"title": "Title", "type": "task", "body": TEXT}`.
3. Set it in progress before working on it: `update_issue {"id": ID, "status": "in-progress"}`.

## While working

- Keep the body's checklist current. Tick each item as soon as it is done with
  `update_issue {"id": ID, "replace_old": ["- [ ] Step"], "replace_new": ["- [x] Step"]}`, and
  add items as you find them.
- File newly discovered work, such as an unrelated bug, as a separate issue
  with `create_issue {"title": "Title", "type": "bug", "discovered_from": ID}` rather than
  widening the current one. Then carry on.
- A `conflict` error means the file changed after zban read it, for example
  because another agent saved it. Run `show_issues {"ids": [ID]}` again and retry.
- A warning about a skipped file, or an `unreadable` error, means an issue file
  is broken, for example by merge conflict markers. Fix it by hand, then run
  `check_issues` until it reports no errors.

## Finishing

- Append a summary and close the issue in one update:
  `update_issue {"id": ID, "append": "## Summary of Changes ...", "status": "done"}`.
  Say what changed and which checks you ran. Closing records `closed_at` and
  moves the file into `.zban/archive/`.
- zban refuses `done` while unchecked task items remain (`unchecked_items`),
  and `done` or `cancelled` while children are open (`open_children`). Finish
  the remaining items or children, close them, or move them into follow-up
  issues. Add `force` only when the user agrees.
- To abandon work, append `## Reasons for Cancelling` with the reason and set
  `update_issue {"id": ID, "status": "cancelled"}`.

## Committing

Commit changed issue files, including moves into and out of `.zban/archive/`,
together with the code they describe. zban never runs Git.

## Handoffs

A handoff is a note on an issue that lets a new session continue unfinished
work without the old conversation.

- Write one only when the user asks, while you still have the conversation:
  `update_issue {"id": ID, "handoff": TEXT}`. It replaces any earlier note and records
  the time, so overwrite the note rather than appending to it.
- Record the commit, the branch and whether the working tree is clean; what
  is done; what is next; what is blocked or uncertain; approaches tried and
  rejected; and the files touched. Use plain bullets, not task items, no
  diffs, and no `#` or `##` headings (`invalid_handoff`).
- Resume only when asked to continue: find the matching issue with
  `list_issues {"status": ["in-progress"]}`, asking the user if several could match,
  show it, and check its note against the repository before acting on it.
- On `done`, replace the handoff with the summary, keeping the rejected
  approaches:
  `update_issue {"id": ID, "no_handoff": true, "append": "## Summary of Changes ...", "status": "done"}`.

## Finding work

- `list_issues {"ready": true}` lists the issues that can start now, best first. Offer
  the top candidates to the user rather than starting one unasked.
- Show an issue before starting it, with `show_issues {"ids": [ID]}`, and read what its body
  refers to.
- For each open issue that is not ready, `list_issues` says why in
  `not_ready`.

## Reference

Types: `epic`, `feature`, `bug`, `task`. An epic groups other issues; its children carry the work.
Statuses: `draft`, `todo`, `in-progress`, `done`, `cancelled`. New issues are `todo`; `done` and `cancelled` are closed.
Priorities, highest first: `critical`, `high`, `normal`, `low`, `deferred`. The default is `normal`.

Relations are stored once, on one issue, and `show_issues` lists both directions:

- `parent`: the issue this one belongs to, usually an epic.
- `blocked_by`: issues that must be closed before this one can start.
- `related`: an informational link that does not affect readiness.
- `discovered_from`: the issue being worked on when this one was found.

zban refuses relations that could never resolve: a cycle through `parent` and
`blocked_by`, and a blocker that is an ancestor of the issue.

An issue is ready when its status is `todo`, it is not an epic, all its
blockers are closed, and no ancestor is closed or has an open blocker. Listings
are ordered by priority, then age. The reasons in `not_ready` are `status`, `epic`, `blocked_by:<id>`, `ancestor_closed:<id>`, `ancestor_blocked:<id>`.

Tools and their parameters. `?` marks an optional parameter and `[...]` a list.
IDs ignore case, and every tool takes `directory`, as described above.

- `list_issues {status?: [S], type?: [T], priority?: [P], tag?: [T], parent?: ID, ready?: true, search?: TEXT, all?: true}`:
  open issues unless `all` or `status` asks for closed ones. The values of one
  list match any of them; `parent` matches direct children only.
- `show_issues {ids: [ID]}`: full issues, with the body and the reverse relations.
- `create_issue {title: TITLE, type?: T, status?: S, priority?: P, tag?: [T], parent?: ID, blocked_by?: [ID], related?: [ID], discovered_from?: ID, body?: TEXT}`
- `update_issue {id: ID, status?: S, type?: T, priority?: P, title?: TEXT, add_tag?: [T], remove_tag?: [T], parent?: ID, no_parent?: true, add_blocked_by?: [ID], remove_blocked_by?: [ID], add_related?: [ID], remove_related?: [ID], discovered_from?: ID, no_discovered_from?: true, body?: TEXT, append?: TEXT, replace_old?: [TEXT], replace_new?: [TEXT], handoff?: TEXT, no_handoff?: true, force?: true}`:
  body edits apply in the order `body`, replacements, `append`,
  the handoff, and the whole update is applied or nothing is. Each `replace_old` text, replaced by
  the `replace_new` text at the same position, must occur
  exactly once (`text_not_found`, `text_ambiguous`).
- `delete_issue {id: ID}`: only for an issue no other issue refers to. To drop work,
  cancel the issue instead.
- `check_issues {fix?: true}`: reports problems in `.zban/` with stable codes. `fix` only moves misplaced files
  and deletes leftover temporary files.
