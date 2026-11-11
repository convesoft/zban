---
title: Set up crate layout and CI
status: todo
type: task
priority: high
parent: zb-9jra34
created_at: 2026-11-11T21:00:01Z
---

Library and binary layout per `DES-SHARED-CORE`, toolchain per `ADR-RUST-BINARY`.

- [ ] Library crate with the `zban` binary
- [ ] CI runs formatting, clippy and tests on pull requests and `main`
- [ ] CI runs `mara schema validate` and `mara project validate`
