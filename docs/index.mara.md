# zban documentation

These pages serve as the user guide and as zban's engineering knowledge. Each
topic page opens with guidance for users, followed by the requirements and
design that govern that topic.

## For users

| Page | What it covers |
| --- | --- |
| [What zban is](product.mara.md) | Purpose, scope, what zban deliberately does not do, goals and vocabulary |
| [Issues](issues.mara.md) | The issue file, types, statuses, priorities, relations, ready work and the archive |
| [Command line](cli.mara.md) | Commands, options and JSON output |
| [MCP server](mcp.mara.md) | Connecting MCP clients and the available tools |
| [Using zban with agents](agents.mara.md) | Agent setup, the agent guide and a typical session |

## For contributors

| Page | What it covers |
| --- | --- |
| [Decisions and risks](decisions.mara.md) | Why zban is shaped the way it is, and the risks it accepts |
| [Verification](verification.mara.md) | How each requirement and goal is checked |

## How to read the specification

The `:::mara` blocks are [Mara](https://github.com/convesoft/mara) items: goals
(`GOAL-`), needs (`NEED-`), terms (`TERM-`), scenarios (`SCN-`), requirements
(`REQ-`), designs (`DES-`), decisions (`ADR-`), risks (`RISK-`), verifications
(`VER-`) and validations (`VAL-`). Lines such as `:satisfies: NEED-…` connect
items to each other. `[[ID]]` mentions another item.

`status: accepted` means the knowledge is agreed. It does not mean the feature
is implemented or tested. `draft` marks open proposals, and `retired` marks
history that no longer applies.

The work to build zban is tracked in [`.zban/`](../.zban/) using zban's own
issue format. Issues refer to these items by ID instead of repeating them.
