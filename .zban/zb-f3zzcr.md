---
title: Stop the guide snapshot tests racing in snapshot-update mode
status: todo
type: task
priority: deferred
tags:
  - deferred
  - backlog
created_at: 2026-11-22T14:29:23Z
---

Found by the architecture audit after milestone zb-v90b0k, at e1c9354. A worker hit it once during the milestone.

With `ZBAN_UPDATE_SNAPSHOTS=1 cargo test`, `prime_prints_the_guide_inside_a_project` rewrites `tests/snapshots/prime.md` with a plain `fs::write` (it repeats `assert_snapshot` inline, `tests/prime.rs` about lines 238–257), and `mcp_server_sends_the_guide_inside_a_project` rewrites `tests/snapshots/mcp.md`. In the same run, `this_repository_loads_the_guide` (`tests/prime.rs`, about line 654) and `prime_and_the_mcp_instructions_carry_no_handoff_notes` (`tests/handoff.rs`, about lines 384–401) read those files from disk, so they can see a stale or half-written snapshot. It only happens in snapshot-update mode.

Suggested fix from the audit: let only one test touch each snapshot file. `prime_prints_the_guide_inside_a_project` calls `assert_snapshot`; the two readers compare against the rendered guide (`zban::guide::render`) or against the output before and after, not against the files.

## Tasks

- [ ] Make each snapshot file read and written by one test only
- [ ] Check that `ZBAN_UPDATE_SNAPSHOTS=1 cargo test` passes when the guide text changes

## Done when

- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [ ] `zban check` reports no errors
