# Skill — naming (full English words; no redundant ancestry)

## Two rules, applied together

Apply this every time you name a type, function, field, variable,
module, or parameter. Two rules pull in opposite directions and only
work as a pair:

1. **Spell every identifier as a full English word.** The default is
   the spelled-out English form. Abbreviations require one of the six
   narrow exceptions below.
2. **Names don't carry their full ancestry.** A type, variant, or
   field belongs to its surrounding namespace; repeating the namespace
   is redundant ceremony. Inside `Profile` the field is `size`, not
   `profileSize`. Inside `signal-spirit` the type is `Entry`,
   not `IntentEntry`.

"Full word" alone produces `IntentRecordIdentifier` (every ancestor
named). The ancestry rule alone produces `Id` or `Ctx` (short but
abbreviated). Apply both: the name carries the words the namespace
doesn't, in full English.

Identifiers are read far more than they are written. A cryptic
abbreviation saves the writer a few keystrokes once and costs the
reader a mental lookup at every occurrence.

## The offender table

Bad → good. When you reach for any two-to-three-letter shape, check
here first.

| bad | good |
|---|---|
| `lex` | `lexer` |
| `tok` | `token` |
| `id` / `ident` | `identifier` |
| `op` | `operation` (or specific: `assert_op`) |
| `de` | `deserializer` |
| `pf` | `pattern_field` |
| `ctx` | `context` (or specific: `parse_context`) |
| `cfg` | `config` (or `configuration`) |
| `addr` | `address` |
| `buf` | `buffer` |
| `tmp` | `temporary` (or name what it holds) |
| `arr` | `array` (or what it contains) |
| `obj` | (name what it actually is) |
| `params` | `parameters` |
| `args` | `arguments` |
| `vars` | `variables` |
| `proc` | `procedure` or `process` |
| `calc` | `calculate` |
| `init` | `initialize` |
| `repr` | `representation` |
| `gen` | `generate` or `generator` |
| `ser` / `deser` | `serialize` / `deserialize` |

## Permitted exceptions — six, named, no others

1. **Loop counters in tight scopes (<10 lines).** `for i in 0..n` is
   fine. Beyond ~10 lines or nested, use descriptive names.
2. **Mathematical contexts** where the math itself uses the symbol:
   `x`, `y`, `z`, `theta`, `phi`, `lambda`, `n` for sample size, `p`
   for probability — only when the surrounding code or comment
   establishes the math context.
3. **Generic type parameters.** `T`, `U`, `V`, `K`, `E`. Use a
   descriptive name when the parameter carries non-trivial semantics.
4. **Acronyms that have fully passed into general English.** `cpu`,
   `url`, `http`, `json` — the acronym has functionally become the
   English word; the spelled form is awkward or no longer remembered.
   The test for adding others: *has the acronym functionally become
   the English word?* `uuid`, `tcp`, `udp`, `dns` often qualify in the
   right context. Internal short forms of system concepts (`db`, `os`,
   `ui`, `io`, `ram`) are convenience shortenings, not English words —
   spell them (`database`, `operating_system`, `interface`) unless the
   spelled form is itself awkward. Do **not** use `id`; spell
   `identifier`. Most code-side "acronyms" (`ctx`, `cfg`, `addr`,
   `tok`, `buf`, `proc`) are convenience shortenings and belong in the
   offender table, not here.
5. **Names inherited from `std` or well-known libraries.** `Vec`,
   `HashMap`, `Arc`, `Rc`, `Box`, `Cell`, `RefCell`, `Mutex`, `mpsc`,
   `regex`. Do not rename these; do not extend the abbreviation
   pattern to your own types.
6. **Domain-standard short names documented in an `ARCHITECTURE.md`.**
   `slot`, `node`, `edge`, `frame` are full words and need no
   exception. If a true short form is load-bearing in the schema, name
   it in `ARCHITECTURE.md` so the exception is explicit; otherwise
   spell it out.

## Name length is proportional to scope

A 3-line loop counter can be `i`. A function parameter that lives for
50 lines must read as English. A module-level type that appears across
the codebase must spell itself out.

This is not "verbose names everywhere":
`calculate_the_total_amount_of_items` is worse than `total_items`. The
goal is *clear*, not *long*.

## When generating code, don't propagate the local dialect

Spell identifiers as full English words by default. When surrounding
code uses cryptic identifiers, do not copy them into new code. Either
rename (if rename is in scope) or use the full form for new
identifiers and flag the inconsistency as a follow-up. Pattern-matching
the local dialect is exactly the failure mode this rule breaks.

## The "feels too verbose" feeling is the bug, not the criterion

When a spelled-out name (`AssertOperation`, `Deserializer`,
`PatternField`, `RelationKind`) "feels needlessly verbose," that
feeling is inherited prejudice from constraints that no longer apply —
not a signal to shorten. The full word reads as English; the
abbreviation reads as ceremony to be decoded. The cost of mis-naming
is paid every time the name is read; the keystroke saving is paid once.

