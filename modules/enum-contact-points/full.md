# Skill — enum contact points

## When this applies

Use when writing the load-bearing logic of an engine, handler,
dispatcher, executor, classifier, or state machine — code whose job
is to decide what to do based on the combination of **two**
structured inputs. Most visible in Rust because `match` makes the
cross-product explicit, but the principle holds for any language
with sum types (Haskell, OCaml, TypeScript discriminated unions,
Python tagged dataclasses, Swift enums) or simulated ones (C tagged
unions).

A single-input dispatcher (one enum, one `match`) is plain pattern
matching — not this skill. Reach here when **two enums meet**, or
when one enum meets a method-derived value the matching depends on.

In the schema-derived stack the enums usually come from schema
files: Signal `Input`/`Output`, Nexus mail/action types, SEMA
`SemaCommand`/`SemaResponse`, route/header enums, mail-event enums.
Treat those generated nouns as the real language of the engine.
Hand-written Rust implements the relationship between them; it does
not create a parallel private enum language to avoid matching the
schema objects.

## The principle

Engine logic at the high level is tree-vs-tree matching:
canonically enum-against-enum, sometimes enum-against-a-mix-of-
enums-and-method-calls. **The cross-product of the two variant sets
is the relationship — make it explicit, as a typed `match` or as a
trait whose impl carries the matrix.**

When `State` has *N* variants and `Operation` has *M*, there are up
to *N × M* meaningful pairs: some valid transitions, some
rejections, some no-ops. The pair is the unit of logic. Naming it —
as the head of a `match`, or as a trait impl over the pair — is
what makes the engine readable, and exhaustiveness checking fails
when a variant is forgotten.

The drift you guard against: spreading the matrix across nested
`if state.is_active() && operation.is_marker_request()` chains,
string predicates (`if name.starts_with(…)`), or sentinel-bool
flags. Each encodes one column in one place and one row in another,
so the reader can't see the matrix at all.

## The canonical traits

When the cross-product is large enough to deserve its own type,
extract it as a trait keyed on the *right-hand* enum (or a token
type naming the variant axis).

### `Reaches<Right>` — left value decides what reaches a right value

The left enum is the active side: "given this right-hand value, do
I touch it?" The trait carries the left enum's discrimination; the
impl carries the right-hand cross-product.

```rust
pub trait Reaches<Right> {
    fn reaches(&self, right: &Right) -> bool;
}

impl Reaches<StoredActivity> for ActivityFilter {
    fn reaches(&self, activity: &StoredActivity) -> bool {
        match self {
            ActivityFilter::RoleFilter(role) => &activity.role == role,
            ActivityFilter::PathPrefix(prefix) => match &activity.scope {
                ScopeReference::Path(path) => path.has_prefix(prefix),
                ScopeReference::Task(_) => false,
            },
            ActivityFilter::TaskToken(token) => match &activity.scope {
                ScopeReference::Path(_) => false,
                ScopeReference::Task(activity_token) => activity_token == token,
            },
        }
    }
}
```

The nested `match` IS the cross-product; the type checker fails if a
(filter, scope) pair is forgotten.

### `Contact<Other>` — symmetric meeting, neither side privileged

Two enums meet at equal status (neither is a verb on the other) —
a collision between protocol versions, a comparison between schema
kinds, a match between two filters. The trait names the meeting.

```rust
pub trait Contact<Other> {
    type Outcome;
    fn contact(&self, other: &Other) -> Self::Outcome;
}
```

If one side is clearly the active verb-bearer, prefer `Reaches`; if
neither is, `Contact` names the node-point honestly. The standard
`From` trait is `Contact` realised for projections — `From<historical::Kind>
for current::Kind` is a per-variant `match` whose missing `_ =>`
forces the author to declare what each new variant projects to.

### `Dispatch<Token>` — input variant decides which method to call

The left enum holds the inputs; the right side is a token type
naming which handler to call. One flat trait with one method per
variant, plus a blanket-impl trait whose `match` maps each variant
to its method.

