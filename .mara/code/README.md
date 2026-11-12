# Rust parsing assets

`rust.wasm` and `rust.scm` were copied from the Mara repository's `.mara/code/`
at revision `29eec86665ad454527d8ff970403168561fa5b5b`. That repository documents
the grammar as tree-sitter-rust 0.24.2. The upstream MIT notice is preserved in
`LICENSE-rust`.

The query captures declarations, names, scopes, attributes and comments for
Mara's symbol content and comment-marker attachment. SCIP identities still come
from rust-analyzer, as configured in `../project.toml`.
