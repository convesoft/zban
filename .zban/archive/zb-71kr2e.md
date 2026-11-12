---
title: Smoke-test the release binaries
status: done
type: task
priority: high
parent: zb-9jra34
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
closed_at: 2026-11-12T13:24:24Z
---

Found by the v0.1.0 review: the release workflow only runs each built binary with `--version` (`ADR-RUST-BINARY`).

- [x] For each target that runs on its build machine, the release workflow runs init, create, update, list and check against a scratch project
- [x] The build-only run on the pull request passes for all four targets

## Summary of Changes

- The Release workflow's "Smoke-test the binary" step replaces "Run the binary". On each native build it prints `--version`, then in a scratch project under `$RUNNER_TEMP`, outside the checkout, runs `init`, `create`, `list`, `update` (ticking the checklist and closing the issue), `list --status done` and `check`. It fails if a command fails, if `list` does not show the issue, or if the closed issue is not in `.zban/archive/`. Only `.github/workflows/release.yml` changed.
- `x86_64-apple-darwin` is cross-compiled on Apple silicon and still not run, as before.
- Checks: the build-only Release run on pull request #20 (https://github.com/convesoft/zban/actions/runs/43226629057) built and packaged all four targets, and the smoke step passed on `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl` and `aarch64-apple-darwin`. Locally: the extracted step passed against a release build and failed when `list --status done` printed nothing; `actionlint`, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `mara schema validate` and `mara project validate` all passed.
