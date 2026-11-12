---
title: Read indented headers with unknown block scalars
status: todo
type: bug
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
---

Found by the v0.1.0 review: a header indented by two spaces with an unknown `notes: |` key before `status:` passes `check`, but every update fails with `invalid_value` ("does not read back unchanged"). The read-back guard prevents damage. Likely cause: the entry's end line in `src/yaml.rs` is taken from the scalar's end marker, which lands on the next key (`DES-ISSUE-FILE`, `REQ-DAMAGED-DATA`).

- [ ] Reproduce in a `VER-ISSUE-FILES` test, then fix the entry boundaries
