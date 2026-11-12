---
title: MCP server
status: done
type: feature
parent: zb-9jra34
blocked_by:
  - zb-gnt4g4
  - zb-y67s20
created_at: 2026-11-11T21:00:08Z
closed_at: 2026-11-12T10:33:00Z
---

Implements `DES-MCP-TOOLS` and the `mcp` command.

## Done when

- [x] `VER-SURFACE-PARITY` passes
- [x] `VER-AGENT-GUIDE` passes for the MCP server instructions

## Summary of Changes

- `zban mcp` (`src/mcp.rs`) is a thin adapter over `operations`, like the CLI. It implements rmcp's `ServerHandler` by hand, without the `macros` feature, on a current-thread `tokio` runtime that only `zban mcp` builds. It serves the six tools of `DES-MCP-TOOLS`, and `list_issues` and `show_issues` carry a read-only hint. Each tool's arguments are a serde type that also yields its input schema, through schemars as rmcp re-exports it, so `Cargo.toml` gains only `rmcp` and `tokio`. Unknown parameters and wrong types give `usage`. Results carry the CLI's `--json` document both as text, serialized like the CLI, and as structured content. Failures are tool errors carrying the same error object. The project is resolved on every call: from `directory`, which must be absolute, then the server's `-C`, then its working directory. Without `directory`, `project_not_found` also suggests passing it. `zban mcp --json` is a `usage` error.
- Agent guide, as the maintainer asked: `src/guide.md` stays the single template and is surface-neutral where it names an operation. Tags such as `{{update id=ID status=in-progress}}`, `{{synopsis update}}` and `{{param force}}` are rendered in `src/guide.rs` for `Surface::Cli` or `Surface::Mcp`. Inline `{{cli}}`/`{{mcp}}` blocks hold the little prose that differs. The operations and parameters come from one table, `OPERATIONS`. Tests check the rendered guides against clap and against the tool schemas, in both directions. `operations::guide` takes the surface. `guide::NOTE` is the short note for a server started outside a project. `tests/snapshots/prime.md` is byte-for-byte unchanged. After rebasing onto zb-hqez4b, the guide's `fix` line and the MCP `fix` parameter description also mention deleting leftover temporary files.
- Choices within the design, now recorded in `DES-MCP-TOOLS`:
  - Tool parameters mirror the command's options: underscores for hyphens, a list for a repeatable option, a boolean for a flag, and `title`, `id` or `ids` for positional arguments.
  - A check that finds errors is a result, not a tool error.
- Docs: `docs/mcp.mara.md` gains a "Tools" section with CLI-to-MCP examples. The "not implemented" notes in `docs/mcp.mara.md`, `docs/cli.mara.md`, `docs/agents.mara.md` and the README now name only the handoff options, which zb-qkj8tj adds to both surfaces.
- Tests: `tests/common/mcp.rs` is a small stdio MCP client: the initialize handshake, `tools/list` and `tools/call`, with newline-delimited JSON-RPC against the real binary. `tests/mcp.rs` has 8 tests for `VER-SURFACE-PARITY`:
  - the tool list and capabilities;
  - a create, block, ready, update, close, check and delete flow on twin fixtures, equal to the CLI's `--json` output after mapping paths, new IDs and new timestamps;
  - 22 invalid requests with the same error codes, with full error objects compared except for `usage`, and nothing written;
  - a check with an error finding, and `--fix`;
  - a relative `directory`;
  - every tool outside a project;
  - `directory` into a worktree copy and into another project, leaving the server's project unchanged, and the server's `-C`;
  - `mcp --json`.
- `tests/prime.rs` adds the `VER-AGENT-GUIDE` MCP checks: the `tests/snapshots/mcp.md` snapshot, delivery from a subdirectory, with `-C` and with an invalid configuration, and the short note outside a project. Every tool, parameter and argument type the MCP guide names exists in the schemas. Each tool's reference line lists exactly its schema's parameters, with their required flags and types. Each command's reference line lists exactly its clap options and positional arguments. The vocabulary, section and 150-line checks now cover both renderings.

Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (133 passed after rebasing onto `main` at 9046492: 36 unit, 8 `check`, 18 `cli`, 5 `ids`, 11 `issue_files`, 12 `lifecycle`, 3 `lifecycle_cli`, 8 `mcp`, 2 `no_git`, 14 `prime`, 6 `project_layout`, 10 `relations`), and `mara schema validate` and `mara project validate` (`valid: true`, evaluation complete, no diagnostics), and `zban check` on this repository after closing this issue (no findings, exit 0). Mutation checks: renaming a table parameter, or adding a clap option missing from the table, makes the guide tests fail; dropping a list filter in the MCP adapter makes the parity flow fail.
