---
title: Make the locking unit test pass on the musl release targets
status: done
type: bug
discovered_from: zb-zqn4z3
created_at: 2026-11-12T15:26:03Z
closed_at: 2026-11-12T16:15:06Z
---

Found while hardening the release workflow (zb-zqn4z3). `store::tests::a_write_proceeds_unlocked_only_where_locking_is_unsupported` in `src/store.rs` (`VER-ISSUE-FILES`) fails on `x86_64-unknown-linux-musl`, the Linux target that `ADR-RUST-BINARY` ships. It checks that the operating system's message for each code in `LOCKING_UNSUPPORTED` contains "not supported" or "not implemented", but musl's message for 95 is "Not supported", with a capital N, where glibc's is "Operation not supported". The lock handling itself is right on musl, because it compares the error code, not the message; only the test depends on the C library.

Reproduce with `rustup target add x86_64-unknown-linux-musl` and `cargo test --release --locked --target x86_64-unknown-linux-musl`. Every other test passes there.

Because of this, the Test step in `release.yml` runs `cargo test --locked` for the runner's own target, not for the musl target the release ships.

- [x] Make the test's message check independent of the C library, for example by ignoring letter case
- [x] Run the Test step in `release.yml` with `--target ${{ matrix.target }}`, so that a release tests the targets it ships
- [x] `cargo test --locked --target x86_64-unknown-linux-musl` passes locally, and all four Build jobs pass on a build-only run of `release.yml`

## Summary of Changes

- `store::tests::a_write_proceeds_unlocked_only_where_locking_is_unsupported` (`VER-ISSUE-FILES`) now compares the operating system's message for each number in `LOCKING_UNSUPPORTED` without letter case, so musl's "Not supported" for 95 passes as glibc's "Operation not supported" does. The lock helpers are unchanged. Replacing 95 with 59, or 38 with 35, still fails the test on both glibc and musl.
- The Test step in `release.yml` runs `cargo test --locked --target ${{ matrix.target }}`, so a release tests the targets it ships (`ADR-RUST-BINARY`).
- Checks run on the merged head 89568e7: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked` (176 passed) and `cargo test --locked --target x86_64-unknown-linux-musl` (176 passed) locally, plus `mara schema validate` and `mara project validate`. Release run 43253925537 passed all four Build jobs. Its Test step with `--target` passed on `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl` and `aarch64-apple-darwin`, and was skipped as designed on the cross-compiled `x86_64-apple-darwin`. CI run 43253925900 passed format, clippy and the Linux and macOS tests.
