---
title: Bump the version to 1.0.0-rc.1
status: done
type: task
related:
  - zb-r2nnaz
created_at: 2026-11-22T14:26:02Z
closed_at: 2026-11-22T14:26:08Z
---

The maintainer decided on 2026-11-22 to ship milestone zb-v90b0k as the release candidate `1.0.0-rc.1`, under the npm dist-tag `next` (`ADR-NPM-DISTRIBUTION`, zb-r2nnaz). A release is cut from `main` after a release pull request bumps `Cargo.toml` and `Cargo.lock`; the release workflow then reads the version from `Cargo.toml`.

## Tasks

- [x] Run `pnpm install` and `pnpm nx release version 1.0.0-rc.1`, and commit the changed `Cargo.toml` and `Cargo.lock`

## Done when

- [x] `Cargo.toml` and `Cargo.lock` give zban the version `1.0.0-rc.1` and nothing else changed in them
- [x] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [x] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [x] `zban check` reports no errors

## Summary of Changes

`pnpm nx release version 1.0.0-rc.1` changed the version of zban from `0.1.0` to `1.0.0-rc.1` in `Cargo.toml` and in `Cargo.lock` (one line each), and nothing else.

Checks run: `cargo fmt --all -- --check` (pass), `cargo clippy --all-targets --locked -- -D warnings` (pass), `cargo test --locked` (188 passed, 0 failed), `mara schema validate` (valid), `mara project validate --format json` (`valid: true`, `evaluation_complete: true`), `zban check` (no problems), and the built binary prints `zban 1.0.0-rc.1`. CI on the pull request covers the tests on the other platforms and the release workflow's build, pack and smoke-test jobs.

Not part of this issue: dispatching the release workflow, which publishes the packages, creates the tag `v1.0.0-rc.1` and the GitHub Release.
