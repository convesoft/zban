---
title: Set up crate layout and CI
status: done
type: task
priority: high
parent: zb-9jra34
created_at: 2026-11-11T21:00:01Z
closed_at: 2026-11-12T07:47:24Z
---

Library and binary layout per `DES-SHARED-CORE`, toolchain per `ADR-RUST-BINARY`, dependencies per `ADR-DEPENDENCIES`.

- [x] Library crate with the `zban` binary
- [x] CI runs formatting, clippy and tests on pull requests and `main`

## Summary of Changes

- `src/lib.rs` starts the core library, still empty. `src/main.rs` is the `zban` binary: a `clap` parser with `--help` and `--version` only. Running it with no arguments prints help and exits 2. Commands come with their own issues.
- Added `clap` (derive) from `ADR-DEPENDENCIES`. The other named crates are left to the issues that first use them.
- `tests/cli.rs` runs the real binary. It checks the `--version` output, exit status 2 for an unknown option, and help with exit status 2 when no arguments are given. The issue cites no `VER-` item, so these are smoke tests for the layout.
- `.github/workflows/ci.yml` runs on pull requests and on pushes to `main`. `cargo fmt --all -- --check` and `cargo clippy --locked --all-targets -- -D warnings` run on Ubuntu. `cargo test --locked` runs on `ubuntu-latest` (x86_64) and `macos-latest` (aarch64), the platforms in `ADR-RUST-BINARY`. It uses the stable toolchain, and no MSRV is set.
- Maintainer decision: Mara validation does not run in CI for now. The checklist item "CI runs `mara schema validate` and `mara project validate`" was dropped, and the workflow has no Mara step.

Checks run locally: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (3 passed), `mara schema validate` and `mara project validate` (`valid: true`, evaluation complete).
