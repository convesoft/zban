---
title: Release workflow passes tarballs to npm publish as GitHub shorthand
status: done
type: bug
priority: high
created_at: 2026-11-22T14:44:00Z
closed_at: 2026-11-22T14:44:45Z
---

The first dry run of `.github/workflows/release.yml` for 1.0.0-rc.1 (run 45033672182, `dist_tag: next`, `dry_run: true`) failed in "Publish to npm (dry run)": npm 11.20.0 read `dist/convesoft-zban-linux-x64-1.0.0-rc.1.tgz` as the GitHub shorthand `owner/repo` and ran `git ls-remote ssh://git@github.com/dist/...` (Permission denied). A relative path given to `npm publish` must start with `./`. The same form is used by the dry-run step and both OIDC publish steps.

## Done when

- [x] Every tarball path passed to `npm publish` in the publish job starts with `./`
- [x] actionlint, fmt, clippy, tests, Mara validation and `zban check` pass
- [x] The release dry run is dispatched after merge (outside this pull request; not run here by instruction)

## Summary of Changes

The three `npm publish` tarball arguments in the publish job of `.github/workflows/release.yml` now start with `./` (dry-run loop, `publish_or_verify` call for the platform packages, `tarball` for the main package); `sha1sum` reads the same paths. Checks run: actionlint 1.7.12 with shellcheck 0.11.0, `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `mara schema validate`, `mara project validate` (valid, evaluation complete), `zban check`. `npm publish` was not run; the workflow's dry run after merge is the proof. ADR-NPM-DISTRIBUTION does not describe the commands, so Mara is unchanged.