```rust
pub trait OperationHandler {
    type Error;
    async fn handle_ask_handover_marker(
        &mut self,
        payload: MarkerRequest,
    ) -> Result<Reply, Self::Error>;
    async fn handle_ready_to_handover(
        &mut self,
        payload: ReadinessReport,
    ) -> Result<Reply, Self::Error>;
    // … one method per Operation variant
}

pub trait OperationDispatch: OperationHandler {
    async fn dispatch_operation(
        &mut self,
        operation: Operation,
    ) -> Result<Reply, Self::Error> {
        match operation {
            Operation::AskHandoverMarker(payload) => {
                self.handle_ask_handover_marker(payload).await
            }
            Operation::ReadyToHandover(payload) => {
                self.handle_ready_to_handover(payload).await
            }
            // … one arm per variant, mechanically derived
        }
    }
}
```

The right shape when the engine is "one method per operation" with
genuinely different per-variant logic. For schema-emitted signal
roots the generator emits this dispatch trait, and the runtime
engine implements it on a data-bearing object: schema supplies the
variant language, Rust supplies behavior on the object that owns
state. The `signal_channel!` macro emits exactly this — a
`<Operation>Handler` trait and a `<Operation>Dispatch` blanket impl
— for every channel, because the `Operation × handler-method`
cross-product is too mechanical to hand-roll.

## When method calls participate in the cross-product

Not every axis is an enum field. Sometimes one side is a
method-derived value. The question is whether it should join the
cross-product as a temporary or be promoted to a stored field.

### Temporary — cheap to recompute

```rust
match (operation.kind(), current_marker.commit_sequence == report.source_marker.commit_sequence) {
    (OperationKind::ReadyToHandover, true)  => self.accept_handover(report),
    (OperationKind::ReadyToHandover, false) => self.reject_advanced(report),
    // …
}
```

The boolean is a temporary derived from two fields; it joins the
cross-product as a tuple element. Storing it as a flag would
duplicate the two `commit_sequence` reads — a drift hazard.

Diagnostic: **if the derived value reads only from inputs already
at the call site, fold it in as a `match` tuple element; don't
store it.**

### Stored — the derived value names a concept

When the same derivation appears at many call sites, it hides a
typed concept. Promote it:

```rust
// Wrong — recomputed at every call site
if marker.commit_sequence == report.source_marker.commit_sequence {…}

// Right — the concept gets a name and a type
pub enum SequenceAlignment { Aligned, Advanced }

impl HandoverMarker {
    pub fn align_with(&self, other: &HandoverMarker) -> SequenceAlignment { … }
}
// Now the matrix is (HandoverState, Operation, SequenceAlignment)
// — three typed enums, all visible.
```

Diagnostic: **if the derived predicate appears as a `match`-arm
guard at more than two sites, it wants to be a typed enum.**

## Anti-patterns

Each hides a cross-product behind something that doesn't read as
one. The refactor is always the same: make the matrix explicit, as
a nested `match` or a trait named after the relationship.

### Nested if chains over state combinations

The matrix is encoded as conjunctions; the compiler can't tell you
when a cell is missed.

```rust
// Wrong — the matrix is invisible
if matches!(state, HandoverState::Active) && marker_aligned {
    self.accept_ready_to_handover()
} else if matches!(state, HandoverState::Active) && !marker_aligned {
    self.reject_advanced()
} else if matches!(state, HandoverState::Ready { .. }) {
    self.reject_already()
} else {
    self.reject_not_ready()
}

// Right — the tuple makes the cross-product visible; exhaustiveness
// catches the missing cell
match (state, marker_aligned) {
    (HandoverState::Active, true)  => self.accept_ready_to_handover(),
    (HandoverState::Active, false) => self.reject_advanced(),
    (HandoverState::Ready { .. }, _) | (HandoverState::Complete, _) => self.reject_already(),
}
```

