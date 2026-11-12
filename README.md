# zban

**zban is a file-based issue tracker for you and your coding agents.** Issues are
Markdown files in your repository, managed through a CLI and an MCP server.

> **Status:** the v1 specification is agreed; implementation is in progress.
> `init`, `create`, `list`, `show`, `update` and `delete` work; `check`, `prime` and
> `mcp` do not yet. Progress is tracked in [`.zban/`](.zban/).

## Why

Coding agents lose track of work between sessions, after context compaction and
when they hand work to each other. zban gives agents and humans one shared backlog
that lives next to the code and is versioned with it:

- **Plain files.** One Markdown file per issue in `.zban/`, readable and editable
  without zban.
- **Branches and merges like code.** No server, no database. zban never runs Git,
  so issue changes travel with your commits.
- **Built for agents.** JSON output for every command, the same operations as MCP
  tools, a computed list of ready work, and a built-in agent guide (`zban prime`).
- **Small on purpose.** Fixed types, statuses and priorities, and a short command
  set. See what zban [deliberately does not do](docs/product.mara.md#what-zban-deliberately-does-not-do).

## What an issue looks like

`.zban/zb-n938cw.md`:

```markdown
---
title: Parse issue files
status: in-progress
type: feature
priority: high
parent: zb-9jra34
blocked_by:
  - zb-naex76
created_at: 2026-11-11T20:00:00Z
---

- [x] Read frontmatter
- [ ] Preserve unknown keys
```

## Planned usage

```sh
zban init
zban create "Parse issue files" -t feature -p high
zban list --ready
zban update zb-n938cw -s in-progress
zban show zb-n938cw --json
zban mcp            # MCP server over stdio
zban prime          # agent guide for CLI-only agents and hooks
```

## Documentation

The [documentation](docs/index.mara.md) is both the user guide and the engineering
specification:

- [What zban is](docs/product.mara.md)
- [Issues](docs/issues.mara.md)
- [Command line](docs/cli.mara.md)
- [MCP server](docs/mcp.mara.md)
- [Using zban with agents](docs/agents.mara.md)
- [Decisions and risks](docs/decisions.mara.md)
- [Verification](docs/verification.mara.md)

## Development

zban is written in Rust. The specification is maintained with
[Mara](https://github.com/convesoft/mara); see [`.mara/README.md`](.mara/README.md)
for setup. Contributors and coding agents start with [AGENTS.md](AGENTS.md).

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
mara schema validate && mara project validate
```

## License

zban will be available under your choice of the MIT license or the Apache License
2.0. The license files will be added before the first release.
