# Skill — Rust discipline

## Rules

Every non-test behavior is a method on a non-zero-sized data-bearing type or a trait implementation. Avoid free helpers except `main` and required test wrappers.

Use domain types for domain values. A string, integer, or bool is not enough when the value has a role in the model.

Crate boundaries return the crate's typed `Error` enum. Use `thiserror` or the repo's existing explicit enum shape. Do not expose `anyhow` or `eyre` as the boundary contract.

Use a real parser for structured or external input. Use the NOTA codec for NOTA and an established parser library for other grammars. Hand-rolled splitting is acceptable only for trivial local primitives.

Persistent state uses the component's typed store. Rust-to-Rust wire traffic uses typed contract records. Keep storage schema, wire contract, and generated type changes version-aware.

Long-lived daemons, engines, routers, watchers, and database owners are actors when they own coherent state and lifecycle. The actor type carries data; typed messages carry verbs; handlers do not block.

Keep names as full English words. Do not prefix types with the crate name. Encode direction in names when a type crosses a boundary.

Keep tests in crate-root `tests/` files when they are more than tiny unit probes. Test-only binaries use the `-test` suffix. Witnesses exercise the production boundary they claim to protect.
