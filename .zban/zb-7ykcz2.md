---
title: Pin the Rust toolchain and check the minimum version in CI
status: in-progress
type: task
created_at: 2026-11-12T19:52:21Z
---

Both workflows install whatever `stable` Rust is current. When Rust 1.100.0 shipped, its rustfmt reformatted `src/issue.rs` and `main` failed CI without any change (fixed by PR #45). Release builds are not reproducible either, and the declared minimum `rust-version` in `Cargo.toml` is never checked. Record the pin and the minimum check in `ADR-RUST-BINARY`.

- [x] Record the pinned toolchain and the minimum-version check in `ADR-RUST-BINARY`
- [x] Add `rust-toolchain.toml` pinning 1.100.0 with the minimal profile, rustfmt, clippy and the four release targets
- [x] Make `ci.yml` and `release.yml` install the pinned toolchain instead of floating `stable`, keeping the release workflow's per-target behaviour
- [x] Add a Linux CI job that runs `cargo test --locked` with the `rust-version` read from `Cargo.toml`
- [x] Mention the pin in the README

## Done when

- [x] `actionlint` accepts both workflows
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass with the pinned toolchain
- [x] `cargo +1.89 test --locked` passes
- [x] `mara schema validate` and `mara project validate` report a valid project with complete evaluation
- [ ] `zban check` reports no errors
- [ ] The build-only Release run on the pull request passes for all four targets, and CI passes, including the minimum-version job
