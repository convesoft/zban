---
title: Bundle documentation in the @convesoft/zban npm package
status: done
type: feature
tags:
  - npm
  - docs
parent: zb-v90b0k
created_at: 2026-11-22T12:35:31Z
closed_at: 2026-11-22T12:58:06Z
---

Follow the same pattern adopted for `@convesoft/nx-rust` (`ADR-BUNDLED-DOCS`): ship zban's user and specification docs (`docs/*.mara.md`) inside the published `@convesoft/zban` npm package so coding agents and developers can read the full documentation directly from the installed package without searching for the GitHub repository.

Relevant items: `ADR-NPM-DISTRIBUTION`, `NEED-NPM-DISTRIBUTION`, `NEED-AGENT-ONBOARDING`.

## Tasks

- [x] Record the decision in Mara (`docs/decisions.mara.md` / `ADR-NPM-DISTRIBUTION`), noting which doc files are bundled in `@convesoft/zban` (not duplicated across the four platform binary packages) and any accepted trade-offs (e.g. links to `.zban/` outside the package, unpacked package size)
- [x] Update `scripts/package-npm.mjs` to copy `docs/*.mara.md` into `@convesoft/zban` and list `docs/` in its `files` field (and verify the packed tarball includes them in `.github/workflows/release.yml`)
- [x] Check whether `README.md` (bundled in the package) or a short root pointer doc should point agents to `docs/index.mara.md` inside the installed package
- [x] Verify local `scripts/package-npm.mjs main` + `npm pack` includes the docs cleanly and update tests/checks as needed

## Done when

- [x] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [x] `actionlint` accepts `.github/workflows/release.yml`
- [x] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [x] `zban check` reports no errors

## Summary of Changes

- `scripts/package-npm.mjs main` copies the whole `docs/` folder into `@convesoft/zban` and lists `docs/` in its `files`. The four platform packages and the release archives keep their file lists and carry the shared README with its new sentence.
- The `prepare` job of `.github/workflows/release.yml` now fails the pack step if the packed `@convesoft/zban` lacks `docs/index.mara.md` or any other file of `docs/`, with an `::error::` naming the missing path.
- `ADR-NPM-DISTRIBUTION` records the bundled pages, why only the main package carries them, and the accepted consequences; `README.md` says where the installed package keeps them.

Checks run: `mara schema validate` and `mara project validate` (`valid: true`, evaluation complete, no diagnostics); `mise x actionlint@1.7.12 shellcheck@0.11.0 -- actionlint` (clean); `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` (all pass); `zban check` (no errors). Packaging: the workflow's own pack step, run locally with `RUNNER_TEMP` under `$TMPDIR`, packed `convesoft-zban-0.1.0.tgz` (13 files; 46.3 kB packed and 133.3 kB unpacked when re-measured after the review fixes and the merge of `origin/main`) with all eight pages byte-identical to `docs/`; the same check fails on a package staged without `docs/`; a platform package packed from a stand-in binary holds no `docs/`.

## Decisions made

- The whole `docs/` folder is copied, not a fixed list or only `*.mara.md`, so a new page ships without a script change (today the folder holds only the eight `*.mara.md` pages). Recorded in `ADR-NPM-DISTRIBUTION`.
- No separate pointer file at the package root: the README's existing links into `docs/` resolve inside the installed package, so one README sentence points there. nx-rust's `PLUGIN.md` follows an Nx convention that has no counterpart for zban. The agent guide is unchanged. Recorded in `ADR-NPM-DISTRIBUTION`.
- Release archives keep their contents (`ADR-RUST-BINARY`); their README and the platform packages' README link into a `docs/` they lack, which is accepted. Recorded in `ADR-NPM-DISTRIBUTION`.
- `docs/**` is not added to the release workflow's pull-request paths: packing does not depend on the pages' content, and every release and packaging change runs the check. Recorded in `ADR-NPM-DISTRIBUTION`.

## Review fixes

- The pack step's check now lists every entry of `docs/` that is not a directory (`find docs ! -type d`) instead of only regular files. A symlinked page is copied into the staged package as a symlink and `npm pack` leaves it out entirely, so the check now fails the step and names the page instead of missing it. Fixture: a disposable copy under `$TMPDIR` with `docs/linked.mara.md` symlinked to `index.mara.md` failed with `::error::package/docs/linked.mara.md is missing from @convesoft/zban` (tarball: 13 files, no `linked` entry); the real `docs/` passes.
- `ADR-NPM-DISTRIBUTION` now says why there is no pointer file like nx-rust's `PLUGIN.md`, and records the package size re-measured after this round's edits and the merge of `origin/main`: about 133 kB unpacked, 46 kB packed.
- Merged `origin/main` (zb-a76tqw, zb-banzx5 and tracker commits) without conflicts. Reran `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `mara schema validate`, `mara project validate --format json` (`valid: true`, `evaluation_complete: true`), `zban check`, `mise x actionlint@1.7.12 shellcheck@0.11.0 -- actionlint` and the local packaging check (13 files, 46.3 kB packed, 133.3 kB unpacked, pages and README byte-identical to the repository's).
