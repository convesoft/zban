---
title: Use one pinned macOS runner for CI and releases
status: done
type: task
created_at: 2026-11-12T17:10:54Z
closed_at: 2026-11-12T17:35:59Z
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
- [x] The x86_64 smoke test under Rosetta 2 is either working or dropped with the reason recorded here
- [x] `actionlint` accepts both workflows
- [x] A build-only Release run on the pull request passes and yields all four archives with the same files, and CI passes


## Notes

- `ADR-RUST-BINARY` names the four targets, the test suite on each platform whose binary the workflow can run, and `cargo-about` on each build, but no runner and no job layout, so it stays unchanged. No `VER-` item covers the workflows.


## Summary of Changes

- `ci.yml`: the macOS test job runs on `macos-15`, the image `release.yml` builds on, instead of `macos-latest`. The check is now named "Test (macos-15)"; `main` has no branch protection or rulesets that name the old one.
- `release.yml`: each matrix entry lists its `targets`, the first being the target its runner runs natively, and the job is named after them. The Linux entries keep their runner and their one target. The two macOS entries became one `macos-15` entry building `aarch64-apple-darwin` and `x86_64-apple-darwin`: one toolchain with both targets, both built one after the other, the Test step (`cargo test --locked --target ${{ matrix.targets[0] }}`, keeping the `--target` behaviour from zb-c83nw7) for `aarch64-apple-darwin`, the smoke test for both, `cargo-about` installed once with `THIRD-PARTY-LICENSES` written per target into its staging directory, and the same archives with the same files and `tar -tzf` check. Each job uploads one artifact named after its runner, and the release job's `zban-*` download still gives the four archives for `SHA256SUMS`.
- A list of targets per entry was chosen over a separate macOS job, which would have repeated the tag checks, smoke test, license and package steps and needed its own place in the release job's `needs`.
- Rosetta 2: the `macos-15` arm64 image installs it (`install-rosetta.sh` in actions/runner-images), so the x86_64 binary is smoke-tested directly. If a runner lacks it, the step tries `sudo softwareupdate --install-rosetta --agree-to-license` and otherwise skips that binary with a warning instead of failing the release. That fallback has not run, since the image has Rosetta.
- `ADR-RUST-BINARY` names no runner or job layout and is unchanged.
- Checks: `actionlint` 1.7.12 on both workflows; `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked` (178 passed) and `cargo test --locked --target x86_64-unknown-linux-musl` (178 passed) locally after merging zb-c83nw7; `mara schema validate`, `mara project validate` and `zban check`. Locally, the Build, Smoke-test and Package step scripts ran for the Linux host target. On the pull request's head 9792a71, Release run 43269793938 passed all three Build jobs: the single macOS job "Build (aarch64-apple-darwin, x86_64-apple-darwin)" smoke-tested both binaries, the x86_64 one under Rosetta 2 (`zban 0.1.0`, then `Created .../smoke-x86_64-apple-darwin/.zban`) with no "Rosetta 2 is unavailable" warning, and packaged both archives with `THIRD-PARTY-LICENSES`. CI passed, including "Test (macos-15)". The final check, CI and the Release run on this pull request's head after closing this issue, is covered by CI on the pull request.
