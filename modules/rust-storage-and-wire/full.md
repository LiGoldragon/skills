# Skill — Rust storage and wire (redb + rkyv)

redb holds component state that must survive a restart; rkyv is the binary contract between Rust components — both for durable values inside redb and for wire bytes between processes.

## What goes where

The first boundary decision: what crosses it, and to whom does the other side answer?

| Boundary | Format | Why |
|---|---|---|
| In-process: actor ↔ actor, method ↔ method | typed Rust values | The type system is the schema. No serialization until something leaves the process. |
| Process ↔ process: daemon ↔ harness, IPC, sockets, pipes | **rkyv** archives | Zero-copy reads, content-addressable canonical bytes, bytecheck validation. The binary contract is the wire. |
| Component ↔ disk: queues, transition logs, harness bindings, snapshots | **redb** tables of rkyv values | Single embedded store, crash-consistent, snapshot reads, no separate server. |
| Component ↔ human: CLI, debug prints, audit dumps | NOTA text projection | Human-readable; projected from the typed record, never the source of truth. |
| Component ↔ legacy external system | the format the legacy demands | Adapters live at the edge; external bytes round-trip through one explicit codec at the boundary. |

rkyv is the binary contract for everything between Rust components. NOTA is the projection when the other side is a human. JSON / serde appears only at external boundaries that demand it.

Sema values are Signal-compatible archived records — not text, and not necessarily IPC frames. A redb table value is the same rkyv-archived typed record that flows on the wire, but it is not literally a Signal `Frame` envelope unless the table records frames. The shared truth is the typed archived record: Signal wraps it for inter-process traffic; redb stores it for durability.

## redb — the durable store

Persistent component state lives in redb: router queues, harness bindings, transition logs, coordination state — anything the running component mutates and re-reads.

- **Persistent state lives in redb.** Not flat files, JSON files, or bare blobs.
- **Values are rkyv-archived bytes.** Not serde-JSON, hand-rolled binary, or text.
- **One redb file per component.** Each component owns its own database; no shared cross-component database.
- **Component state goes through the component-owned Sema layer.** No ad hoc registry files, sidecar indexes, JSON catalogs, or text manifests for state the component mutates and re-reads. If the data is component state, declare it as typed Sema tables owned by that component.

```rust
// Right — typed record archived with rkyv, stored in redb
const CLAIMS: TableDefinition<&str, &[u8]> = TableDefinition::new("claims");

let txn = self.db.begin_write()?;
{
    let mut table = txn.open_table(CLAIMS)?;
    let bytes = rkyv::to_bytes::<rancor::Error>(claim)?;
    table.insert(role.as_str(), &bytes[..])?;
}
txn.commit()?;
```

Anti-pattern: a flat-file append-only log (`state.log`) re-read on startup — no transactions, no atomic updates, and the parser races the writer.

## rkyv — the binary contract on the wire (signaling)

The rkyv-archive-on-the-wire pattern is **signal** (canonical reference: `~/primary/repos/signal`). The verb is **to signal**: a component signals another by sending a length-prefixed rkyv archive on the wire. Today signaling is local IPC over Unix sockets, TCP, pipes, or mmap; cross-machine signaling is a deferred extension.

Both ends compile against the *same* rkyv feature set, exchange `Archived<T>` for a shared frame type `T`, and frame with a length prefix per archive.

```rust
// Right — rkyv frame, length-prefixed
let archived = rkyv::to_bytes::<rancor::Error>(&request)?;
stream.write_all(&(archived.len() as u32).to_be_bytes())?;
stream.write_all(&archived)?;

// Reader (zero-copy validate-on-receive)
let archived = rkyv::access::<ArchivedRequest, rancor::Error>(&buf)?;
let id = archived.id;        // direct read, no allocation
```

The wire schema *is* the framing; both parties know the same `Frame` type and the bytes are `Archived<Frame>`. The discipline:

- **The shared `Frame` type lives in a contract repo.** When two or more components speak the same wire, the record types are not re-defined per consumer; they live in a dedicated crate every consumer depends on. See `skills/contract-repo.md`.
- **One frame type per channel.** A socket carries one shared `Frame` enum; new request kinds are new variants, not new channels.
- **Same feature set both ends.** Adding or dropping an rkyv feature (`little_endian`, `pointer_width_32`, `unaligned`, `bytecheck`) breaks archive compatibility *silently*. Pin the feature set exactly per lore's `rust/rkyv.md`.
- **Validate on receive.** Use `rkyv::access` (or `from_bytes`), which runs bytecheck. Don't read fields out of unvalidated buffers.
- **Newtype the wire form.** `WirePath(Vec<u8>)` over `PathBuf`; platform-dependent stdlib types don't archive deterministically.
- **No `serde_json` between Rust components, ever.** JSON erases the schema; it appears only at external boundaries that demand it.

