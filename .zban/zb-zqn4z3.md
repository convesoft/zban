---
title: Harden the release workflow
status: todo
type: task
priority: low
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
---

Found by the v0.1.0 review (`ADR-RUST-BINARY`): a tag push runs no tests and does not require the tagged commit to be on `main`, and the archives carry no notices for the statically linked third-party crates.

- [ ] Run the test suite before creating a release
- [ ] Refuse a tag whose commit is not on `main`
- [ ] Decide how to ship third-party license notices (a tool such as cargo-about needs a recorded decision)
