# Project knowledge

This project uses Mara 0.4.0-alpha.1 with the engineering template. Canonical
knowledge lives in `*.mara.md` files; the schema, policies, queries, views and
gates live in this directory. No starter knowledge items are generated.

From the project root, validate with:

```sh
mara schema validate
mara project validate
```

## Rust code links

`project.toml` configures Rust symbol indexing with `rust-analyzer scip` for
unignored `.rs` files. Install the component on each development machine:

```sh
rustup component add rust-analyzer
```

File references use `code:src/main.rs`. Symbol references use
`code:<path>::rust::<exact-SCIP-descriptor>`; copy exact references returned by
Mara navigation rather than guessing descriptors. The engineering schema's
`implements` relation connects code to requirements, designs or verifications.
After creating the target item, add an inverse assertion with
`mara relation add <item-ID> implemented_by <code-reference>`, then inspect
the item with `mara related <item-ID>`.

The bundled Tree-sitter grammar/query assets enable comment markers and expanded
declaration content. Put a marker on its own comment line before a supported
declaration, using an existing item ID:

```rust
// @mara implements REQ-EXAMPLE
fn example() {}
```

Markers may target requirements, designs or verifications. Confirm the exact
symbol endpoint with `mara related <item-ID>`; validation alone also accepts
file-level associations. Indexing can invoke Cargo and build scripts.
