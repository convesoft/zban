---
title: Harden the release workflow
status: done
type: task
priority: low
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
closed_at: 2026-11-12T15:26:49Z
---

Found by the v0.1.0 review (`ADR-RUST-BINARY`): a tag push runs no tests and does not require the tagged commit to be on `main`, and the archives carry no notices for the statically linked third-party crates.

- [x] Run the test suite before creating a release
- [x] Refuse a tag whose commit is not on `main`
- Shipping third-party license notices needs a maintainer decision first, because a tool such as cargo-about would be new. It moved to the draft zb-ve67q7, which lays out the options with a proposed answer.

## Summary of Changes

- `release.yml`: each Build job checks out the full history. On a tag push, the new step "Check that the tagged commit is on main" fetches `origin/main` and fails unless the checked-out commit is an ancestor of it, before anything is built. The new Test step runs `cargo test --locked` on the three runners that build natively: x86_64 and aarch64 Linux and aarch64 macOS, whose runner also cross-compiles x86_64 macOS. The release job still runs only on a pushed `v*` tag and needs every Build job, so it creates no release unless the tag checks and the tests pass. Pull request and manual runs now build and test.
- `ADR-RUST-BINARY` now states the release steps: a tag `vX.Y.Z` that names the `Cargo.toml` version on a commit of `main`, and the test suite run on the tagged commit before the release is created. The README's Development section says the same.
- Third-party license notices moved to the draft zb-ve67q7.
- Running the suite for the musl release targets showed that `store::tests::a_write_proceeds_unlocked_only_where_locking_is_unsupported` fails there, because musl's message for error 95 is "Not supported". Filed as zb-c83nw7. Until it is fixed, the Test step tests the runner's own target rather than the musl target the release ships.
- Checks run on x86_64 Linux on 2026-11-12: `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` pass. `mara schema validate` and `mara project validate` report a valid project with complete evaluation and no diagnostics. `zban check` reports no findings. `actionlint` 1.7.12 accepts both workflows. The ancestry check's script, run in a scratch clone, passed for a tag on `main` and failed with its error annotation for a lightweight and an annotated tag on a side branch; `v0.1.0` passes it. `cargo test --release --locked --target x86_64-unknown-linux-musl` fails only in the test of zb-c83nw7.
- Not checked: a real tag push, because both tag checks run only on tags and this work pushed none. The build-only run on the pull request runs the Test step on all three native runners.
- Rejected: testing the shipped musl targets now, which fails until zb-c83nw7 is fixed. A separate test job, because a step in each Build job already gates the release job. Checking ancestry in the release job, which would refuse a tag only after all four builds.
