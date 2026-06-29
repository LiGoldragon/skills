# Skill — Rust storage and wire

## Rules

Keep boundaries typed. In-process values stay Rust types. Rust-to-Rust wire bytes use the component family's typed binary contract. Durable mutable state uses the component's typed store. Human-facing text is a projection, not the machine truth.

Use JSON, TOML, or other external formats only at boundaries that require them. Keep the adapter at the edge and convert immediately into typed domain values.

## Storage

Persistent component state lives in a transactional typed store. Values have schema-owned types; do not persist ad hoc text, sidecar indexes, or flat append logs that the daemon re-parses as truth.

Each state-bearing component owns its own state. Do not create a shared cross-component database unless the architecture explicitly names that component as the shared state owner.

Version persisted layouts. Reordering fields, changing enum discriminants, or changing archive feature sets is a coordinated storage-schema change, not a refactor.

## Wire

A channel carries one shared frame type from a contract crate. Add request kinds as variants or typed records in that contract, not as parallel untyped channels.

Validate bytes on receive before reading fields. Newtype platform-dependent or ambiguous values before they cross the wire.

Do not send NOTA text between Rust components as the hot-path protocol. NOTA is for authored input, diagnostics, CLI surfaces, tests, and human-readable projections.

## Human projection

A CLI, lock file, debug dump, or audit output may project typed values to NOTA. The daemon keeps the typed value as truth and regenerates the projection; it does not reconstruct owned state by parsing its own projection.

## Schema changes

Treat storage and wire schema changes as versioned upgrades. Add guards or migrations where old data may exist. Enum variants used in persisted data append without shifting existing discriminants; semantic ordering lives in explicit code, not declaration order.
