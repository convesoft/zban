---
title: Reformat for rustfmt in Rust 1.100
status: done
type: task
priority: high
created_at: 2026-11-12T19:44:29Z
closed_at: 2026-11-12T19:44:29Z
---

Rust 1.100.0 is now the stable toolchain that CI installs (`dtolnay/rust-toolchain@stable`). Its rustfmt formats three macro invocations in `src/issue.rs` differently, so `cargo fmt --all -- --check` fails on `main` and on every pull request (first seen on PR #44).

- [x] Run `cargo fmt --all` with Rust 1.100 and commit the result
- [x] `cargo clippy --all-targets -- -D warnings` and `cargo test` pass on Rust 1.100

## Summary of Changes

Ran `cargo +stable fmt --all` with rustc 1.100.0; only `src/issue.rs` changed (three `vocabulary!`-style macro calls now put one argument per line). On 1.100.0, `cargo clippy --all-targets --locked -- -D warnings` passes and `cargo test --locked` passes 178 tests. The rustfmt of Rust 1.89, the declared `rust-version`, formats these calls the other way, so formatting is checked with the current stable only, as CI already does. The format check itself runs in CI on this pull request, which covers it under the AGENTS.md rule.
