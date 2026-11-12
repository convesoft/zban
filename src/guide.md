# zban workflow

This project tracks its work in zban. Each issue is a Markdown file named after
its ID, with a short YAML header and a free-form body. Open issues live in
`.zban/` and closed ones in `.zban/archive/`. Change issues with `zban`
commands, which validate every change, rather than by editing the files. Where
this project's own agent instructions add to or differ from this guide, follow
them.

Commands print text for people. Add `--json` to get one JSON document with
every field; a failure then prints `{"error": {"code": ..., "message": ...}}`
with a stable code. Options that take text read standard input when given `-`,
which suits multi-line bodies: `zban create "Title" --body - <<'EOF'`.

## Track work in zban

- Record work in zban issues instead of ad-hoc todo lists, plan files or TODO
  comments, whenever it should outlive your current step or session.
- Give each issue one outcome. Group larger work under an epic with
  `--parent`, and put steps in order with `--blocked-by`.
- Write each body for a reader who has not seen this conversation: the goal,
  the context that matters, and a checklist of `- [ ]` task items.

## Before starting

1. Look for an existing issue with `zban list --search TEXT`, or with
   `zban show ID` when the user names one.
2. If none fits, create one: `zban create "Title" --type task --body -`.
3. Set it in progress before working on it: `zban update ID --status in-progress`.

## While working

- Keep the body's checklist current. Tick each item as soon as it is done with
  `zban update ID --replace-old "- [ ] Step" --replace-new "- [x] Step"`, and
  add items as you find them.
- File newly discovered work, such as an unrelated bug, as a separate issue
  with `zban create "Title" --type bug --discovered-from ID` rather than
  widening the current one. Then carry on.
- A `conflict` error means the file changed after zban read it, for example
  because another agent saved it. Run `zban show ID` again and retry.
- A warning about a skipped file, or an `unreadable` error, means an issue file
  is broken, for example by merge conflict markers. Fix it by hand, then run
  `zban check` until it reports no errors.

## Finishing

- Append a summary and close the issue in one update:
  `zban update ID --append "## Summary of Changes ..." --status done`.
  Say what changed and which checks you ran. Closing records `closed_at` and
  moves the file into `.zban/archive/`.
- zban refuses `done` while unchecked task items remain (`unchecked_items`),
  and `done` or `cancelled` while children are open (`open_children`). Finish
  the remaining items or children, close them, or move them into follow-up
  issues. Add `--force` only when the user agrees.
- To abandon work, append `## Reasons for Cancelling` with the reason and set
  `zban update ID --status cancelled`.

## Committing

Commit changed issue files, including moves into and out of `.zban/archive/`,
together with the code they describe. zban never runs Git.

## Finding work

- `zban list --ready` lists the issues that can start now, best first. Offer
  the top candidates to the user rather than starting one unasked.
- Show an issue before starting it, with `zban show ID`, and read what its body
  refers to.
- For each open issue that is not ready, `zban list --json` says why in
  `not_ready`.

## Reference

Types: {{types}}. An epic groups other issues; its children carry the work.
Statuses: {{statuses}}. New issues are `todo`; `done` and `cancelled` are closed.
Priorities, highest first: {{priorities}}. The default is `normal`.

Relations are stored once, on one issue, and `zban show` lists both directions:

- `parent`: the issue this one belongs to, usually an epic.
- `blocked_by`: issues that must be closed before this one can start.
- `related`: an informational link that does not affect readiness.
- `discovered_from`: the issue being worked on when this one was found.

zban refuses relations that could never resolve: a cycle through `parent` and
`blocked_by`, and a blocker that is an ancestor of the issue.

An issue is ready when its status is `todo`, it is not an epic, all its
blockers are closed, and no ancestor is closed or has an open blocker. Listings
are ordered by priority, then age. The reasons in `not_ready` are {{not_ready}}.

Commands and their main options. `...` marks a repeatable option, IDs ignore
case, and every command takes `-C DIR` to act as if started in `DIR`.

- `zban list [--status S]... [--type T]... [--priority P]... [--tag T]... [--parent ID] [--ready] [--search TEXT] [--all]`:
  open issues unless `--all` or `--status` asks for closed ones. Repeated values
  of one option match any of them; `--parent` matches direct children only.
- `zban show ID...`: full issues, with the body and the reverse relations.
- `zban create TITLE [--type T] [--status S] [--priority P] [--tag T]... [--parent ID] [--blocked-by ID]... [--related ID]... [--discovered-from ID] [--body TEXT|-]`
- `zban update ID [--status S] [--type T] [--priority P] [--title TEXT] [--add-tag T]... [--remove-tag T]... [--parent ID|--no-parent] [--add-blocked-by ID]... [--remove-blocked-by ID]... [--add-related ID]... [--remove-related ID]... [--discovered-from ID|--no-discovered-from] [--body TEXT|-] [--append TEXT|-] [--replace-old TEXT --replace-new TEXT]... [--force]`:
  body edits apply in the order `--body`, replacements, `--append`, and the
  whole update is applied or nothing is. Each `--replace-old` text must occur
  exactly once (`text_not_found`, `text_ambiguous`).
- `zban delete ID`: only for an issue no other issue refers to. To drop work,
  cancel the issue instead.
- `zban check [--fix]`: reports problems in `.zban/` with stable codes and
  exits with status 1 when any is an error. `--fix` only moves misplaced files
  and deletes leftover temporary files.
