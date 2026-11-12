---
title: Release v0.1.0
status: in-progress
type: task
parent: zb-9jra34
blocked_by:
  - zb-yj8xta
  - zb-bbyjed
  - zb-as3s58
  - zb-qkj8tj
created_at: 2026-11-11T21:00:10Z
---

Distribution and licensing follow `ADR-RUST-BINARY`.

Maintainer decision (2026-11-12): zban is distributed only through GitHub Releases, as prebuilt binaries and as source builds with `cargo install --git ... --tag`, and is not published to crates.io. The items "Confirm the `zban` crate name is available" and "Publish to crates.io" were dropped. npm distribution needs its own decision first (zb-2fcnft).

- [x] Choose release target triples for the platforms in `ADR-RUST-BINARY`
- [x] Add `LICENSE-MIT` and `LICENSE-APACHE`
- [ ] Release workflow builds the binaries for every target, checked by a build-only run on the pull request
- [ ] The `v0.1.0` tag creates the GitHub release with the binaries and `SHA256SUMS` attached, and the README install commands work against it
- [x] README installation instructions match the release
- [x] `VAL-SCOPE-REVIEW` is done and its shortfalls are filed

## Scope review (`VAL-SCOPE-REVIEW`)

- Every requirement and design is accepted and traces through `satisfies` to an accepted need. `mara project validate` passes the acceptance policies.
- Since the v1 specification (#1), no command or option was added to `DES-CLI`, and `DES-ISSUE-FILE` is unchanged. The `DES-CLI` edits only clarified the order of body edits, absolute `path` values and the `invalid_handoff` cases.
- Shortfall: `zban list` accepts `-s`, `-t` and `-p`, which `DES-CLI` does not list. Filed as zb-x481ey.
- `zban prime` prints 127 lines and has every section of `DES-AGENT-GUIDE`. The `VER-AGENT-GUIDE` tests check the operations it names against clap and the MCP tool schemas.
- `VAL-DOGFOOD` cannot be judged yet: four issues have gone through zban since zb-as3s58, short of ten. zb-hr38j4 tracks it.
- Shortfall: `AGENTS.md` still says implementation is in progress. Added to zb-mq3eqv.
