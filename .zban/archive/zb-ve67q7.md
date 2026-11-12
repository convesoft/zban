---
title: Decide how release archives ship third-party license notices
status: done
type: task
discovered_from: zb-zqn4z3
created_at: 2026-11-12T15:25:56Z
closed_at: 2026-11-12T16:44:19Z
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

- [x] Record the chosen option in `ADR-RUST-BINARY`
- [x] Ship the notices as decided, and list them with the archive contents in the README
- [x] Check that an archive from a build-only run of `release.yml` holds notices for every crate compiled into the binary

## Summary of Changes

The maintainer chose option 1 on 2026-11-12. After review they also confirmed accepting `Unicode-3.0` for `unicode-ident`, quoting `rmcp`'s and `saphyr-parser`'s license files verbatim, and deciding notices for the standard library, musl and libunwind separately in draft zb-e7vs4j.

- `ADR-RUST-BINARY` records the decision: each release archive also holds `THIRD-PARTY-LICENSES`, which the release workflow writes with `cargo-about` installed at a pinned version, from `about.toml` and `about.hbs`. A license outside the accepted list, or a quoted license file whose checksum changed, fails the release. `ADR-DEPENDENCIES` covers only runtime crates and is unchanged. The README's install section names the file.
- `release.yml`: the new step "Write the third-party licenses" installs `cargo-about` 0.9.3 with `cargo install --locked --features cli` and runs `cargo about generate --locked --fail --target <target>`. It fails when cargo-about logs a warning, because a checksum mismatch is only a warning that falls back to the generic license text. The Package step adds the file and checks that `tar -tzf` lists it. Pull requests that change `about.toml` or `about.hbs` now run the workflow. cargo-about is not cached: tag runs cannot restore caches from pull request runs, and the release job should not run an executable restored from a cache.
- `about.toml` accepts `MIT`, `Apache-2.0`, `Zlib` (`foldhash`) and `Unicode-3.0` (`unicode-ident`, a proc-macro dependency), preferring MIT, which is what `Cargo.lock` needs today. It leaves out zban itself and the dev-dependencies, and uses the built-in `chrono` workaround. It quotes, with checksums, `rmcp`'s LICENSE, which adds a relicensing note, an MIT text and a CC-BY-4.0 note for documentation, and `saphyr-parser`'s two authors' MIT files from `.licenses/`. `about.hbs` writes plain text, like the archive's other files.
- Checks: `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` (178 passed) pass on the merged head. `mara schema validate` and `mara project validate` report a valid project with complete evaluation, no errors and no warnings. `zban check` reports nothing, and `actionlint` 1.7.12 accepts the workflows. Locally, with cargo-about 0.9.3 installed into a session scratch directory, the step failed with a changed checksum and with `Unicode-3.0` removed from the accepted list.
- Release run 43260618786 on the pull request passed all four Build jobs. Each downloaded `zban-<target>.tar.gz` holds `zban`, `README.md`, `LICENSE-MIT`, `LICENSE-APACHE` and `THIRD-PARTY-LICENSES`. Each file named all 80 crates that `cargo tree -e normal,build --target <target>` lists, but also 5 crates that only the tests use (`tempfile` and its dependencies), which `main` had added meanwhile. Setting `ignore-dev-dependencies` removed them: generating locally for each of the four targets now gives exactly those 80 crates, with no warnings.
- Filed draft zb-e7vs4j: the notices do not cover Rust's standard library, or musl libc and libunwind, which the Linux binaries link statically.
- Rejected: HTML output, which needs a browser and escaping. Listing the targets in `about.toml`, which would duplicate the build matrix. Caching cargo-about, for the reasons above. Leaving `rmcp` and `saphyr-parser` to cargo-about's detection, which replaced their files with the generic MIT or Apache-2.0 text and lost the copyright lines.
