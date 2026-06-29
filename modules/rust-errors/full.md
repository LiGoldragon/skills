# Skill — Rust errors (typed enums via thiserror)

Each crate defines its own structured `Error` enum; `anyhow`/`eyre`
never cross a component boundary.

## Typed enum per crate via thiserror

Each crate defines its own `Error` enum in `src/error.rs`, derived
with `thiserror`. Variants are structured — they carry the data needed
to render a useful message. Foreign error types convert via `#[from]`.

```rust
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("chunk not found: {0}")]
    ChunkNotFound(Hash),

    #[error("deserialization failed: {0}")]
    DeserializationFailed(String),

    #[error("invalid node: {0}")]
    InvalidNode(String),

    #[error("merge conflict on key ({} bytes)", key.len())]
    MergeConflict { key: Vec<u8> },

    #[error("network: {0}")]
    Network(#[from] reqwest::Error),
}
```

Public APIs return `Result<T, Error>` with the crate's own enum.
Never `anyhow::Result`, `eyre::Result`, or `Result<T, Box<dyn Error>>`:
they erase the error type at the boundary, so callers can no longer
pattern-match on what went wrong, losing the typed-failure discipline
the rest of the Rust rules build up.
