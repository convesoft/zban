---
title: Make test temporary directories unique across processes
status: todo
type: task
priority: low
discovered_from: zb-aapz52
created_at: 2026-11-12T12:56:21Z
---

Found by the v0.1.0 review: the per-name counter added in zb-rvab3w is per process, so a process-per-test runner such as cargo-nextest would bring back the shared-directory race (`VER-AGENT-GUIDE`). The directories also live inside this repository, so a test that skips `init` would find the real `.zban/`.

- [ ] Add the process ID to the directory names
- [ ] Consider moving test projects to the system temporary directory
