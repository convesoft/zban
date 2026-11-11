# MCP server

`zban mcp` serves zban to MCP clients over stdio. It provides the same operations
as the [command line](cli.mara.md) and tells the client how to use them.

> zban is not implemented yet. This page describes the agreed v1 interface.

## Setup

Claude Code, in the project's `.mcp.json`:

```json
{
  "mcpServers": {
    "zban": { "command": "zban", "args": ["mcp"] }
  }
}
```

Codex, in `~/.codex/config.toml` (start Codex from the project directory):

```toml
[mcp_servers.zban]
command = "zban"
args = ["mcp"]
```

The server uses the project found from its working directory. Pass `--root PATH`
in `args` to pin a project.

## Requirement

:::mara requirement REQ-MCP-STDIO
:mid: 01M764FJ3ZY1HWBNVEGEVYMDFR
:title: Serve zban over MCP
:status: accepted
:satisfies: NEED-FIND-NEXT
:satisfies: NEED-TRACK-PROGRESS

`zban mcp` runs an MCP server over stdio for the project found from its working directory or given by `--root`. It exposes listing, showing, creating, updating, deleting and checking issues as tools whose input schemas accept the same arguments as the command line.
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
| `update_issue` | `update`, including body edits and `force` |
| `delete_issue` | `delete` |
| `check_issues` | `check`, with optional `fix` |

Each tool result carries the JSON document the CLI prints with `--json`, both as structured content and as text. Failures are tool errors with the same error codes. The server's `instructions` are the agent guide rendered for MCP ([[DES-AGENT-GUIDE]]). v1 offers no resources or prompts, and no tool for `init`, which is a one-time human action.
:::
