---
title: Do not leave .zban/ behind when init fails
status: todo
type: bug
priority: low
discovered_from: zb-gjwr73
created_at: 2026-11-12T15:24:39Z
---

`zban init` creates `.zban/` and then writes `config.toml` into it (`DES-PROJECT-LAYOUT`). When writing the configuration fails, the empty `.zban/` stays behind. No project is found there, because discovery needs `.zban/config.toml`, yet running `init` again fails with `project_exists` until the directory is removed by hand.

Seen with zban 0.1.0 on an `rclone mount` of a local directory, where linking the configuration into place fails with `io` (EPERM, see zb-gjwr73). A full disk or any other write error leaves the same state.

- [ ] Reproduce with a write error that does not depend on hard links
- [ ] Remove the directory `init` created when it fails afterwards, or decide in Mara how a half-made project is reported, and cover it with a test
