# Skill — engine analysis

## What this is for

Use when asked to explain how an engine currently works: what talks to what, which paths are wired in code, which are only architectural, and what state each component owns.

The output is an analysis, not a rewrite plan. Code beats architecture: every claim states whether it is **hooked**, **stubbed**, **contract-only**, **conceptual**, or **stale**.

Apply the triad-engine readability test as a primary lens. The schema names the interface; generated Rust names the objects and traits; handwritten code is mostly real algorithm, typed forwarding, and typed return construction. Plumbing that repeats across components is a candidate for schema emission or shared runtime.

## Reading order

1. Workspace intent and coordination: `ESSENCE.md`, `repos/lore/AGENTS.md`, `orchestrate/AGENTS.md`, `protocols/active-repositories.md`.
2. The apex engine repo's `ARCHITECTURE.md`, then each component repo's `AGENTS.md`, `skills.md`, `ARCHITECTURE.md`, `TESTS.md`, and code map.
3. Every contract crate on the engine fabric, before judging channels.
4. Runtime code, daemon/client entrypoints, Nix apps/checks, scripts, and the tests that witness the runtime path.

When outside research is asked for, borrow vocabulary without importing process: C4 zoom levels (system/container/component), arc42 (building-block, runtime, deployment, crosscutting), SEI views (module / component-and-connector / allocation), DFD trust boundaries for security, and trace/span language for worked request paths.

## Analysis passes

### 1. Engine boundary map

List the engine, components, processes, binaries, sockets, state roots, and deployment/sandbox entrypoints. Mark which processes actually launch today.

### 2. Channel ledger

For each channel, record:

| Field | Question |
|---|---|
| Producer | Who writes or initiates? |
| Consumer | Who reads or handles? |
| Contract | Which `signal-*` crate or byte protocol defines the payload? |
| Transport | Unix socket, raw PTY stream, CLI stdout, file, database, etc. |
| Payloads | Closed request/reply/event variants crossing the boundary. |
| Authority | Which side mints sender, origin, time, slots, IDs, revisions? |
| State effect | Which component state can change after receipt? |
| Status | Hooked, stubbed, contract-only, conceptual, or stale. |

Call out contract-version skew and duplicated wire types.

### 3. Component state machine

For each component, name: entrypoints and public surface; long-lived actors or blocking workers; state fields and durable tables; messages/events handled; transition rules; logs/traces/events written; and what the component refuses to own.

Actors should carry state. If a type is only a forwarding shell, say so.

### 4. Flow traces

Work concrete examples end to end. Each includes: inbound payload; transport and contract; actor/mailbox or direct-call path; durable writes and in-memory mutations; reply/event emitted; downstream possible effects; and the current breakpoint where the path stops.

Use trace/span language: one request path, named steps, propagation across process boundaries.

### 5. Trust, permissions, and auth

Map trust boundaries separately from message payloads: Unix socket owner/mode, runtime directory, system user, filesystem ACLs; provenance tags carried as audit context; cryptographic verification services, keys, signatures, revocation, replay; inter-engine or inter-persona channels; and which checks are implemented versus planned.

Do not treat provenance tags as runtime auth gates unless the code does.

### 6. Observability

Inventory logs, structured events, traces, transcript storage, worker lifecycle events, daemon stderr, database event tables, and CLI output. Say what is durable, what is memory-only, and what is merely a test witness.

### 7. Witness inventory

List tests by constraint, not by filename. A good witness proves the intended component path was used, not just that visible behavior succeeded.

### 8. Drift and next questions

Separate findings into:

- **wired**: code does this now;
- **stubbed**: a valid request returns typed unimplemented or placeholder;
- **contract-only**: shared types exist, no daemon path yet;
- **conceptual**: architecture says it, code does not;
- **stale**: docs, skills, reports, or code names contradict current truth.

Questions for the human are self-contained: restate the concrete code fact, why it matters, and the options.

## Report shape

An engine compendium includes: a one-screen current-state summary; a diagram of hooked versus planned component paths; the channel ledger table; one page per component; worked flow examples; a trust/auth and state-storage summary; the witness inventory; and gaps with decision questions.

Prefer tables and small Mermaid diagrams. Use local file links for code, plain URLs for external method references. Current code and current architecture are primary; history is secondary.
