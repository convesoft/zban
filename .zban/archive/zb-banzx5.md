---
title: Make draft ancestors hold back descendants in readiness
status: done
type: feature
priority: high
tags:
  - readiness
  - guide
parent: zb-v90b0k
created_at: 2026-11-22T10:56:27Z
closed_at: 2026-11-22T13:01:09Z
---

Currently a `draft` ancestor does not hold its `todo` descendants back in readiness (`DES-READINESS`). In practice this caused planned epics with `todo` children to be dispatched immediately under autonomous orchestrators, and forced every orchestrator prompt to filter out parked and human-only work with ad-hoc tag/priority conventions (`backlog`, `deferred`, `maintainer`).

## Tasks

- [x] Record the decision in Mara (`docs/decisions.mara.md`) and update `DES-READINESS`, `docs/issues.mara.md`, `DES-AGENT-GUIDE`, `VER-RELATIONS-READY` and `VER-AGENT-GUIDE`
- [x] Hold back descendants of any `draft` ancestor in readiness with reason `ancestor_draft:<id>`
- [x] Update `src/guide.md` (keeping within the MCP character budget in `DES-AGENT-GUIDE`) so agents park planned epics, backlog and human-only work as `draft`
- [x] Update readiness and guide tests (including real-binary/MCP checks per `VER-RELATIONS-READY` and `VER-AGENT-GUIDE`)

## Done when

- [x] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [x] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [x] `zban check` reports no errors

## Summary of Changes

A `draft` ancestor now holds back every descendant in readiness, with one `ancestor_draft:<id>` reason per `draft` ancestor (`src/relations.rs`). The agent guide (`src/guide.md`) tells agents to park planned epics, backlog and human-only work as `draft` and states the new readiness rule. To stay within the MCP budget it drops two redundant phrases, "An epic groups other issues." and the parenthetical after `discovered_from`, which the "Track work" and "While working" sections already convey. The MCP rendering is 3,787 of 3,800 characters. Mara: new decision `ADR-DRAFT-HOLDS-DESCENDANTS`; updated `DES-READINESS`, the status table and the "Ready work" text in `docs/issues.mara.md`, `DES-AGENT-GUIDE`, `ADR-FIXED-VOCABULARY`, `VER-RELATIONS-READY` and `VER-AGENT-GUIDE`. Tests: the readiness fixture gains a grandchild of the draft and a child of an `in-progress` issue; a new library test covers stacked draft and blocked reasons and promotion to `todo`; a new real-binary and MCP parity test covers a draft epic's subtree; a new guide test checks the parking line and the rule; the guide snapshots are updated.

## Decisions made

- `ancestor_draft` comes last among the reasons, after `ancestor_blocked`, so the reasons that existing output already reports keep their order. Like the other ancestor reasons, it is reported once per `draft` ancestor, nearest first (`DES-READINESS`).
- An `in-progress` ancestor still holds nothing back, because its children carry its work (`ADR-DRAFT-HOLDS-DESCENDANTS`).
- The guide's parking line sits in "Track work in zban", and the readiness rule in "Reference" names `draft` ancestors (`DES-AGENT-GUIDE`). Two redundant phrases were cut to fit the budget (see above).

Checks run: `mara schema validate` (valid), `mara project validate` (`valid: true`, evaluation complete, 20 of 20 policy checks passed), `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all suites pass) and `zban check` (no findings).
