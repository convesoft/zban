---
title: Leave the docs layout rules to the mara-docs-layout skill
status: done
type: task
created_at: 2026-11-13T15:28:34Z
closed_at: 2026-11-13T15:28:40Z
---

The owner's personal `mara-docs-layout` agent skill now holds the documentation layout rules for Mara projects: page shape, item homes, when to split pages and add folders, and per-package docs roots. `AGENTS.md` repeats part of them in its `## Documentation layout` section, which would drift from the skill.

zban follows the skill's rules without exceptions: one `docs/` root at stage 1, with decisions and verification as single pages. The skill reads that from the tree, so `AGENTS.md` needs no replacement text.

- [x] Remove the `## Documentation layout` section from `AGENTS.md`
- [x] `zban check` reports no problems

## Summary of Changes

Removed the `## Documentation layout` section from `AGENTS.md`; the `mara-docs-layout` skill now carries those rules, and zban has no exceptions to record. Docs-only change: ran `zban check` (no problems). No code changed, so `cargo fmt`, `clippy` and `test` were not run; CI on the pull request covers them.
