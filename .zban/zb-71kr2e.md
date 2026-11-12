---
title: Smoke-test the release binaries
status: todo
type: task
priority: high
parent: zb-9jra34
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
---

Found by the v0.1.0 review: the release workflow only runs each built binary with `--version` (`ADR-RUST-BINARY`).

- [ ] For each target that runs on its build machine, the release workflow runs init, create, update, list and check against a scratch project
- [ ] The build-only run on the pull request passes for all four targets
