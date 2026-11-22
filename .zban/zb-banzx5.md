---
title: Make draft ancestors hold back descendants in readiness
status: in-progress
type: feature
priority: high
tags:
  - readiness
  - guide
parent: zb-v90b0k
created_at: 2026-11-22T10:56:27Z
---

Currently a `draft` ancestor does not hold its `todo` descendants back in readiness (`DES-READINESS`). In practice this caused planned epics with `todo` children to be dispatched immediately under autonomous orchestrators, and forced every orchestrator prompt to filter out parked and human-only work with ad-hoc tag/priority conventions (`backlog`, `deferred`, `maintainer`).

## Tasks

- [ ] Record the decision in Mara (`docs/decisions.mara.md`) and update `DES-READINESS`, `docs/issues.mara.md`, `DES-AGENT-GUIDE`, `VER-RELATIONS-READY` and `VER-AGENT-GUIDE`
- [ ] Hold back descendants of any `draft` ancestor in readiness with reason `ancestor_draft:<id>`
- [ ] Update `src/guide.md` (keeping within the MCP character budget in `DES-AGENT-GUIDE`) so agents park planned epics, backlog and human-only work as `draft`
- [ ] Update readiness and guide tests (including real-binary/MCP checks per `VER-RELATIONS-READY` and `VER-AGENT-GUIDE`)

## Done when

- [ ] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [ ] `zban check` reports no errors
