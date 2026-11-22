---
title: State the exact tag rule in DES-ISSUE-FILE
status: todo
type: task
priority: deferred
tags:
  - deferred
  - backlog
discovered_from: zb-esx9nv
created_at: 2026-11-22T14:13:28Z
---

`DES-ISSUE-FILE` (`docs/issues.mara.md`, the `tags` row of the header table) describes tags as "distinct lowercase words: letters, digits and single hyphens, starting with a letter". The rule zban enforces (`valid_tag` in `src/issue.rs`) is narrower: ASCII lowercase letters and digits only, a letter first, and hyphens only between words (no trailing hyphen, no `--`). "Lowercase letters" reads as admitting non-ASCII lowercase letters such as `é`, and "single hyphens" does not say a tag cannot end with one.

Since zb-esx9nv, `list --tag` and `list --not-tag` refuse values by this rule with `invalid_value`, so users meet it outside issue files too, and the item should state it exactly.

Found while working on zb-esx9nv. The gap predates that change.

## Tasks

- [ ] Reword the `tags` row of `DES-ISSUE-FILE` to state the enforced rule exactly (ASCII lowercase letters and digits, starting with a letter, hyphens only between words); also align the error message in `src/issue.rs` if it differs
- [ ] Add a test case for a non-ASCII lowercase tag and a trailing hyphen if none exists
- [ ] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation; `cargo test` passes
