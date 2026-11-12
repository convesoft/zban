---
title: Note an enclosing project on init
status: todo
type: feature
priority: high
parent: zb-9jra34
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
---

Found by the v0.1.0 review: `zban init` inside an existing project silently creates a nested project, and commands run below it then see an empty backlog (`DES-PROJECT-LAYOUT`).

Maintainer decision: allow nesting, but `init` prints a note naming the enclosing project (and returns it in JSON). Record it in `DES-PROJECT-LAYOUT` and `DES-CLI` first.

- [ ] Mara updated
- [ ] CLI behaviour and JSON field, with tests
