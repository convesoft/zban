---
title: Correct DES-AGENT-GUIDE on which commands the CLI guide lists
status: todo
type: task
priority: deferred
tags:
  - deferred
  - backlog
discovered_from: zb-banzx5
created_at: 2026-11-22T13:09:00Z
---

`DES-AGENT-GUIDE` (`docs/agents.mara.md`, section 8 "Reference") says the command-line rendering of the guide "lists every command with its options". The guide deliberately leaves out the setup commands `init`, `prime`, `mcp` and `help` (`SETUP_COMMANDS` in `tests/prime.rs`, checked by `guide_lists_every_command_agents_use`).

Found while reviewing zb-banzx5. The gap predates that change.

## Tasks

- [ ] Reword `DES-AGENT-GUIDE` (and `VER-AGENT-GUIDE` if it repeats the claim) so that the reference lists every command agents use, leaving out the setup commands
- [ ] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
