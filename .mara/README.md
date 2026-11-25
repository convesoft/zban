# Project knowledge

This project uses `@convesoft/mara` (`0.4.0-rc.1`, installed as a dev dependency)
with the engineering template. Canonical knowledge lives in `docs/*.mara.md`;
the configuration, schema and Soufflé rules live in this directory.

After `pnpm install`, fetch the pinned Rust tree-sitter grammar and validate
from the project root:

```sh
pnpm mara code add rust
pnpm mara schema validate
pnpm mara project validate
```

Run `pnpm mara prime` for the full agent workflow guide.

## Rust code links

`config.toml` configures Rust code-marker scanning for `src/**/*.rs` and
`tests/**/*.rs` with a pinned tree-sitter grammar (`pnpm mara code add rust`).
Without the cached grammar, Mara falls back to basic comment-prefix scanning and
reports a diagnostic.

The engineering schema's `implements` relation connects code to requirements,
designs or verifications. Put a marker on its own comment line before a
declaration, using an existing item ID:

```rust
// @mara implements REQ-EXAMPLE
fn example() {}
```

Inspect links for an item with `pnpm mara relation list <ID>`.
