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
- [x] `zban check` reports no errors
- [ ] The build-only Release run on the pull request passes for all four targets, and CI passes, including the minimum-version job

## Handoff (2026-11-12T19:56:57Z)

- Branch `zb-7ykcz2`, pushed, PR https://github.com/convesoft/zban/pull/46. The work is in commit 5862108; this note's commit follows it. The working tree is clean.
- Done: `rust-toolchain.toml` pins 1.100.0 (minimal profile, rustfmt, clippy, the four release targets). `ci.yml` and `release.yml` install it with `rustup toolchain install`. The release workflow's pull request paths include `rust-toolchain.toml`. New CI job "Test (minimum Rust version)" reads `rust-version` with Python's `tomllib` and sets `RUSTUP_TOOLCHAIN`. `ADR-RUST-BINARY` and the README Development section are updated.
- Passed locally on x86_64 Linux: clippy and `cargo test` with 1.100.0, `cargo +1.89 test --locked`, both Mara validations (valid, evaluation complete, no diagnostics), `zban check`, and `actionlint` 1.7.12. With shellcheck, actionlint also reports two SC1010 warnings on the smoke test's `--status done`, which `main` already has.
- Blocked: `cargo fmt --all -- --check` fails on `src/issue.rs` with rustfmt 1.100 until PR #45 merges. Expect the CI "Format and clippy" job to fail until then.
- Next: once #45 is merged, merge `origin/main` into this branch with git, rerun fmt, clippy and test, tick the fmt/clippy/test item, then tick the CI item and close the issue under the AGENTS.md CI rule, and push. Confirm that the Release run passes for all four targets and that CI passes, including the minimum-version job.
- Uncertain: whether every runner image's rustup is at least 1.28, which `rustup toolchain install` without arguments needs. Watch the first runs, especially `ubuntu-24.04-arm`.
- Rejected: `dtolnay/rust-toolchain@master` with the version read from the file, because it adds a third-party action and splits the components and targets from the file. `cargo metadata` for reading `rust-version`, because it would first install the pinned toolchain. Adding `rust-analyzer` to the pinned components, because the maintainer set the component list; it is an open question.
- Files: `rust-toolchain.toml`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `docs/decisions.mara.md`, `README.md`, `.zban/zb-7ykcz2.md`.
