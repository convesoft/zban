---
title: Update how agents run zban here after the release
status: todo
type: task
priority: low
parent: zb-9jra34
blocked_by:
  - zb-aapz52
discovered_from: zb-as3s58
created_at: 2026-11-12T10:14:08Z
---

`AGENTS.md` tells agents to run zban from their checkout with `cargo run -q -- <command>`, because zban has no release yet. Once zb-aapz52 publishes one, that sentence is out of date.

- [ ] Decide whether agents here use the released `zban` or keep running the checkout's build, which always reads this branch's issue format
- [ ] Update `AGENTS.md` to match
