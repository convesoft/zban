---
title: Distribute zban on npm and adopt @convesoft/nx-rust for releases
status: done
type: feature
priority: high
tags:
  - release
  - npm
discovered_from: zb-2fcnft
created_at: 2026-11-22T12:06:17Z
closed_at: 2026-11-22T12:10:38Z
---

Implement npm distribution and Nx release tooling as decided in `ADR-NPM-DISTRIBUTION` and `NEED-NPM-DISTRIBUTION` (from zb-2fcnft):

- Ship `@convesoft/zban` plus four platform packages (`@convesoft/zban-linux-x64`, `@convesoft/zban-linux-arm64`, `@convesoft/zban-darwin-arm64`, `@convesoft/zban-darwin-x64`) using `npm/zban.cjs` and `scripts/package-npm.mjs`.
- Adopt Nx with `@convesoft/nx-rust@1.0.0-rc.1` (`package.json`, `pnpm-lock.yaml`, `nx.json`) so `Cargo.toml` stays the single version source of truth (`pnpm nx release version` bumps `Cargo.toml` + `Cargo.lock`; `pnpm nx release changelog` creates the tag and GitHub Release).
- Update `.github/workflows/release.yml` for `workflow_dispatch` (`dist_tag`, `dry_run`), local pre-publish npm install smoke tests, ordered idempotent OIDC Trusted Publishing (`platform` packages → wait → `@convesoft/zban` → `npx` smoke test), and attaching the four `.tar.gz` archives + `SHA256SUMS` to the GitHub Release.
- Update `README.md` and `docs/mcp.mara.md` installation and release documentation.

## Tasks

- [x] Add Nx and `@convesoft/nx-rust@1.0.0-rc.1` workspace config (`package.json`, `pnpm-lock.yaml`, `nx.json`, `.gitignore`)
- [x] Add `npm/zban.cjs` dispatcher shim and `scripts/package-npm.mjs` packager
- [x] Update `.github/workflows/release.yml` for npm packaging, local smoke tests, OIDC Trusted Publishing and `nx release changelog`
- [x] Update `README.md` and `docs/mcp.mara.md` for npm installation and the updated release flow

## Done when

- [x] `pnpm nx show projects`, `pnpm nx release version --dry-run` and local `package-npm.mjs` + `npm pack` + local install smoke test succeed
- [x] `actionlint` accepts `.github/workflows/ci.yml` and `.github/workflows/release.yml`
- [x] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [x] `mara schema validate` and `mara project validate` report `valid: true` with complete evaluation
- [x] `zban check` reports no errors

## Summary of Changes

- Added Nx and `@convesoft/nx-rust@1.0.0-rc.1` workspace configuration (`package.json`, `pnpm-lock.yaml`, `pnpm-workspace.yaml`, `nx.json`, `.gitignore`), keeping `Cargo.toml` as the single version source of truth (`pnpm nx release version` bumps `Cargo.toml` + `Cargo.lock`; `pnpm nx release changelog` tags `vX.Y.Z` and creates the GitHub Release).
- Added `npm/zban.cjs` dispatcher shim and `scripts/package-npm.mjs` packager for `@convesoft/zban` and the four platform packages (`@convesoft/zban-linux-x64`, `@convesoft/zban-linux-arm64`, `@convesoft/zban-darwin-arm64`, `@convesoft/zban-darwin-x64`).
- Updated `.github/workflows/release.yml` for `workflow_dispatch` (`dist_tag`, `dry_run` defaulting to `true`), 4-target archive and npm `.tgz` packing, pre-publish local npm install smoke tests (`--ignore-scripts`), ordered idempotent OIDC Trusted Publishing (`platform` packages -> wait -> `@convesoft/zban` -> `npx` smoke test), `pnpm nx release changelog` tagging/GitHub Release creation, and uploading the four `.tar.gz` archives + `SHA256SUMS`.
- Updated `README.md` and `docs/mcp.mara.md` with npm/`npx` installation and the Nx release flow.
- Checks run:
  - `pnpm install --frozen-lockfile`, `NX_USE_LOCAL=true NX_DAEMON=false pnpm nx show projects` (`zban`), `pnpm nx run zban:fmt-check --output-style=static`, `pnpm nx release version --dry-run` (computed `0.1.0` -> `0.1.1` diff on `Cargo.toml`), `pnpm nx release changelog 0.2.0 --dry-run`.
  - Local `scripts/package-npm.mjs` + `npm pack` + `npm install --ignore-scripts` + `./node_modules/.bin/zban --version`, `init`, `check` smoke test passed.
  - `mise x actionlint@1.7.12 shellcheck@0.11.0 -- actionlint` passed with 0 errors and 0 warnings.
  - `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` passed.
  - `mara schema validate` and `mara project validate` reported `valid: true` with `evaluation_complete: true` and 0 errors/warnings.
  - `zban check` reported 0 findings.
