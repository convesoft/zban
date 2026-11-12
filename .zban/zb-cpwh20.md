---
title: Quote YAML 1.1 boolean words when writing headers
status: draft
type: bug
priority: low
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
---

Found by the v0.1.0 review: a title such as `yes` is written plain, which is correct YAML 1.2 (`DES-ISSUE-FILE`), but YAML 1.1 readers such as PyYAML read it as a boolean. Quoting `yes`, `no`, `on`, `off` and their case variants would change the canonical form, so it needs a `DES-ISSUE-FILE` decision first.
