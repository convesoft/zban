---
title: Use one pinned macOS runner for CI and releases
status: in-progress
type: task
created_at: 2026-11-12T17:10:54Z
---

macOS jobs keep being cancelled before they start. GitHub annotates them with "The job was not acquired by Runner of type hosted even after multiple attempts … Due to capacity constraints, jobs targeting macOS arm64 runners may experience longer queue times." CI's test job uses `macos-latest` (currently `macos-26` on arm64), while `.github/workflows/release.yml` pins `macos-15` and asks for two macOS runners per run: one builds `aarch64-apple-darwin` natively, the other cross-compiles `x86_64-apple-darwin`. Every extra macOS job is another chance to wait in that queue.

The maintainer decided:

1. CI's macOS test job in `.github/workflows/ci.yml` runs on `macos-15`, the image the release builds on.
2. `release.yml` builds both macOS targets of `ADR-RUST-BINARY` in one job on one `macos-15` runner: one toolchain with both targets, the test suite and the smoke test for the native `aarch64-apple-darwin` (keeping the Test step's `--target` behaviour), both targets built one after the other, `cargo-about` installed once with `THIRD-PARTY-LICENSES` written per target, and the same two archives with the same files and `tar -tzf` check. The Linux entries stay as they are, and the release job still gets the same four archives plus `SHA256SUMS`.
3. Best effort: smoke-test `x86_64-apple-darwin` in that job under Rosetta 2. If the image lacks Rosetta, try `softwareupdate --install-rosetta --agree-to-license`; if that is impossible or flaky, drop the x86_64 smoke test and say so rather than fail the job.

## Done when

- [x] `ADR-RUST-BINARY` checked, and updated if it names the runners or describes one job per target
- [x] CI's macOS test job runs on `macos-15`
- [x] `release.yml` builds, tests, smoke-tests and packages both macOS targets in one `macos-15` job, with the Linux entries unchanged
- [ ] The x86_64 smoke test under Rosetta 2 is either working or dropped with the reason recorded here
- [x] `actionlint` accepts both workflows
- [ ] A build-only Release run on the pull request passes and yields all four archives with the same files, and CI passes


## Notes

- `ADR-RUST-BINARY` names the four targets, the test suite on each platform whose binary the workflow can run, and `cargo-about` on each build, but no runner and no job layout, so it stays unchanged. No `VER-` item covers the workflows.
