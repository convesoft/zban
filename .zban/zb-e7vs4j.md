---
title: Decide whether release archives ship notices for the Rust standard library and musl
status: draft
type: task
discovered_from: zb-ve67q7
created_at: 2026-11-12T15:51:48Z
---

Found while adding third-party license notices to the release archives (zb-ve67q7, `ADR-RUST-BINARY`). `THIRD-PARTY-LICENSES` covers the crates in `Cargo.lock`, which is all `cargo-about` can see. The binaries also contain code that `Cargo.lock` does not list:

- Rust's standard library, `MIT OR Apache-2.0`, and the crates it is built from, under their own licenses, on every target.
- On the two `*-unknown-linux-musl` targets, the musl C library (MIT) and LLVM's libunwind (`Apache-2.0 WITH LLVM-exception`), linked statically from the toolchain's `lib/rustlib/<target>/lib/self-contained/`. A local `x86_64-unknown-linux-musl` release build holds musl's `__libc_start_main` and libunwind's `__unw_getcontext`.

The macOS binaries link the system C library dynamically. The Rust toolchain documents the standard library's licenses in `share/doc/rust/COPYRIGHT-library.html` and `share/doc/rust/licenses/`.

This needs a maintainer decision: whether the archives should carry these notices too, and if so how, for example fixed texts appended through `about.hbs` or files copied from the toolchain.

- [ ] Decide whether the archives need notices for the standard library, musl and libunwind, and record the answer in `ADR-RUST-BINARY`
- [ ] Ship them as decided