### Sentinel values masquerading as state

A field `current_phase: u8` with magic `0 = Active`, `1 = Ready`,
`2 = Complete` is the same matrix with the type system erased.
Every `if phase == 1` is an invisible row. Refactor: define
`pub enum SpiritPhase { Active, Ready, Complete }` and `match` over
`(SpiritPhase, Operation)` — same code, exhaustive.

### Boolean flags hiding a closed enum

```rust
// Wrong — three booleans encoding a three-way axis; illegal
// combinations type-check
if request.is_owner && !state.is_handover && operation.is_marker_request() { … }
```

Three booleans = an eight-cell cube where only some cells are
legal. If the booleans are mutually exclusive or the combinations
are constrained, the type is a closed enum (or a struct of
`Option<T>` where variants carry payloads). Lift them into one
enum, then match. See `skills/typed-records-over-flags.md`.

### String matching as dispatch

```rust
// Wrong — closed set hidden behind a string
match variable.as_str() {
    "PERSONA_SPIRIT_SOCKET"       => Self::MissingSpiritSocket,
    "PERSONA_SPIRIT_OWNER_SOCKET" => Self::MissingOwnerSpiritSocket,
    _ => Self::InputOutput { reason: format!("missing socket {variable}") },
}

// Right — the two strings are a closed set
pub enum SocketEnvironmentVariable { Spirit, SpiritOwner }
impl From<SocketEnvironmentVariable> for &'static str { … }
// match on the enum, not the string
```

See `skills/rust/methods.md` §"Don't hide typification in strings".

### Predicate-method soup on the inner type

When one side needs inspecting through many `is_*` methods, those
methods are an enum waiting to be named.

```rust
impl NotaValue {
    pub fn is_record(&self) -> bool { … }
    pub fn is_sequence(&self) -> bool { … }
    pub fn is_map(&self) -> bool { … }
    pub fn is_identifier(&self) -> bool { … }
    pub fn is_pascal_identifier(&self) -> bool { … }
    // … 14 such predicates
}
```

A caller doing `if v.is_record() { … } else if v.is_sequence() { …
}` is manually walking the (caller-context, NotaValue-variant)
matrix. Refactor: define a `Shape` enum and one `shape(&self) ->
Shape` method. Callers `match` on `Shape`; exhaustiveness catches
missed cases.

Tradeoff: `is_*` predicates are fine for fielded methods used once
or twice (`skills/rust/methods.md` §"Use existing trait domains").
The diagnostic is **scale**: when more than ~4 mutually-exclusive
`is_*` predicates cover the same value, they want to be one enum.

## When the trait is overkill

The trait shapes earn their ceremony when:

- the cross-product has more than ~8 cells,
- the same matrix appears at more than one call site, or
- the relationship deserves a name in its own right.

For a one-call-site matrix smaller than ~8 cells, **a nested
`match` is the right shape** — it fits on one screen and a trait
would add ceremony without revealing structure the `match` doesn't.

Diagnostic: **name the matrix as a trait when naming it helps the
reader; otherwise write the `match`.**

## What this means for engine design

Engine logic decomposes into: receive a typed input → read a typed
state → compute the cross-product entry (direct match or trait
impl) → emit a typed output. The contact points between input and
state are where the engine's logic lives.

When well-designed, the read order is: open the file, see one outer
`match operation` per handler; within each arm an inner `match`
over the relevant state; each inner arm names what happens — usually
one constructor call on the reply enum, optionally with a state
transition. No engine source where this holds is hard to follow.
Every source where the matrix is hidden in
`if state.is_a() && op.starts_with("...") && other_flag` is.

## See also

- `skills/abstractions.md` — verb belongs to noun. The
  cross-product `match` IS the verb; the noun is the relationship
  type.
- `skills/typed-records-over-flags.md` — close cousin: the type
  system carries the meaning.
- `skills/rust/methods.md` §"Don't hide typification in strings" —
  the same rule at the value level.
