# zban

**zban is a file-based issue tracker for you and your coding agents.** Issues are
Markdown files in your repository, managed through a CLI and an MCP server.

> **Status:** the v1 specification is agreed and implemented. Releases are
> published on [GitHub Releases](https://github.com/convesoft/zban/releases), and
> progress is tracked in [`.zban/`](.zban/).

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

## Installation

zban runs on Linux and macOS, on x86_64 and aarch64. It is distributed only
through [GitHub Releases](https://github.com/convesoft/zban/releases), not through
crates.io or other package managers.

### Prebuilt binary

Each release attaches one archive per platform, holding the `zban` binary, this
README and the license files, and a `SHA256SUMS` file:

| Platform | Archive |
| --- | --- |
| Linux, x86_64 | `zban-x86_64-unknown-linux-musl.tar.gz` |
| Linux, aarch64 | `zban-aarch64-unknown-linux-musl.tar.gz` |
| macOS, Apple silicon | `zban-aarch64-apple-darwin.tar.gz` |
| macOS, Intel | `zban-x86_64-apple-darwin.tar.gz` |

The Linux binaries are statically linked and run on any distribution. To install
the latest release into `~/.local/bin`, set `target` to your platform's archive
name:

```sh
target=x86_64-unknown-linux-musl
curl -fsSLO "https://github.com/convesoft/zban/releases/latest/download/zban-$target.tar.gz"
curl -fsSLO https://github.com/convesoft/zban/releases/latest/download/SHA256SUMS
sha256sum -c --ignore-missing SHA256SUMS   # macOS: shasum -a 256 -c --ignore-missing SHA256SUMS
mkdir -p ~/.local/bin
tar -xzf "zban-$target.tar.gz" -C ~/.local/bin zban
zban --version
```

Make sure `~/.local/bin` is on your `PATH`. The macOS binaries are not notarized
by Apple. If you downloaded the archive with a browser instead of `curl`, macOS
blocks the binary until you run `xattr -d com.apple.quarantine ~/.local/bin/zban`.

### From source

With Rust 1.89 or later, build a release tag, replacing `vX.Y.Z` with the
release you want, such as `v0.1.0`:

```sh
cargo install --locked --git https://github.com/convesoft/zban --tag vX.Y.Z
```

## Usage

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

Pushing a tag `vX.Y.Z` that matches the version in `Cargo.toml` on a commit of
`main` runs [`release.yml`](.github/workflows/release.yml), which runs the
tests, builds the binaries and creates the GitHub release. A tag that names
another version or a commit outside `main` fails the run, and no release is
created.

## License

zban is available under your choice of the [MIT license](LICENSE-MIT) or the
[Apache License 2.0](LICENSE-APACHE).

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in zban by you, as defined in the Apache-2.0 license, shall be dual
licensed as above, without any additional terms or conditions.
