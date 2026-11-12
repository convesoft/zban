---
title: Close the lost-update window between concurrent updates
status: todo
type: bug
priority: high
parent: zb-9jra34
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
---

Found by the v0.1.0 review: two `zban update` runs on one issue at the same moment can both pass the unchanged-bytes comparison, and the later rename drops the earlier change while both exit 0 (31 of 300 rounds on tmpfs, 13 of 200 on btrfs). This breaks `REQ-NO-LOST-UPDATES`; see `DES-SHARED-CORE` and `VER-ISSUE-FILES`.

Maintainer decision: hold a short OS advisory lock on the `.zban/` directory only across the compare-and-rename (and compare-and-delete) step, using std `File::lock` with no new dependency and no lock file. Record it in Mara first, replacing the current "no locks" wording, and set `rust-version` in `Cargo.toml` to the minimum the code then needs.

- [ ] Mara updated: `REQ-NO-LOST-UPDATES`, `DES-SHARED-CORE`, the user docs on simultaneous changes, and `VER-ISSUE-FILES`
- [ ] Lock taken around every compare-then-write in the store
- [ ] `rust-version` set
- [ ] A test runs many parallel updates on one issue and loses none
