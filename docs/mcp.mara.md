# MCP server

`zban mcp` serves zban to MCP clients over stdio. It provides the same operations
as the [command line](cli.mara.md) and tells the client how to use them.

> zban is not implemented yet. This page describes the agreed v1 interface.

## Setup

One server can serve every project, so you can configure it once for your user.

Claude Code, for all projects:

```sh
claude mcp add --scope user zban -- zban mcp
```

or for one project, in its `.mcp.json`:

```json
{
  "mcpServers": {
    "zban": { "command": "zban", "args": ["mcp"] }
  }
}
```

Codex, in `~/.codex/config.toml`:

```toml
[mcp_servers.zban]
command = "zban"
args = ["mcp"]
```

## Choosing the project

Every tool takes an optional `directory`: an absolute path that zban searches
upward from, as if it had been started there. Agents pass their working
directory, which matters when they work in a worktree or outside the directory
the server started in. Otherwise the tool would change issues in a different
checkout, on a different branch.

Without `directory`, the server uses the directory given with `-C` in `args`,
or else its own working directory. If no project is found, the server still
starts, but its tools report `project_not_found`.

The server sends the full [agent guide](agents.mara.md) only when it starts
inside a zban project. Otherwise it sends a short note, so a user-scope server
costs almost nothing in sessions outside zban projects. See the known limitation
in [Using zban with agents](agents.mara.md#setup).

## Requirement

:::mara requirement REQ-MCP-STDIO
:mid: 01M764FJ3ZY1HWBNVEGEVYMDFR
:title: Serve zban over MCP
:status: accepted
:satisfies: NEED-FIND-NEXT
:satisfies: NEED-TRACK-PROGRESS
:satisfies: NEED-BRANCH-SAFE

`zban mcp` runs an MCP server over stdio. Each tool call may name the directory to search for the project from, so one server serves any project or worktree. Without it, the server searches from the directory given with `-C`, or else from its working directory. It exposes listing, showing, creating, updating, deleting and checking issues as tools whose input schemas accept the same arguments as the command line.
:::

## Design

:::mara design DES-MCP-TOOLS
:mid: 01M764FJ3Z6VP70QDFVWGMZFB8
:title: MCP tools and server instructions
:status: accepted
:satisfies: REQ-MCP-STDIO
:satisfies: REQ-AGENT-GUIDE

| Tool | Operation |
| --- | --- |
| `list_issues` | `list` |
| `show_issues` | `show`, for one or more IDs |
| `create_issue` | `create` |
| `update_issue` | `update`, including body edits, the handoff and `force` |
| `delete_issue` | `delete` |
| `check_issues` | `check`, with optional `fix` |

Every tool accepts an optional `directory`, an absolute path with the meaning of the CLI's `-C`: the project is searched upward from it. A relative path is rejected. The project is resolved on every call, from the call's `directory`, then the server's `-C`, then the server's working directory.

Each tool result carries the JSON document the CLI prints with `--json`, both as structured content and as text. Failures are tool errors with the same error codes. The server's `instructions` are the agent guide rendered for MCP, in full when a project is found at startup and as the short note otherwise ([[DES-AGENT-GUIDE]]). When no project is found, the server still starts, and every tool without a `directory` fails with `project_not_found`, with a message suggesting `directory`. v1 offers no resources or prompts, and no tool for `init`, which is a one-time human action.
:::
