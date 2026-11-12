---
title: Release v0.1.0
status: done
type: task
parent: zb-9jra34
blocked_by:
  - zb-yj8xta
  - zb-bbyjed
  - zb-as3s58
  - zb-qkj8tj
  - zb-0psza8
  - zb-9d823t
  - zb-bf3yyh
  - zb-ghc7nn
  - zb-ec346b
  - zb-97k0ne
  - zb-71kr2e
  - zb-98enq3
  - zb-906f27
created_at: 2026-11-11T21:00:10Z
closed_at: 2026-11-12T14:43:31Z
---

Distribution and licensing follow `ADR-RUST-BINARY`.

Maintainer decision (2026-11-12): zban is distributed only through GitHub Releases, as prebuilt binaries and as source builds with `cargo install --git ... --tag`, and is not published to crates.io. The items "Confirm the `zban` crate name is available" and "Publish to crates.io" were dropped. npm distribution needs its own decision first (zb-2fcnft).

- [x] Choose release target triples for the platforms in `ADR-RUST-BINARY`
- [x] Add `LICENSE-MIT` and `LICENSE-APACHE`
- [x] Release workflow builds the binaries for every target, checked by a build-only run on the pull request
- [x] The `v0.1.0` tag creates the GitHub release with the binaries and `SHA256SUMS` attached, and the README install commands work against it
- [x] README installation instructions match the release
- [x] `VAL-SCOPE-REVIEW` is done and its shortfalls are filed

## Scope review (`VAL-SCOPE-REVIEW`)

- Every requirement and design is accepted and traces through `satisfies` to an accepted need. `mara project validate` passes the acceptance policies.
- Since the v1 specification (#1), no command or option was added to `DES-CLI`, and `DES-ISSUE-FILE` is unchanged. The `DES-CLI` edits only clarified the order of body edits, absolute `path` values and the `invalid_handoff` cases.
- Shortfall: `zban list` accepts `-s`, `-t` and `-p`, which `DES-CLI` does not list. Filed as zb-x481ey.
- `zban prime` prints 127 lines and has every section of `DES-AGENT-GUIDE`. The `VER-AGENT-GUIDE` tests check the operations it names against clap and the MCP tool schemas.
- `VAL-DOGFOOD` cannot be judged yet: four issues have gone through zban since zb-as3s58, short of ten. zb-hr38j4 tracks it.
- Shortfall: `AGENTS.md` still says implementation is in progress. Added to zb-mq3eqv.

## Summary of Changes

- Released zban v0.1.0. Pull request #16 added `.github/workflows/release.yml`, `LICENSE-MIT`, `LICENSE-APACHE`, `publish = false` in `Cargo.toml` and the README installation section, and recorded distribution through GitHub Releases only, on four targets, in `ADR-RUST-BINARY`. The maintainer pushed tag `v0.1.0` on `main` at `9169669`.
- Release run https://github.com/convesoft/zban/actions/runs/43239456808 succeeded: all four Build jobs, whose smoke test printed `zban 0.1.0` on x86_64 and aarch64 Linux and on aarch64 macOS, then "Create the GitHub release".
- https://github.com/convesoft/zban/releases/tag/v0.1.0 is published, neither draft nor prerelease, with generated notes, `SHA256SUMS` and the four archives named in the README table.
- Checks run on x86_64 Linux on 2026-11-12:
  - `gh release view v0.1.0` confirmed the release, its assets and their digests.
  - The README's prebuilt binary commands, run as written with `target=x86_64-unknown-linux-musl` and a scratch `HOME`: both `releases/latest/download/` URLs resolved, `sha256sum -c --ignore-missing SHA256SUMS` reported OK, and `zban --version` printed `zban 0.1.0`. The binary is a static-pie x86-64 ELF.
  - All four archives fetched from `releases/latest/download/` match `SHA256SUMS` and the checksums the release job printed, and each holds `zban`, `README.md`, `LICENSE-MIT` and `LICENSE-APACHE`. The aarch64 Linux binary is statically linked and printed `zban 0.1.0` under qemu-aarch64. The macOS binaries are Mach-O x86_64 and arm64 executables.
  - `cargo install --locked --git https://github.com/convesoft/zban --tag v0.1.0`, into a scratch `--root`, built `9169669` with rustc 1.99.1 and installed a `zban` that printed `zban 0.1.0`.
  - All thirteen blockers are done.
- Not checked: the README commands on a Mac, and the x86_64-apple-darwin binary on an Intel Mac or under Rosetta. CI only cross-compiles that binary.
- Rejected: glibc binaries on Linux, because they would have to be built against the oldest glibc to support. Intel macOS runners, which GitHub is phasing out, because Apple silicon cross-compiles x86_64 with no extra setup. Versions in archive names, because they would break the `releases/latest/download/` URLs. Third-party release actions, because `gh release create` on the runner is enough.
