---
title: Decide how release archives ship third-party license notices
status: draft
type: task
discovered_from: zb-zqn4z3
created_at: 2026-11-12T15:25:56Z
---

Found while hardening the release workflow (zb-zqn4z3). The release archives of `ADR-RUST-BINARY` hold the statically linked `zban` binary, `README.md` and zban's own `LICENSE-MIT` and `LICENSE-APACHE`, but no notices for the third-party crates compiled into the binary. For `x86_64-unknown-linux-musl`, `Cargo.lock` resolves about 80 normal dependencies, some of them proc macros that do not end up in the binary. Most are `MIT OR Apache-2.0`; 13 are `MIT` only, `rmcp` is `Apache-2.0` only, `foldhash` is `Zlib` and `memchr` is `Unlicense OR MIT`. MIT asks that its notice be included in all copies of the software, and Apache-2.0 asks anyone who redistributes it in object form to give recipients a copy of the license and the text of any `NOTICE` file. The v0.1.0 archives already shipped without these notices.

This needs a maintainer decision before any work: option 1 adds a tool that neither `ADR-RUST-BINARY` nor `ADR-DEPENDENCIES` names, and every option changes what a release ships.

## Options

1. **cargo-about in the release workflow only.** The release job's runners install a pinned `cargo-about` with `cargo install --locked` and generate one `THIRD-PARTY-LICENSES` file from `Cargo.lock` into each archive, from an `about.toml` that lists the accepted licenses and a short template in the repository. Notices follow every dependency update with no manual step, and the accepted list fails the release when a dependency brings an unexpected license. It is not a runtime or development dependency and contributors never run it, but it adds a build tool, two configuration files and about a minute of install time per release build. A prebuilt binary through a third-party installer action would be faster, but the v0.1.0 release (zb-aapz52) chose to avoid third-party release actions.
2. **A hand-maintained notices file.** A committed `THIRD-PARTY-LICENSES` that the Package step copies into each archive. It needs no tool, but someone has to update about 80 crates' copyright lines and license texts on every `Cargo.lock` change, nothing checks that they did, and it will drift.
3. **Rely on `Cargo.lock` and `cargo tree`.** Ship nothing more, and point the README at the source tree, where `Cargo.lock` names each dependency and its crate carries its license. This adds no work, but the archives would still not carry the notices that MIT and Apache-2.0 ask binary redistributors to include.

## Proposed answer

Option 1. It is the only option that ships correct notices without manual upkeep, and keeping it in `release.yml` leaves the code, the development checks and `GOAL-SMALL-TOOL` untouched. Record it in `ADR-RUST-BINARY`, next to the release steps, as a release-only tool, with a pinned version installed through `cargo install --locked`.

## Tasks once decided

- [ ] Record the chosen option in `ADR-RUST-BINARY`
- [ ] Ship the notices as decided, and list them with the archive contents in the README
- [ ] Check that an archive from a build-only run of `release.yml` holds notices for every crate compiled into the binary
