---
title: Count code fences only where Markdown does
status: todo
type: bug
priority: high
parent: zb-9jra34
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
---

Found by the v0.1.0 review: a fence counts at any indentation, so an indented "```" in prose hides every later task item from the close check (`DES-LIFECYCLE`) and every later heading from handoff detection (`DES-HANDOFF`).

Maintainer decision: a fence opens only when indented by at most three spaces, or inside a list item, as in Markdown. Record this in `DES-LIFECYCLE` and `DES-HANDOFF` first.

- [ ] Mara updated
- [ ] The shared fence scanning follows the rule, with tests for both task counting and handoff detection
