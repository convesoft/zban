# zban workflow

This project tracks its work in zban issues: Markdown files in `.zban/`, moved
to `.zban/archive/` when closed. Change them only with
`zban` commands. The project's own agent instructions take precedence.

Commands print text for people. Add `--json` to get one JSON document with
every field; a failure then prints `{"error": {"code": ..., "message": ...}}`
with a stable code. Options that take text read standard input when given `-`,
which suits multi-line bodies: `zban create "Title" --body - <<'EOF'`.

## Track work in zban

- Record work that outlives your current step in issues, not todo lists, plan
  files or TODO comments.
- One outcome per issue. Group larger work under an epic with
  `--parent`, and order steps with `--blocked-by`.
- Write bodies for a reader without this conversation: the goal, the context
  and `- [ ]` task items.

## Before starting

1. Find an existing issue: `zban list --search TEXT`, or
   `zban show ID` if the user names one.
2. Otherwise create one: `zban create "Title" --type task --body -`.
3. Set it in progress: `zban update ID --status in-progress`.

## While working

- Tick each task item once done, and add items as you find them:
  `zban update ID --replace-old "- [ ] Step" --replace-new "- [x] Step"`.
- File unrelated work you discover as a separate issue, then carry on:
  `zban create "Title" --type bug --discovered-from ID`.
- On a `conflict` error, show the issue again and retry.
- Fix an issue file that zban skips or reports `unreadable`, such as one with
  merge conflict markers, by hand until `zban check` reports no errors.

## Finishing

- Close with a summary of what changed and the checks you ran:
  `zban update ID --append "## Summary of Changes ..." --status done`.
- Unchecked task items block `done` (`unchecked_items`), and open children
  block `done` and `cancelled` (`open_children`): finish, close or move them
  to follow-up issues. Use `--force` only if the user agrees.
- To abandon work, append `## Reasons for Cancelling` and set `cancelled`.

## Committing

Commit issue files, archive moves included, with the code they describe.

## Handoffs

A handoff note lets a new session continue unfinished work.

- Write one only when the user asks, while you have the conversation:
  `zban update ID --handoff -`. It replaces any earlier note.
- Record the commit, branch and whether the tree is clean; what is done, next,
  and blocked or uncertain; rejected approaches; and files touched. Use plain
  bullets: no task items, diffs, or `#` or `##` headings.
- Resume only when asked: find the issue with `zban list --status in-progress`,
  asking the user if several match, and check its note against the repository
  first.
- When closing, add `--no-handoff` and keep the rejected approaches in
  the summary.

## Finding work

- `zban list --ready` lists issues that can start now, best first. Offer the top
  ones to the user rather than starting one unasked.
- Show an issue before starting it, and read what its body refers to.

## Reference

Types: `epic`, `feature`, `bug`, `task`. An epic groups other issues.
Statuses: `draft`, `todo`, `in-progress`, `done`, `cancelled`. New issues are `todo`; `done` and `cancelled` are closed.
Priorities, highest first: `critical`, `high`, `normal`, `low`, `deferred`.

Relations: `parent`, `blocked_by` (issues to close first), `related` and
`discovered_from` (the issue in progress when this one was found). `zban show`
shows both directions.

An issue is ready when it is `todo`, not an epic, all its blockers are closed,
and no ancestor is closed or has an open blocker.
The reasons in `not_ready` are `status`, `epic`, `blocked_by:<id>`, `ancestor_closed:<id>`, `ancestor_blocked:<id>`.

Commands and their options. `...` marks a repeatable option, IDs ignore case,
and every command takes `-C DIR` to act as if started in `DIR`.

- `zban list [--status S]... [--type T]... [--priority P]... [--tag T]... [--parent ID] [--ready] [--search TEXT] [--all]`:
  open issues unless `--all` or `--status` asks for closed ones.
  Repeated values of one option match any of them; `--parent` matches
  direct children only.
- `zban show ID...`: full issues, with the body and the reverse relations.
- `zban create TITLE [--type T] [--status S] [--priority P] [--tag T]... [--parent ID] [--blocked-by ID]... [--related ID]... [--discovered-from ID] [--body TEXT|-]`
- `zban update ID [--status S] [--type T] [--priority P] [--title TEXT] [--add-tag T]... [--remove-tag T]... [--parent ID | --no-parent] [--add-blocked-by ID]... [--remove-blocked-by ID]... [--add-related ID]... [--remove-related ID]... [--discovered-from ID | --no-discovered-from] [--body TEXT|-] [--append TEXT|-] [--replace-old TEXT --replace-new TEXT]... [--handoff TEXT|- | --no-handoff] [--force]`:
  body edits apply in the order `--body`, replacements,
  `--append`, the handoff, and the whole update is applied or nothing
  is. Each `--replace-old` text must occur exactly once
  (`text_not_found`, `text_ambiguous`).
- `zban delete ID`: only for an issue no other issue refers to. To drop
  work, cancel the issue instead.
- `zban check [--fix]`: reports problems in `.zban/` with stable codes and
  exits with status 1 when any is an error. `--fix` only moves
  misplaced files and deletes leftover temporary files.
