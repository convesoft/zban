---
title: Release workflow passes tarballs to npm publish as GitHub shorthand
status: in-progress
type: bug
priority: high
created_at: 2026-11-22T14:44:00Z
---

The first dry run of `.github/workflows/release.yml` for 1.0.0-rc.1 (run 45033672182, `dist_tag: next`, `dry_run: true`) failed in "Publish to npm (dry run)": npm 11.20.0 read `dist/convesoft-zban-linux-x64-1.0.0-rc.1.tgz` as the GitHub shorthand `owner/repo` and ran `git ls-remote ssh://git@github.com/dist/...` (Permission denied). A relative path given to `npm publish` must start with `./`. The same form is used by the dry-run step and both OIDC publish steps.

## Done when

- [ ] Every tarball path passed to `npm publish` in the publish job starts with `./`
- [ ] actionlint, fmt, clippy, tests, Mara validation and `zban check` pass
- [ ] The release dry run succeeds after merge (verified outside this pull request)
