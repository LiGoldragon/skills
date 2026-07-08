# Skill — Rust storage and wire

## Rules

Keep boundaries typed. In-process values stay Rust types. A daemon's runtime and public boundary speaks typed signal over rkyv. Durable mutable state uses the component's typed store. Human-facing text is a projection, not the machine truth.

Daemons are not text systems. They do not parse or emit JSON or NOTA. Internal tools, tests, evals, fixtures, diagnostics, and new surfaces use typed Rust records with NOTA projection. Use JSON, YAML, CSV, TOML, or other non-NOTA text only when a named external consumer or protocol requires that exact format, such as a provider HTTP API. Keep the adapter, client, CLI, tool, or harness at the edge and convert immediately into typed domain values.

## Storage

Persistent component state lives in a transactional typed store. Values have schema-owned types; do not persist ad hoc text, sidecar indexes, or flat append logs that the daemon re-parses as truth.

Each state-bearing component owns its own state. Do not create a shared cross-component database unless the architecture explicitly names that component as the shared state owner.

Version persisted layouts. Reordering fields, changing enum discriminants, or changing archive feature sets is a coordinated storage-schema change, not a refactor.

## Wire

A channel carries one shared frame type from a contract crate. Add request kinds as variants or typed records in that contract, not as parallel untyped channels.

Validate bytes on receive before reading fields. Newtype platform-dependent or ambiguous values before they cross the wire.

Do not send NOTA text between Rust components as the hot-path protocol. NOTA is the human and agent projection of typed signals, not the daemon wire protocol.

Diagnostics and events from daemon runtime are typed signals. Text rendering, reporting, debug dumps, and audit output belong to tools and clients.

## Human projection

A CLI, tool, harness, or adapter may project typed rkyv signals to NOTA, and parse NOTA back into typed signals when text is needed. The daemon keeps the typed value as truth and regenerates projections; it does not reconstruct owned state by parsing its own projection.

When a daemon needs model judgment or another text-mediated service, put prompts, raw model text parsing, and text diagnostic artifacts in an adapter or client process. That process returns a typed rkyv signal to the daemon.

## Schema changes

Treat storage and wire schema changes as versioned upgrades. Add guards or migrations where old data may exist. Enum variants used in persisted data append without shifting existing discriminants; semantic ordering lives in explicit code, not declaration order.
