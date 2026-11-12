---
title: Count code fences only where Markdown does
status: done
type: bug
priority: high
parent: zb-9jra34
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
closed_at: 2026-11-12T13:26:20Z
---

Found by the v0.1.0 review: a fence counts at any indentation, so an indented "```" in prose hides every later task item from the close check (`DES-LIFECYCLE`) and every later heading from handoff detection (`DES-HANDOFF`).

Maintainer decision: a fence opens only when indented by at most three spaces, or inside a list item, as in Markdown. Record this in `DES-LIFECYCLE` and `DES-HANDOFF` first.

- [x] Mara updated
- [x] The shared fence scanning follows the rule, with tests for both task counting and handoff detection

## Summary of Changes

- `DES-LIFECYCLE` now defines fenced code blocks precisely: a fence line may be indented at most three columns past its margin (the content of the list item holding it, or the line start), with tab stops every four columns. It also defines list items: their markers, their content column, and the lines they hold. `DES-HANDOFF` cites that definition for the headings it ignores. `VER-LIFECYCLE` and `VER-HANDOFF` name the new cases.
- `src/markdown.rs`: the shared `Fences` scanner keeps a stack of list item content columns. Fences and list items open only within three columns of the current margin, and a closing fence keeps the margin of its opening fence. There is no new dependency.
- Known simplification, recorded in Mara: a lazy continuation line ends a list item, and lines inside a block neither start nor end list items, so a block opened in a list item runs to its closing fence or the end of the body.
- Tests: unit tests for indentation, nested and ordered items, a marker followed by a fence, and closing margins. `tests/lifecycle.rs` checks that task items after a four-column fence line count while nested list-item blocks still hide theirs (`VER-LIFECYCLE`). `tests/handoff.rs` checks through the binary that a handoff after a four-column fence line is found and replaced in place (`VER-HANDOFF`). Both integration tests fail against the old scanner.
- Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all pass), `mara schema validate` and `mara project validate` (`valid: true`, evaluation complete, no diagnostics).
