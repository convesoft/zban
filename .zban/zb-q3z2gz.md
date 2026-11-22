---
title: Bundle documentation in the @convesoft/zban npm package
status: todo
type: feature
tags:
  - npm
  - docs
parent: zb-v90b0k
created_at: 2026-11-22T12:35:31Z
---

Follow the same pattern adopted for `@convesoft/nx-rust` (`ADR-BUNDLED-DOCS`): ship zban's user and specification docs (`docs/*.mara.md`) inside the published `@convesoft/zban` npm package so coding agents and developers can read the full documentation directly from the installed package without searching for the GitHub repository.

Relevant items: `ADR-NPM-DISTRIBUTION`, `NEED-NPM-DISTRIBUTION`, `NEED-AGENT-ONBOARDING`.

## Tasks

- [ ] Record the decision in Mara (`docs/decisions.mara.md` / `ADR-NPM-DISTRIBUTION`), noting which doc files are bundled in `@convesoft/zban` (not duplicated across the four platform binary packages) and any accepted trade-offs (e.g. links to `.zban/` outside the package, unpacked package size)
- [ ] Update `scripts/package-npm.mjs` to copy `docs/*.mara.md` into `@convesoft/zban` and list `docs/` in its `files` field (and verify the packed tarball includes them in `.github/workflows/release.yml`)
- [ ] Check whether `README.md` (bundled in the package) or a short root pointer doc should point agents to `docs/index.mara.md` inside the installed package
- [ ] Verify local `scripts/package-npm.mjs main` + `npm pack` includes the docs cleanly and update tests/checks as needed

## Done when

- [ ] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [ ] `actionlint` accepts `.github/workflows/release.yml`
- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [ ] `zban check` reports no errors
