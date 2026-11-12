---
title: Make the locking unit test pass on the musl release targets
status: in-progress
type: bug
discovered_from: zb-zqn4z3
created_at: 2026-11-12T15:26:03Z
---

Found while hardening the release workflow (zb-zqn4z3). `store::tests::a_write_proceeds_unlocked_only_where_locking_is_unsupported` in `src/store.rs` (`VER-ISSUE-FILES`) fails on `x86_64-unknown-linux-musl`, the Linux target that `ADR-RUST-BINARY` ships. It checks that the operating system's message for each code in `LOCKING_UNSUPPORTED` contains "not supported" or "not implemented", but musl's message for 95 is "Not supported", with a capital N, where glibc's is "Operation not supported". The lock handling itself is right on musl, because it compares the error code, not the message; only the test depends on the C library.

Reproduce with `rustup target add x86_64-unknown-linux-musl` and `cargo test --release --locked --target x86_64-unknown-linux-musl`. Every other test passes there.

Because of this, the Test step in `release.yml` runs `cargo test --locked` for the runner's own target, not for the musl target the release ships.

- [x] Make the test's message check independent of the C library, for example by ignoring letter case
- [x] Run the Test step in `release.yml` with `--target ${{ matrix.target }}`, so that a release tests the targets it ships
- [ ] `cargo test --locked --target x86_64-unknown-linux-musl` passes locally, and all four Build jobs pass on a build-only run of `release.yml`
