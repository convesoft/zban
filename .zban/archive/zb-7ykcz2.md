---
title: Pin the Rust toolchain and check the minimum version in CI
status: done
type: task
created_at: 2026-11-12T19:52:21Z
closed_at: 2026-11-12T20:03:07Z
---

Both workflows install whatever `stable` Rust is current. When Rust 1.100.0 shipped, its rustfmt reformatted `src/issue.rs` and `main` failed CI without any change (fixed by PR #45). Release builds are not reproducible either, and the declared minimum `rust-version` in `Cargo.toml` is never checked. Record the pin and the minimum check in `ADR-RUST-BINARY`.

- [x] Record the pinned toolchain and the minimum-version check in `ADR-RUST-BINARY`
- [x] Add `rust-toolchain.toml` pinning 1.100.0 with the minimal profile, rustfmt, clippy and the four release targets
- [x] Make `ci.yml` and `release.yml` install the pinned toolchain instead of floating `stable`, keeping the release workflow's per-target behaviour
- [x] Add a Linux CI job that runs `cargo test --locked` with the `rust-version` read from `Cargo.toml`
- [x] Mention the pin in the README

## Done when

- [x] `actionlint` accepts both workflows
- [x] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass with the pinned toolchain
- [x] `cargo +1.89 test --locked` passes
- [x] `mara schema validate` and `mara project validate` report a valid project with complete evaluation
- [x] `zban check` reports no errors
- [x] The build-only Release run on the pull request passes for all four targets, and CI passes, including the minimum-version job

## Summary of Changes

- `rust-toolchain.toml` pins Rust 1.100.0 with the minimal profile, `rustfmt`, `clippy` and the four release targets.
- `ci.yml` and `release.yml` install it with `rustup toolchain install`, which reads the file. No job installs floating `stable`, and `dtolnay/rust-toolchain` is no longer used. Each release runner still builds, tests, smoke-tests and packages only its own targets, but it now installs the standard library for all four targets because the file lists them. The Release workflow's pull request paths include `rust-toolchain.toml`, so a toolchain bump gets the build-only release run.
- New CI job "Test (minimum Rust version)" on `ubuntu-latest`: it reads `rust-version` from `Cargo.toml` with Python's `tomllib`, installs that toolchain with the minimal profile and runs `cargo test --locked` with it through `RUSTUP_TOOLCHAIN`.
- `ADR-RUST-BINARY` records the pin, its deliberate bump in a pull request of its own, the minimum-version check and why the floating `stable` channel was rejected. The README's Development section describes the pin, says to rerun `rustup component add rust-analyzer` after a bump (Mara's code indexer needs it, and the maintainer chose to keep it out of the pinned components), and adds `cargo +1.89 test --locked` to the checks.
- Checks run on x86_64 Linux on 2026-11-12, after merging `origin/main` with PR #45: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass with 1.100.0, and `cargo +1.89 test --locked` passes. `mara schema validate` and `mara project validate` report a valid project with complete evaluation and no diagnostics. `zban check` reports no findings. `actionlint` 1.7.12 accepts both workflows; with shellcheck 0.11.0 it also reports two SC1010 warnings on the smoke test's `--status done`, which `main` already has.
- On the pull request's previous head, CI passed "Test (ubuntu-latest)", "Test (macos-15)" and "Test (minimum Rust version)" and failed only the rustfmt diff that PR #45 fixes. The Release run had passed both Linux builds, including `ubuntu-24.04-arm`. CI and the build-only Release run on this pull request cover the last check.
- Rejected: `dtolnay/rust-toolchain@master` with the version read from the file, because it adds a third-party action and separates the components and targets from the file. Reading `rust-version` with `cargo metadata`, because that would first install the pinned toolchain. Adding `rust-analyzer` to the pinned components, because CI does not need it.
