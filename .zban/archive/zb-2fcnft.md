---
title: Decide on npm distribution
status: done
type: task
discovered_from: zb-aapz52
created_at: 2026-11-12T11:12:41Z
closed_at: 2026-11-22T12:10:37Z
---

zban might later also be distributed through npm, for example so that agents can start it with `npx`. `ADR-RUST-BINARY` limits v1 to GitHub Releases and requires a decision before any package-manager wrapper is added. Record that decision in Mara and have the maintainer accept it before any packaging work starts.

- [x] Draft a need and a decision for npm distribution in Mara, with the alternatives and their costs
- [x] Ask the maintainer to accept or reject the decision

## Summary of Changes

- Recorded `NEED-NPM-DISTRIBUTION` (`docs/product.mara.md`) and `ADR-NPM-DISTRIBUTION` (`docs/decisions.mara.md`) with the maintainer's accepted decision: distribute `@convesoft/zban` plus four per-platform binary packages (`@convesoft/zban-linux-x64`, `@convesoft/zban-linux-arm64`, `@convesoft/zban-darwin-arm64`, `@convesoft/zban-darwin-x64`) via npm OIDC Trusted Publishing alongside GitHub Releases, using Nx with `@convesoft/nx-rust` while keeping `Cargo.toml` as the single version source of truth.
- Updated `ADR-RUST-BINARY` to cross-reference `ADR-NPM-DISTRIBUTION`.
- Filed and implemented the follow-up delivery issue zb-cegs21.
- Checks run: `mara schema validate` and `mara project validate` (`valid: true`, `evaluation_complete: true`, 0 errors, 0 warnings), `zban check` (0 findings).
