# Module - Rust core

## Rust Core Purpose

Rust work follows workspace Rust discipline without requiring a worker packet to
carry every Rust reference file. Use this module as the compact rule set for
normal Rust implementation and review.

## Rust Parsing Storage And Wire

Use a real parser for structured input. In this workspace that normally means
the NOTA codec for NOTA and `winnow` or an established parser library for other
grammars. Hand-rolled string splitting is review debt unless the input is truly
trivial and local.

Persistent state normally uses redb. Binary wire and durable schema objects use
rkyv where the surrounding component family already does. Keep storage schema,
wire contract, and generated type changes version-aware.

## Rust Actors And Components

Long-lived daemons, state engines, routers, watchers, delivery engines, and
database owners are actors when they own coherent state and lifecycle. In
Kameo-shaped code, the actor type itself carries the data, and each verb is a
typed message implementation rather than one untyped message enum.

Component work keeps the daemon, thin CLI, and signal-* contract distinct. A
CLI drives the daemon path; it does not recreate daemon state transitions by
opening the database directly.

## Rust Tests And Layout

Keep tests in crate-root `tests/` files when they are more than tiny unit
probes. Test-only binaries use the `-test` suffix. Witnesses exercise the
production boundary they claim to protect: parser, trait surface, actor
message, wire frame, daemon CLI, or storage reader.