The messaging substrate that lets Persona and the eventual Criome merge is rkyv on the wire — that convergence works only because both sides agree on the same archive contract today. ("Criome" here means the eventual universal computing paradigm, not today's `criome` daemon.)

## NOTA — the human-facing projection

NOTA is the project's text syntax; Nexus is a NOTA-using request/message surface, not a second syntax. NOTA is **not the wire between Rust components** — it is what a typed record *projects to* when a human, a CLI, or a git diff is on the other side.

- A `Lock` record is a typed Rust value, archives to rkyv inside redb, and projects to NOTA when written to a `<role>.lock` file. The text projection is regenerated from the record; the daemon never reconstructs the record by parsing the text.
- The CLI form `orchestrate '(ClaimScope ...)'` takes one NOTA record on argv and prints one NOTA record on stdout. Inside the binary the value travels as typed Rust.
- A convenience CLI such as `message` may hide a common Nexus wrapper, but still constructs a typed NOTA record shape within NOTA syntax.
- Debug dumps, audit logs, error renderings — all NOTA projections of typed records.

The asymmetry: humans use NOTA, machines use rkyv. The codec at the boundary is `nota-codec`, the *only* text codec each crate ships. No second project-wide text format.

## Anti-patterns

| Anti-pattern | Why it's wrong | Replace with |
|---|---|---|
| Flat-file log as durable state | No transactions; parser races writer | redb table with rkyv values |
| Ad hoc registry file (`registry.json`, sidecar text index) | Splits truth from the component's typed store; no transaction boundary or schema guard | Component-owned Sema tables in the component's redb |
| JSON between Rust components | Schema erased; no pattern-match on bytes; bytecheck unavailable | rkyv frame + length prefix |
| Ad-hoc binary (`to_le_bytes` chains) | No schema validation; byte-order bugs; rewriting rkyv badly | rkyv archive |
| NOTA text on the inter-component wire | Forces re-parsing canonical text in the hot path | rkyv frames; NOTA stays the CLI/lock-file form |
| Storage actor as namespace (`StorageActor` answering "store this"/"fetch that" for everyone) | Verb-shaped; owns *storing*, not domain data | Each domain actor opens its own tables on the shared `Database` |
| `Arc<Mutex<Database>>` shared across actors | Coarse lock defeats redb's transaction model; serializes writers | One actor per logical data domain; pass values, not handles |
| Blocking work inside a normal actor handler | The mailbox stops receiving; the hidden wait becomes the real lock | Dedicated supervised IO/command/worker actor or pool |
| Public ZST actor noun (`ClaimNormalizer` empty, exported as the actor) | The public name is a label; verbs drift onto the wrong noun | Kameo's `Self IS the actor`: fields on the actor type, methods on `&mut self`, consumers reach for `ActorRef<ClaimNormalizer>` |
| Reading a record from text in the daemon (`Record::from_nota(disk_text)?`) | The text is a projection; typed state and disk text drift silently | Daemon owns the typed record; text is only a boundary projection |
| Mixed rkyv feature set across crates | Archives from one don't validate in the other; failure is silent (wrong values, not a parse error) | Pin the exact rkyv feature string per lore |
| Reordering struct fields casually | rkyv archives change layout on field reorder in 0.8 — old data unreadable | Append-only fields; treat layout changes as coordinated upgrades |
| `anyhow` / `eyre` at component boundaries | Erases typed-failure discipline; callers can't pattern-match | crate's own `Error` enum via thiserror |

## Validated patterns

| Pattern | When to use | Notes |
|---|---|---|
| `TableDefinition<&str, &[u8]>` with rkyv-encoded value | Most component tables | Key is domain-typed (`RoleName`, `MessageId.as_str()`); value is rkyv bytes |
| Single `Frame` enum per channel | Inter-component sockets | New variants for new requests; never a second channel for "the new thing" |
| Length-prefixed framing | TCP / UDS streams | 4-byte big-endian length, then the archive |
| `rkyv::access` on the read path | Hot-path reads where ownership isn't needed | Returns `&Archived<T>`; zero allocation |
| Version-skew guard at boot | Any persisted store or long-lived socket | Known-slot record `(schema_version, wire_version)`; hard-fail on mismatch |
| Sync façade on actor `State` | Tests for components owning redb + rkyv | See lore's `rust/testing.md` |
| Newtype around platform-fragile stdlib types | `PathBuf`, `OsString`, `SocketAddr` on the wire | `WirePath(Vec<u8>)`; deterministic across platforms |

## Named exceptions — text-on-disk that stays text

The rule covers *state the component mutates and re-reads* and *bytes between Rust components*. Some text-on-disk forms stay text by design:

- **Lock-file projections.** `<role>.lock` files are gitignored human-readable runtime coordination state, read with `cat` when needed or replaced by `orchestrate "(Observe Roles)"`. The redb store is the in-process truth; the lock file is the outward projection regenerated from the record.
- **Configuration files.** `Cargo.toml`, `flake.nix`, per-repo configs — inputs, not state.
- **Reports and prose docs.** Markdown is markdown.
- **Interchange artifacts.** A NOTA-line file shared for one-shot ingestion is interchange, not running state.
- **Logs for human eyes.** A line-oriented audit log for `tail -f` is a projection. The structured log a component re-reads on restart is not — that lives in redb.

If a component owns the data and mutates it during operation, it lives in redb + rkyv. If a component sends bytes to another Rust component, those bytes are rkyv archives.

## Schema discipline

rkyv archives are schema-fragile: adding, removing, or reordering fields changes the layout.

- **No silent backward compatibility.** Old archives don't read into new types or vice versa.
- **Version-skew guard.** A known-slot record carrying `(schema_version, wire_version)`, checked at boot, hard-failing on mismatch. rkyv's own version handling is not enough.
- **Treat schema changes as coordinated upgrades.** In 0.8, a field reorder *and* a field addition are both breaking. Plan rollout across every consumer.
- **Enum variant evolution: append at the end, express semantic order separately.** A derived-`Archive` enum with persisted data must never reorder or insert variants in a way that shifts existing discriminants. New variants append last under `#[repr(u8)]` so prior variants keep their byte values and archived bytes still decode. Semantic ordering (a variant should sort "lowest"/"highest") is expressed via a manual `Ord` / `order_rank` impl, NEVER `#[derive(Ord)]` on declaration order. Example: `Magnitude::Zero` appended after `Maximum` to keep `Minimum=0..Maximum=6` stable, with manual `order_rank` returning `Zero=0` for semantic-bottom. The "declare new variant first for derived `Ord`" shape is archive-unsafe — it shifts every persisted byte by one.

For tool-level details (the canonical feature set character-for-character, derive-alias pattern, encode/decode API, `bytecheck` semantics), see lore's `rust/rkyv.md`. This skill is *what discipline to apply*; lore is *how the tool works*.

## The sema-engine pattern (default for new components)

This describes today's typed-storage substrate (the eventual `Sema` is broader). Two layers:

- **`sema`** — the storage *kernel*: redb file lifecycle, the typed `Table<K, V: Archive>` wrapper, txn helpers, the standard `Error` enum, the version-skew guard, and the `Slot(u64)` + slot-counter utility. Low-level; most components do not depend on it directly.
- **`sema-engine`** — the full *database engine* library over `sema` and `signal-core`: registered record families, typed Signal-verb execution (`Assert`, `Match`, `Subscribe`), operation log + snapshot identity, subscription surface. Pure library — no daemon, Kameo, tokio, NOTA, or `signal-persona-*` deps. First consumer is `persona-mind`; Criome follows.

**Default for new state-bearing components: depend on `sema-engine`, not on `sema` directly.** `sema-engine` owns the engine surface (record family registration, Assert/Match/Subscribe verbs, operation log range, mutation receipts, snapshot identity); the component owns domain validation, actors, sockets, authorization, and the daemon shape. Reach for `sema` directly only for low-level kernel operations the engine doesn't expose (rare; usually a signal that `sema-engine` should grow the surface instead).

Ownership is **by state-bearing component**:

```
signal-core            sema-engine             sema
  signal-persona-mind    Engine in persona-mind  ├─ persona-mind.redb
  signal-persona-router  Engine in persona-router├─ persona-router.redb
  signal-persona-harness Engine in persona-harness ─ persona-harness.redb
```

The Engine instance lives inside the component daemon, opens the component's redb file through `Sema::open_with_schema`, and registers the typed record families the component owns. Record types live in the matching `signal-*` contract crate when they cross a component boundary; purely internal persisted records may live inside the component.

**Signal traffic builds on `signal-core`** — the wire kernel (typed frames, envelopes, channel macro). Every component-specific `signal-*` contract crate layers its typed records on top of `signal-core`'s primitives. Don't invent a parallel framing or envelope mechanism per contract.

New components consuming the substrate:

```toml
# Cargo.toml
sema-engine = "..."           # the typed database engine
signal-core = "..."           # wire kernel for inter-component frames
signal-<component> = "..."    # the contract crate(s) this component speaks
```

Inside the component:

```rust
use sema_engine::{Engine, EngineOpen, TableDescriptor, TableName, Assertion, QueryPlan};

let mut engine = Engine::open(EngineOpen::new(database_path, SchemaVersion::new(1)))?;
let family = engine.register_table(TableDescriptor::new(TableName::new("thoughts")))?;
engine.assert(Assertion::new(family.clone(), thought))?;
let snapshot = engine.match_records(QueryPlan::all(family))?;
```

Prefer an internal module for component-local table layouts (`persona-mind/src/tables.rs`). Create a dedicated Sema crate only after reuse is real and its architecture is explicitly named. Do not create broad umbrella Sema crates for meta projects just because the meta repo composes several components — meta projects compose component storage owners; they do not own a shared storage layer by default.

## Why this discipline is strict

The rules feel laborious before the components are written but not *while* they run: a typed wire makes wrong calls fail at compile time, a typed store makes wrong reads fail at boot time, and projection-from-record makes the disk and in-memory truth impossible to disagree.

## See also

- `skills/contract-repo.md` — how typed contracts are organized in repos.
- `skills/rust/errors.md` — typed errors at storage and wire boundaries.
- `lore/rust/rkyv.md` — rkyv tool reference (feature pin, derive alias, encode/decode API, bytecheck).
