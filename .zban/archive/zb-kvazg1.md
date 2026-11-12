---
title: Read indented headers with unknown block scalars
status: done
type: bug
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
closed_at: 2026-11-12T14:10:59Z
---

Found by the v0.1.0 review: a header indented by two spaces with an unknown `notes: |` key before `status:` passes `check`, but every update fails with `invalid_value` ("does not read back unchanged"). The read-back guard prevents damage. Likely cause: the entry's end line in `src/yaml.rs` is taken from the scalar's end marker, which lands on the next key (`DES-ISSUE-FILE`, `REQ-DAMAGED-DATA`).

- [x] Reproduce in a `VER-ISSUE-FILES` test, then fix the entry boundaries

## Summary of Changes

- `src/yaml.rs`: a block scalar's end marker sits after the indentation of the first line beyond it, so in an indented header it landed on the next key's line and that key became part of the unknown entry. An entry whose content ends after nothing but spaces now ends on the line above, which generalises the old column-0 rule. The read-back guard in `encode` is unchanged.
- `tests/issue_files.rs`: new `VER-ISSUE-FILES` test `unknown_block_scalars_survive_an_update_in_indented_headers` runs `zban check` and `zban update ID --status in-progress` through the real binary on flush-left and two-space-indented headers with literal, folded, `|-`, `|+`, empty and nested block scalars, a block scalar as the last entry, and one followed by flow and plain unknown values. Each unknown entry is written back unchanged, without the header's indentation. Before the fix, 7 of the 8 indented cases failed with `invalid_value`; only the last-entry case passed.
- `src/yaml.rs` unit test `keeps_block_scalars_whole` now runs at indentations 0, 2 and 4 and fails with the old rule.
- Checks run: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all pass), `mara schema validate && mara project validate` (valid), and the original reproduction through the binary now updates the issue.
