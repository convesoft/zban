---
title: Read and write issue files
status: done
type: feature
priority: high
parent: zb-9jra34
blocked_by:
  - zb-naex76
created_at: 2026-11-11T21:00:02Z
closed_at: 2026-11-12T08:16:27Z
---

Implements `DES-PROJECT-LAYOUT` (including `init`), `DES-ISSUE-ID` and `DES-ISSUE-FILE`,
with the YAML crate chosen per `ADR-DEPENDENCIES`.

## Done when

- [x] `VER-ISSUE-FILES` passes, using a copy of this repository's `.zban/` as a fixture
- [x] `VER-ID-GENERATION` passes

## Summary of Changes

- Core library modules per `DES-SHARED-CORE`: `id` (prefix and ID rules, IDs from `getrandom`), `config` (`config.toml`), `timestamp`, `issue` (model, vocabulary, parsing into an issue or per-file `DES-CHECK` problem codes, validation, canonical rendering), `yaml` (the header layer on `saphyr-parser`) and `store` (`Project`: `init`, discovery, load, create, replace).
- Writes go through a fully written, synced `.zban-<random>.tmp` file in the target directory. Replace reads the target again, fails with `conflict` if it changed, then renames. Create hard-links the temporary file to `<id>.md`, which fails instead of replacing, and draws again when the ID exists in either directory. Every write first checks that the rendered file parses back to the same issue.
- The writer quotes a string only when parsing its plain form back with `saphyr-parser`, plus the YAML 1.2 core schema, does not give the same string.
- Reading choices within `DES-ISSUE-FILE`: plain scalars in known keys are taken as written (`title: 2027` is the title "2027"), null values count as absent, and IDs in headers match case-insensitively and are written lowercase. Every explicit YAML tag is rejected, as `ADR-DEPENDENCIES` says, and the header must be a block mapping. An unknown entry runs from its key to the end of its value, so block scalars keep their trailing lines.
- The binary has `zban init [--prefix P]` and the global `-C, --directory DIR` option, with human output only. `--json` and the error object come with zb-gnt4g4.
- Knowledge: `ADR-DEPENDENCIES` now names `saphyr-parser` and says `config.toml` is read without a TOML crate. The command line page and the README say that only `zban init` works so far.
- Tests: `tests/issue_files.rs` for `VER-ISSUE-FILES`, including a copy of this repository's `.zban/` written back byte-identical (it fails if a hand-edited backlog file is not in canonical form); the interrupted-write test in `src/store.rs`; `tests/ids.rs` for `VER-ID-GENERATION`, including `zban init` through the binary and the collision analysis (32^6 = 2^30 IDs, about 0.093% for 1000 issues on each of two branches); `tests/project_layout.rs` for `DES-PROJECT-LAYOUT`.
- Coverage of "rejected by create and update and reported by check" in `VER-ISSUE-FILES` is at the library level: vocabulary parsing fails with `invalid_value`, `Project::create` and `Project::replace` reject invalid values without writing, and loading reports `invalid_value` for the file. The CLI, MCP and `zban check` surfaces of that clause belong to `VER-CLI` (zb-gnt4g4) and `VER-CHECK` (zb-yj8xta).
- Filed zb-hqez4b (draft) for temporary files left by interrupted writes.

Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (47 passed: 22 unit, 3 `cli`, 5 `ids`, 11 `issue_files`, 6 `project_layout`), `mara schema validate` and `mara project validate` (`valid: true`, evaluation complete).
