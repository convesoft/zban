---
title: Review the decisions made in 1.0.0-rc.1
status: todo
type: task
tags:
  - maintainer
related:
  - zb-v90b0k
created_at: 2026-11-22T12:59:51Z
---

The orchestrator of milestone zb-v90b0k records here, one line each, the decisions made on the maintainer's behalf while the milestone ran: what was decided, which Mara items it rests on and how to undo it. Each decision is also written into the Mara item it belongs to.

This issue is for the maintainer to read before releasing `1.0.0-rc.1`. It is never dispatched to an agent.

## Decisions

- The milestone ships as `1.0.0-rc.1` under the npm dist-tag `next` instead of `0.2.0`: the maintainer decided this on 2026-11-22, because the v1 specification is implemented and no further enhancements are queued. It rests on `ADR-NPM-DISTRIBUTION` (the version comes from `Cargo.toml`, bumped with `pnpm nx release version <version>` in a release pull request). Undo: bump to `0.2.0` instead and rename zb-v90b0k back.

## Done when

- [ ] The maintainer has read every line under `## Decisions` and reversed or accepted each one