When you catch the feeling: re-read the name as English. Does
`AssertOperation` read as English? Yes. Does `AssertOp`? No — it
requires expansion. The full English form wins unless the name falls in
one of the six exception classes. There is no exception class for
"feels verbose."

## The name carries the context the namespace doesn't

When naming a field, method, or local that *could* be a short word
(`size`, `name`, `body`), ask: **does the surrounding namespace already
give the noun?**

- If the access path is `profile::size` (module path) or `profile.size`
  (field of a `Profile`-typed thing), `size` reads as English at the
  call site — `profile.size` *is* the description.
- If the field stands alone — a top-level binding, an unqualified
  parameter, a record field that appears outside its parent type's
  context — then `size` is too thin and `profileSize` carries the
  missing context.

```rust
// Right — namespace qualifies; field name stays short
struct Profile {
    pub size: u64,        // accessed as profile.size
}

// Right — naked parameter; name carries the context
impl MetricsRecorder {
    pub fn record(&self, profileSize: u64, requestCount: u32) { … }
}

// Wrong — Profile's namespace already names "profile"
struct Profile {
    pub profileSize: u64,  // profile.profileSize reads as repetition
}

// Wrong — naked parameters claim a context that isn't there
impl MetricsRecorder {
    pub fn record(&self, size: u64, count: u32) { … }
    //                   which size? which count?
}
```

Naked names *claim* a context they don't have; the type system can't
catch that silent failure of clarity. This refines "full English
words": it isn't *more words* that wins, it's *the words the namespace
doesn't already supply*. `messageId` when there's no `Message`
namespace; `id` when there is.

## Anti-pattern: prefixing names with their namespace or domain

A name belongs to its surrounding context, not to the cross-crate
global namespace. The crate, module, contract, channel, enclosing
enum, and owning component are all namespaces; repeating any of them in
the name is redundant ceremony.

```rust
// Wrong — crate name redundant at every use site
pub struct ChromaRequest { … }
pub struct ChromaError { … }

// Right — call sites read chroma::Request, chroma::Error
pub struct Request { … }
pub struct Error { … }
```

For contract crates the same rule applies to the contract's domain:

```rust
// Wrong — the contract crate already says repository-ledger
pub struct RepositoryPushObservation { … }
pub enum RepositoryLedgerRequest {
    RepositoryPushObservation(RepositoryPushObservation),
}

// Right — use-site reads signal_repository_ledger::PushObservation
pub struct PushObservation { … }
pub enum Request {
    PushObservation(PushObservation),
}
```

The discriminator: **does the leading word *describe* the type, or name
a namespace already visible at the use site?** Descriptive words stay;
namespace prefixes go.

| Prefix is wrong | Prefix is fine |
|---|---|
| `ChromaRequest` (Chroma is the crate) | `VisualState` (Visual describes the kind) |
| `StylixOptions` (Stylix is the crate) | `ColorScheme` (descriptive) |
| `NotaCodecError` | `LexerError` |
| `PersonaMessageRouter` | `MessageRouter` |
| `RepositoryChangedFileQuery` in `signal-repository-ledger` | `ChangedFileQuery` |
| `HarnessHarnessEvent` in `signal-persona-harness` | `LifecycleEvent` |

The standard library is the canonical reference: `Vec`, `HashMap`,
`Arc`, `Cell`, `Mutex` — never `StdVec`, `StdArc`. Well-shaped crates
name their types as if `use crate_name::*` were the norm, even when it
isn't.

