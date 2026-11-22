---
title: Remove repeated whole-project work from show, list and Graph::new
status: todo
type: task
priority: deferred
tags:
  - deferred
  - backlog
created_at: 2026-11-22T14:29:23Z
---

Found by the architecture audit after milestone zb-v90b0k, at e1c9354. Structural debt: nothing is wrong today, the work is repeated.

- `Graph::reverse` (`src/relations.rs`, about lines 248–266) calls `self.issues()`, which copies and sorts every readable issue, to find the few that refer to one ID. `show` calls it once per requested ID (`src/operations.rs`, about lines 308–315 and 463–477). Iterating `self.readable()` and sorting only the four small result lists gives the same order.
- `Graph::new` (`src/relations.rs`, about lines 115–128) collects `graph.readable()` into a temporary vector to build `child_counts`; building the map from `entries.values()` before constructing the graph avoids it.
- `Project::findings` (`src/check.rs`, about lines 89–119) builds its own `by_id` map to find duplicate IDs and then builds a `Graph`, which already marks them as ambiguous, as `operations::warnings` uses (`src/operations.rs`, about lines 505–513).
- `list` (`src/operations.rs`, about lines 263–290) lowercases every title and body for `--search` before the cheap filters run, and builds the full view, with its two Markdown scans, of every open issue before `--ready` drops the ones that are not ready.

None of this changes output. Relevant items: `DES-SHARED-CORE`, `GOAL-SMALL-TOOL`.

## Tasks

- [ ] Make `Graph::reverse` sort only its results
- [ ] Build `child_counts` without the temporary vector, and drop `by_id` from `Project::findings`
- [ ] Run the cheap filters of `list` before the search, and check readiness before building the view

## Done when

- [ ] Output of `list`, `show` and `check` is unchanged (the existing tests pass without edits to their expectations)
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [ ] `zban check` reports no errors
