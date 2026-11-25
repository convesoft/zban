---
title: Migrate repository to @convesoft/mara 0.4.0-rc.1 and validate docs in CI
status: done
type: task
created_at: 2026-11-25T10:07:48Z
closed_at: 2026-11-25T10:10:57Z
---

Add `@convesoft/mara@0.4.0-rc.1` as a dev dependency, convert the repository's Mara schema, configuration and `docs/*.mara.md` files from the 0.3.0 / `:::mara` format to the 0.4.0-rc.1 format, carry over the engineering template Soufflé rules and Rust code-link grammar, remove legacy 0.3.0 `.mara/` files, update docs/agent instructions, and run `pnpm mara` docs validation in CI.

## Tasks

- [x] Add `@convesoft/mara` `0.4.0-rc.1` (exact) as a dev dependency and update `pnpm-workspace.yaml` / `pnpm-lock.yaml`
- [x] Run `pnpm mara project convert` and resolve any items reported in `.mara/conversion-report.json`
- [x] Carry over the 0.4.0-rc.1 engineering template Soufflé rules (`.mara/rules/*.dl`) and `[rules]` / `[invariants]` / `[queries]` / `[views]` / `[gates]` config
- [x] Configure `[code.languages.rust]` (`src/**/*.rs`, `tests/**/*.rs`) and pin the Rust grammar with `pnpm mara code add rust`
- [x] Remove legacy 0.3.0 `.mara/` files and clean up `.gitignore`
- [x] Update docs and agent instructions (`AGENTS.md`, `README.md`, `docs/index.mara.md`, `.mara/README.md`, `.github/pull_request_template.md`) for Mara 0.4.0-rc.1 and `pnpm mara`
- [x] Add Mara docs validation job to `.github/workflows/ci.yml` using `pnpm mara` from dev dependencies

## Done when

- [x] `pnpm mara --no-daemon project validate` passes with 0 errors and complete rule/code evaluation
- [x] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` pass
- [x] `zban check` passes
- [x] CI passes on the pull request, including the new Mara docs validation job

## Summary of Changes

- Added `@convesoft/mara` (`0.4.0-rc.1`, exact) to `devDependencies` in `package.json`, added `@convesoft/mara@0.4.0-rc.1` and its Linux platform packages to `minimumReleaseAgeExclude` in `pnpm-workspace.yaml`, and updated `pnpm-lock.yaml`.
- Converted `.mara/schema.toml`, `.mara/config.toml`, and `docs/*.mara.md` with `pnpm mara --no-daemon project convert`, carried over the 0.4.0-rc.1 engineering template Soufflé rules (`.mara/rules/knowledge.dl`, `readiness.dl`, `risk-matrix.dl`) and their `[rules]`/`[queries]`/`[views]`/`[invariants]`/`[gates]` configuration, configured `[code.languages.rust]` for `src/**/*.rs` and `tests/**/*.rs`, and pinned the Rust tree-sitter grammar (`0.24.2`) via `pnpm mara --no-daemon code add rust`.
- Removed the legacy 0.3.0 `.mara/` files (`project.toml`, `schema.yaml`, `engineering-*`, `.mara/code/`) and `/.mara/mutation.lock` from `.gitignore`.
- Updated `AGENTS.md`, `README.md`, `docs/index.mara.md`, `.mara/README.md`, and `.github/pull_request_template.md` for Mara 0.4.0-rc.1 and `pnpm mara`.
- Added a `Docs validation` job to `.github/workflows/ci.yml` that installs dev dependencies with pnpm, fetches the pinned Rust grammar, and runs `pnpm mara --no-daemon schema validate` and `pnpm mara --no-daemon project validate`.

Checks run:
- `pnpm mara --no-daemon schema validate` (`valid: 0 errors, 0 warnings`)
- `pnpm mara --no-daemon --format json project validate` (`valid: true`, 0 errors, 0 warnings)
- `zban check` (no problems found)
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- CI on the pull request covers the CI workflow check.
