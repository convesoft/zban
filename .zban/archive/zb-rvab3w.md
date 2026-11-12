---
title: Fix the temporary-directory race in the guide tests
status: done
type: bug
priority: high
parent: zb-9jra34
created_at: 2026-11-12T11:45:46Z
closed_at: 2026-11-12T11:48:14Z
---

The `VER-AGENT-GUIDE` tests in `tests/prime.rs` fail intermittently. CI on an unrelated pull request showed:

```
error: .: No such file or directory (os error 2) [io]
test guide_lists_every_tool_with_all_its_parameters ... FAILED
thread 'guide_lists_every_tool_with_all_its_parameters' panicked at tests/common/mcp.rs:85:13:
the server closed its output during initialize
```

## Cause

`tool_schemas()` starts the MCP server in `project("mcp-tools")`, which calls `temp_dir("prime-mcp-tools")` from `tests/common/mod.rs`. `temp_dir` deletes and recreates a fixed directory under `CARGO_TARGET_TMPDIR`. Two tests call `tool_schemas()`: `guide_names_only_tools_and_parameters_that_exist` and `guide_lists_every_tool_with_all_its_parameters`. They run in parallel, so one can delete the directory while the other's `zban mcp` is starting in it. The server then cannot resolve `.` and exits before it answers `initialize`.

A survey of every `temp_dir` and `temp_dir_outside_projects` call during `cargo test` found no other name that more than one test uses, in the same test binary or across binaries. The fixed names still leave room for the same race whenever a new test reuses a name.

## Tasks

- [x] Give every `temp_dir` call its own directory, in the tests only and without serializing the suite
- [x] Run `cargo test` several times and `cargo test --test prime` 20 times without a failure
- [x] Run `cargo fmt --all -- --check` and `cargo clippy --all-targets -- -D warnings`

Done when the checks above pass.


## Summary of Changes

`temp_dir` and `temp_dir_outside_projects` in `tests/common/mod.rs` now add a number to the name that differs on each call with the same name, so no two calls share a directory. Names stay stable across runs, so `target/tmp` holds the same set of directories each time. Only test code changed; the suite still runs in parallel.

Checks run:

- Reproduced the failure before the fix: `cargo test --test prime -- guide_` failed 1 of 100 times, with `.: No such file or directory`, and the `prime` test binary run directly failed 8 of 1000 times.
- After the fix: `cargo test` 10 times, `cargo test --test prime` 20 times, `cargo test --test prime -- guide_` 100 times and the `prime` test binary 1000 times, all without a failure.
- `cargo fmt --all -- --check` and `cargo clippy --locked --all-targets -- -D warnings` pass, and `cargo test --locked` passes 151 tests.
