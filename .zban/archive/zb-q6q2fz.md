---
title: Should list --tag refuse invalid tag values, as --not-tag does?
status: done
type: task
tags:
  - maintainer
  - question
related:
  - zb-esx9nv
created_at: 2026-11-22T14:04:27Z
closed_at: 2026-11-22T14:19:47Z
---

A question for the maintainer from the orchestrator of milestone zb-v90b0k. Nothing waits on the answer: the recommendation below stands until it is answered, and it can be undone.

## Context

zb-esx9nv adds `zban list --not-tag T` and `not_tags` on the MCP tool `list_issues`. The issue asked for tag validation on the new filter. The worker also made the existing filter validate: a value that is not a valid tag (for example `"Bad Tag"`) now fails with `invalid_value` in `--tag`/`tags` as well as in `--not-tag`/`not_tags`, as an unknown status, type or priority already does. Before, `zban list --tag "Bad Tag"` returned an empty list with exit status 0. The worker wrote this into `REQ-LIST-FILTERS` ("A status, type, priority or tag value that no issue could carry is refused rather than matching nothing") and `DES-CLI`. It changes what an existing option does, which was not part of what the maintainer approved for zb-esx9nv.

## Options

1. Recommended, and what pull request #57 does: refuse an invalid tag value in both filters. No readable issue can carry such a tag, so the filter would quietly select or exclude nothing, and the four filters then behave alike. Cost: a script that passes an invalid tag to `--tag` now gets exit status 1 instead of an empty list.
2. Validate only `--not-tag`/`not_tags` and leave `--tag`/`tags` as in 0.1.0. Cost: the two tag filters treat the same bad value differently.
3. Validate neither. Cost: a mistyped tag in `--not-tag` silently excludes nothing, which hides exactly the work the filter was meant to hide.

## How to undo option 1

Drop the `check_tag` call for `tags` in `list` (`src/operations.rs`), restore the sentence in `REQ-LIST-FILTERS` and `DES-CLI`, and remove the `--tag` case from the tests of zb-esx9nv.

## Done when

- [x] The maintainer has chosen an option, the answer is recorded here and in zb-r2nnaz, and any change it needs is filed

## Summary of Changes

The maintainer answered on 2026-11-22: option 1. An invalid tag value is refused with `invalid_value` in both `--tag`/`tags` and `--not-tag`/`not_tags`. That is what pull request #57 (zb-esx9nv) already merged, so no change is needed. The answer is recorded in zb-r2nnaz, under the decisions of zb-esx9nv.

Checks run: `zban check` (no problems). No code or Mara item changed.