LLM agents are prone to this because the prefix "feels safe" (avoids
collisions, matches the file name, looks self-documenting) and tokens
are free. The agent skips the harder thinking ("what does this type
actually represent?") for the shallower disambiguator ("which crate is
it from?").

## Anti-pattern: repeated category words across sibling names

When several adjacent types or variants share a prefix or suffix —
`*Query`, `*Command`, `*Event`, `*Listing`, `*Selection`, `*Mode`,
`*Result` — stop and ask which layer the repeated word belongs to. It
may be a missing parent enum, relation, record, module, contract
operation, or lower execution effect exposed at the wrong layer.
Repeated category words are **schema smells**, not naming choices.

```rust
// Wrong — Query repeated as a suffix across five siblings
Match EventQuery(EventQuery),
Match RecentRepositoriesQuery(RecentRepositoriesQuery),
Match ChangedFileQuery(ChangedFileQuery),
Match CommitMessageQuery(CommitMessageQuery),
Match CatalogQuery(CatalogQuery),

// Possible correction — Query is the parent enum; siblings name targets
Match Query(Query),

pub enum Query {
    Events(EventSelection),
    RecentRepositories(RecentRepositorySelection),
    ChangedFiles(ChangedFileSelection),
    CommitMessages(CommitMessageSelection),
    Catalog(CatalogSelection),
}
```

That correction is not automatic. If `Query` is the public act this
contract receives, it may belong one layer higher as a contract
operation while the lower database/read operation stays inside the
daemon — the daemon may lower it internally to a Sema Match effect. The
threshold is behavioral, not numeric: when you find yourself adding a
third sibling with the same suffix, stop and ask. The schema is asking
for a new structural layer; don't decide in advance whether that layer
is a parent payload, a contract operation, or a lower execution effect.

This pairs with the no-redundant-ancestry rule. Ancestry says "don't
restate what the namespace already supplies." Repeated-category says "if
a word recurs across siblings, the schema is missing a namespace that
would supply it." Together: names carry only what the schema's structure
doesn't; when names repeat a word, that word should become structure.
Both are failure modes that flatten a schema that should grow into a
tree.

## Anti-pattern: framework-category suffixes on type names

A type's name should describe what it IS or what role it plays — never
the framework category it falls into. A `Counter` that implements the
`Actor` trait IS an actor; calling it `CounterActor` adds the category
without adding meaning.

```rust
// Wrong — framework-category suffix
pub struct CounterActor { count: i64 }
pub struct IncMessage { amount: i64 }
pub struct ClaimNormalizerActor { … }

// Right — name says what the type IS / does
pub struct Counter { count: i64 }
pub struct Inc { amount: i64 }
pub struct ClaimNormalizer { … }
```

The discriminator: **does the suffix describe the type's role, or tag
the framework category it falls into?** Role-shaped suffixes stay;
category-shaped suffixes go.

| Suffix is wrong (framework category) | Suffix is fine (descriptive role) |
|---|---|
| `*Actor` | `*Supervisor` (it supervises children) |
| `*Message`, `*Msg` | `*Resolver` (it resolves something) |
| `*Handler` | `*Decoder`, `*Encoder` (it decodes/encodes) |
| `*Listener`, `*Subscriber` (as a generic trait-participation tag) | `*Tracker`, `*Cache`, `*Ledger` (it holds that state); `Subscriber` as the *role* of the long-lived actor on the receiving side of a publish/subscribe channel |
| `*Object`, `*Type`, `*Class` | `*Builder`, `*Factory` (when actually building) |
| | `*Handle`, `*Client`, `*Ref` — relationship-naming (the value IS a held authority on the target; same shape as `JoinHandle`, `FileHandle`) |

`Handle` is not a framework-category tag. It names a relationship — the
value IS the caller's held authority to a live service or resource, same
pattern as `tokio::task::JoinHandle`, `std::fs::File`,
`std::process::Child`. `*Handle` earns its place when the wrapper
carries domain content: lifecycle ownership, capability narrowing, error
vocabulary mapping, topology insulation, or send-policy enforcement. A
bare `Handle` that just holds an `ActorRef<A>` and delegates
method-by-method without adding domain content is runtime-laundering —
drop the wrapper and expose `ActorRef<A>` directly.

A category tag forces the reader to mentally strip it at every use site
("`CounterActor` — it's a Counter that's an Actor — it's always an Actor
here, so just Counter"). That strip is paid every time. LLM agents reach
for category tags for the same reason as crate-name prefixes: the
shallower disambiguator over the harder work of finding the role-shaped
name.

## Different scopes get different names

When a concept names both **what is built today** and the **larger
eventual form** it is one step toward, those are different things and
get different names. Same-name conflation lets the encompassing vision
silently overwrite today's snapshot — readers can't tell which scope a
doc is in.

Today's piece earns a concrete narrower name; the eventual name stays
reserved for the realized form. Live examples:

- `sema-db` (today's typed database library) vs `Sema` (the eventual
  universal medium for meaning).
- The current `criome` daemon (today's sema-ecosystem records
  validator) vs `Criome` (the eventual universal computing paradigm in
  Sema).

This is a scope discipline, not a quality one. Today's narrower piece is
held to ESSENCE's full priorities — built rightly for its scope, not as
a draft of the eventual. "Today's piece" is not a license to cut
corners.

## Schema and emitted Rust mirror each other

The naming system mirrors between schema-emitted code and Rust source.
A colon-path namespace in a schema (`spirit:signal:Frame`) maps directly
to the Rust module-and-type path (`spirit::signal::Frame`):
colon-to-double-colon, kebab-case crate names becoming snake_case module
names, PascalCase type names unchanged.

Two consequences:

- **Grep across both surfaces uses the same identifier.** Searching
  `Frame` in the schema file and in the emitted Rust finds the two views
  of one identity.
- **Either surface is a sufficient entry point.** An agent reading
  either can locate the other mechanically — no separate mapping table.

This pairs with side-by-side file placement: emitted Rust lives at
`src/schema/<module>.rs` inside the consumer crate, alongside
hand-written Rust. The mirror-naming makes the two navigable in either
direction.

## See also

- `skills/abstractions.md` — verb-belongs-to-noun; forces the naming
  step that this rule then decides.
- `skills/beauty.md` — a name that doesn't read as English is a
  diagnostic reading of structural ugliness.
- `skills/rust-discipline.md` — Rust-specific application, including the
  no-crate-name-prefix enforcement and `Handle`/`ActorRef` shape.
