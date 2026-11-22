---
title: Split tests/cli.rs and tests/mcp.rs by concern
status: todo
type: task
priority: deferred
tags:
  - deferred
  - backlog
blocked_by:
  - zb-apwgpv
created_at: 2026-11-22T14:29:23Z
---

Found by the architecture audit after milestone zb-v90b0k, at e1c9354. Structural debt: `tests/cli.rs` grew from 1,224 to 1,411 lines during the milestone and `tests/mcp.rs` from 747 to 1,040, and four issues in a row had to wait for each other because they all edited these two files.

Do this after the shared helpers exist (the blocker of this issue). Each file under `tests/` is its own test binary, so the audit recommends few files, grouped by name:

- `tests/cli.rs` (about 520 lines): the core contract, mutations, body edits and human output.
- `tests/cli_list.rs` (about 290): listing, filters and `child_counts`: `list_filters_select_issues_in_listing_order`, `list_not_tag_leaves_out_issues_carrying_an_excluded_tag`, `list_short_options_select_what_the_long_options_select`, `list_parent_fails_on_an_issue_show_cannot_find`, `child_counts_count_direct_children_only`, `an_ambiguous_child_is_left_out_of_child_counts_but_blocks_closing`.
- `tests/cli_errors.rs` (about 320): errors, exit statuses and damaged data: `failures_print_an_error_object_and_exit_with_status_1`, `invalid_usage_exits_with_status_2`, `each_error_code_is_produced`, `commands_outside_a_project_fail_with_project_not_found`, `broken_files_do_not_stop_work_on_other_issues`.
- `tests/mcp.rs` (about 520): the server contract, the parity flows and project selection.
- `tests/mcp_errors.rs` (about 280): error parity, usage and refused parameters: `invalid_arguments_give_the_same_error_codes`, `an_unknown_id_hints_at_another_branch_or_worktree_on_both_surfaces`, `unpaired_replacements_and_set_and_removed_values_are_usage_errors`, `create_and_list_take_tags_and_not_tags_and_refuse_the_singular`, `a_relative_directory_is_rejected`, `mcp_takes_no_json_option`.

While moving them, fold one duplicate: `show zb-zzzzzz`, `list --parent zb-zzzzzz` and `show nonsense` run in both `invalid_arguments_give_the_same_error_codes` and `an_unknown_id_hints_at_another_branch_or_worktree_on_both_surfaces`. Keep every `not_found` code and hint check in the second, add `list --parent nonsense` to it, and drop the four repeated rows from the first.

## Tasks

- [ ] Split `tests/cli.rs` into `cli.rs`, `cli_list.rs` and `cli_errors.rs`
- [ ] Split `tests/mcp.rs` into `mcp.rs` and `mcp_errors.rs`, folding the repeated `not_found` rows
- [ ] Keep every `// @mara implements` marker on its test

## Done when

- [ ] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [ ] `zban check` reports no errors
