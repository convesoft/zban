---
title: Release v0.1.0
status: in-progress
type: task
parent: zb-9jra34
blocked_by:
  - zb-yj8xta
  - zb-bbyjed
  - zb-as3s58
  - zb-qkj8tj
created_at: 2026-11-11T21:00:10Z
---

Distribution and licensing follow `ADR-RUST-BINARY`.

Maintainer decision (2026-11-12): zban is distributed only through GitHub Releases, as prebuilt binaries and as source builds with `cargo install --git ... --tag`, and is not published to crates.io. The items "Confirm the `zban` crate name is available" and "Publish to crates.io" were dropped. npm distribution needs its own decision first (zb-2fcnft).

- [x] Choose release target triples for the platforms in `ADR-RUST-BINARY`
- [x] Add `LICENSE-MIT` and `LICENSE-APACHE`
- [x] Release workflow builds the binaries for every target, checked by a build-only run on the pull request
- [ ] The `v0.1.0` tag creates the GitHub release with the binaries and `SHA256SUMS` attached, and the README install commands work against it
- [x] README installation instructions match the release
- [x] `VAL-SCOPE-REVIEW` is done and its shortfalls are filed

## Scope review (`VAL-SCOPE-REVIEW`)

- Every requirement and design is accepted and traces through `satisfies` to an accepted need. `mara project validate` passes the acceptance policies.
- Since the v1 specification (#1), no command or option was added to `DES-CLI`, and `DES-ISSUE-FILE` is unchanged. The `DES-CLI` edits only clarified the order of body edits, absolute `path` values and the `invalid_handoff` cases.
- Shortfall: `zban list` accepts `-s`, `-t` and `-p`, which `DES-CLI` does not list. Filed as zb-x481ey.
- `zban prime` prints 127 lines and has every section of `DES-AGENT-GUIDE`. The `VER-AGENT-GUIDE` tests check the operations it names against clap and the MCP tool schemas.
- `VAL-DOGFOOD` cannot be judged yet: four issues have gone through zban since zb-as3s58, short of ten. zb-hr38j4 tracks it.
- Shortfall: `AGENTS.md` still says implementation is in progress. Added to zb-mq3eqv.

## Handoff (2026-11-12T11:18:12Z)

- Work is on branch `zb-aapz52`, pull request https://github.com/convesoft/zban/pull/16, committed with a clean working tree. It is to be squash-merged, so after the merge the work is the merge commit on `main`.
- Done: `ADR-RUST-BINARY` records distribution through GitHub Releases only and the four release targets (musl on Linux). `Cargo.toml` sets `publish = false`. `LICENSE-MIT` and `LICENSE-APACHE` are added, `.github/workflows/release.yml` is written, and the README has installation instructions. The `VAL-SCOPE-REVIEW` is done, with its shortfalls in zb-x481ey and zb-mq3eqv, and zb-2fcnft is the npm follow-up. The build-only Release run on the pull request (https://github.com/convesoft/zban/actions/runs/43207678414) built and packaged all four targets. The Linux binaries are statically linked, and each native build printed `zban 0.1.0`.
- Next, for the maintainer, after merging the pull request:
  - No version bump is needed: `Cargo.toml` already has `version = "0.1.0"`. The workflow fails on a tag other than `v` plus that version.
  - Tag the merge commit and push the tag: `git switch main && git pull && git tag -a v0.1.0 -m "zban v0.1.0" && git push origin v0.1.0`.
  - Watch the Release workflow run for the tag: four Build jobs, then "Create the GitHub release".
- Verify afterwards:
  - Release `v0.1.0` exists, is not a draft, has generated notes, and has `SHA256SUMS` and these archives attached: `zban-x86_64-unknown-linux-musl.tar.gz`, `zban-aarch64-unknown-linux-musl.tar.gz`, `zban-x86_64-apple-darwin.tar.gz` and `zban-aarch64-apple-darwin.tar.gz`.
  - The README's install commands work against the release on Linux and macOS: the `releases/latest/download/` URLs resolve, the checksum check reports OK, and `zban --version` prints `zban 0.1.0`.
  - `cargo install --locked --git https://github.com/convesoft/zban --tag v0.1.0` installs zban.
  - If possible, run the `x86_64-apple-darwin` binary on an Intel Mac or under Rosetta. CI only cross-compiles it.
  - Then tick the last checklist item, replace this note with a `## Summary of Changes` and close zb-aapz52 with `zban update zb-aapz52 --no-handoff --append - --status done`.
- If the release job fails: fix the problem in a pull request, remove the tag and any partial release with `gh release delete v0.1.0 --cleanup-tag`, and push the tag again on the fixed commit.
- Uncertain: the agent wrote the reason that `ADR-RUST-BINARY` gives for having no crates.io crate, so the maintainer may want to reword it. The README's dual-licensing clause for contributions is the usual Rust wording and can be dropped.
- Rejected: glibc binaries on Linux, because they would have to be built against the oldest glibc to support. Intel macOS runners, which GitHub is phasing out, because Apple silicon cross-compiles x86_64 with no extra setup. Versions in archive names, because they would break the `releases/latest/download/` URLs. Third-party release actions, because `gh release create` on the runner is enough.
- Files touched: `docs/decisions.mara.md`, `Cargo.toml`, `LICENSE-MIT`, `LICENSE-APACHE`, `.github/workflows/release.yml`, `README.md`, `.zban/zb-aapz52.md`, `.zban/zb-mq3eqv.md`, and the new `.zban/zb-2fcnft.md` and `.zban/zb-x481ey.md`.
