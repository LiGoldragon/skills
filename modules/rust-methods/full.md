# Skill — Rust methods and types

This is the Rust enforcement of the cross-language rules in
`skills/abstractions.md`, `skills/naming.md`, and `skills/beauty.md`.

## Methods on types, not free functions

Every Rust function in production is a method or associated function
on an `impl` block of a **non-zero-sized data-bearing type**, or a
trait impl. The only exemptions are `fn main()` and items inside
`#[cfg(test)]` modules. **Module-level `fn`, `const fn`, and
`async fn` are all forbidden** — the rule is about *function
placement*; "it's a `const fn`" and "it's an `async fn`" are not
escape hatches. Test code may use free helpers when that keeps a
test readable; production code may not.

Trait methods are preferred over inherent methods; methods on real
data-bearing types are the minimum.

```rust
// Wrong
pub fn parse_cert(pem: &str) -> Result<Cert, Error> { … }

// Right
impl Cert {
    pub fn from_pem(pem: &str) -> Result<Self, Self::Error> { … }
}
```

Private helpers are not an exception. A private `fn` at module scope
is the sign the owning object hasn't been found. Put the behavior on
the data type being read or written, on a data-bearing helper
object, or on a trait implemented for the real object. If a
calculation only exists to support one method body, make it a small
private method on the same object — the local-helper carve-out from
`abstractions.md` does **not** apply in Rust; even a small private
helper goes inside an `impl` block.

For projection / conversion, reach for `impl From<X> for Y` instead
of a `fn project_x_to_y(...)`.

## Schema-generated objects are the method surface

In the schema-derived stack, the authored schema names the real
objects. The generator emits Rust types for those objects;
hand-written code attaches behavior to those generated types with
inherent methods or trait impls. Schema is the noun-source, Rust is
the verb-attachment, and the verb-attachment goes **on** the
schema-emitted type — not beside it. Workflow:

1. Change the schema.
2. Regenerate the Rust types and derives.
3. Write or adjust methods on the regenerated nouns.

Do not hand-write a parallel mirror of a generated type to get a
method surface. Do not add reusable free functions around generated
types because "the generated code has no method yet." The missing
method belongs on the generated type, or on a data-bearing runtime
type that owns the state being read or written.

This matters most for schema-emitted signal surfaces: `Input`,
`Output`, operation payloads, route/header types, codecs, and store
records are the nouns. A request being treated is a method on the
request, or on the engine/store object that owns the state. If the
method can't be placed cleanly, the schema or the runtime noun isn't
specific enough yet.

Upgrade and mail behavior follow the same rule. A changed generated
type implements the generated upgrade trait for the previous type;
an unchanged type carries no upgrade method. A sent signal root
creates a generated `MessageSent` event, and push hooks are methods
on that event. Nexus owns in-flight mail as `NexusMail<Payload>` and
emits `MessageProcessed<Reply>` after SEMA or execution produces a
reply. Do not create free `upgrade_*`, `send_*`, or `notify_*`
helpers beside generated types.

### Async mail flow is object flow

Async behavior does not justify free procedural glue. The signal
protocol's asynchronous lifecycle is carried by generated data types
and state-bearing actor objects:

- `Input` / `Output` — the Signal root message types.
- `MessageSent` — lifecycle event when Signal hands mail to Nexus.
- `NexusMail<Payload>` — mail currently owned by Nexus.
- `NexusInput` / `NexusOutput` — the Nexus execution language.
- `SemaInput` / `SemaOutput` — the SEMA state language.
- `MessageProcessed<Reply>` — lifecycle event after Nexus gets a
  SEMA or execution reply.

Methods attach to those objects, or to data-bearing runtime owners
such as `Engine`, `Mailbox`, `MailLedger`, `Nexus`, or `Store`.
Avoid module-level helpers named like `route_mail`, `process_mail`,
`dispatch_signal`, or `apply_sema`. The missing noun is usually
visible in such a helper's arguments: make that noun the receiver,
or create the state-bearing actor that owns the phase.

## No ZST method holders

A `pub struct Foo;` whose `impl Foo` is just a parking lot for
functions doing real work on data they don't carry is a free
function in namespace clothing — the methods-on-types rule evaded
one level deeper. Find the noun whose data the verb reads or writes;
invent it if it doesn't exist yet.

```rust
// Wrong — ZST as a folder for free functions
pub struct CertParser;

impl CertParser {
    pub fn parse_pem(pem: &str) -> Result<Cert, Error> { … }
    pub fn parse_der(der: &[u8]) -> Result<Cert, Error> { … }
    pub fn fingerprint(cert: &Cert) -> Hash { … }
}

// Right — the verbs belong on the noun whose data they touch
impl Cert {
    pub fn from_pem(pem: &str) -> Result<Self, Error> { … }
    pub fn from_der(der: &[u8]) -> Result<Self, Error> { … }
    pub fn fingerprint(&self) -> Hash { … }
}
```

If parsing genuinely needs its own state (a buffered lexer,
accumulated diagnostics, a configurable mode), the noun is
`CertParser` *with fields*. Either the work belongs on the data
type, or on a stateful parser type. The ZST middle ground is the
gap.

This applies to internal macro and parser code as much as public
APIs. A `RootMacro;` unit struct implementing a trait is acceptable
only if the type itself is doing type-level work. If it has runtime
behavior — a name, a delimiter it accepts, a position it lowers,
state it records — put that data in fields and make the methods read
those fields. Don't use a unit struct merely because a trait object
needs a concrete implementor.

### Legitimate ZST uses — narrow, named

ZSTs earn their keep when they carry **type-level information**
rather than pretending to carry runtime state:

- **`PhantomData<T>`** and other generic-parameter trackers.
- **Marker types required by external frameworks** — sealed-trait
  gates, or an `Iterator` impl on a unit struct that genuinely has
  no carried state. The ZST has *only* trait-impl methods that
  delegate to a data-bearing partner; never inherent methods doing
  real work. (For actors the runtime is Kameo, whose `Self IS the
  actor` shape removes the need for marker types — the actor type
  carries data fields and is the noun.)
- **Type-level enum variants** in trait-encoded state machines,
  where the unit struct *is* the state and the type system enforces
  transitions.

The test: does the ZST's job vanish if you erase its name from the
type system? If yes, it was a namespace and the verbs need a real
noun. If no (phantom parameter, marker, state position), the ZST is
fine.

## Typestate retires when borrow rules enforce its invariant

A typestate is valuable when the invariant it carries *cannot* be
expressed by Rust's existing borrow rules. When the invariant *can*
be expressed by `&mut self` exclusive borrow, the typestate is
redundant — its safety property already lives in the borrow checker.

Canonical example: a runtime holding a resource across a mutation
phase, modeled with a typestate carrier.

```rust
// Wrong — typestate that duplicates a borrow rule
struct Mail<Phase> { identifier: Identifier, phase: Phase }
struct BeingProcessed { input: ApplyInput }
struct Processed { output: ApplyOutput }

impl Mail<BeingProcessed> {
    fn run(self, engine: &mut Engine) -> Mail<Processed> {
        let output = engine.apply(self.phase.input);
        Mail { identifier: self.identifier, phase: Processed { output } }
    }
}
```

The intent is *"the engine holds the mail ⇒ it is being processed"*
as a compile-time fact: `Mail<Processed>` cannot exist without
consuming a `Mail<BeingProcessed>` through `run`. But once
`Engine::apply` is a trait method:

```rust
// Right — the trait surface carries the invariant
trait EngineApi {
    fn apply(&mut self, input: ApplyInput) -> ApplyOutput;
}
```

The `&mut self` exclusive borrow already enforces *"only one apply
at a time on this engine"*. The `Mail<Phase>` wrapper adds no safety
property the borrow checker doesn't already enforce; lifecycle
events fire inside the composer or as hook calls, and the type-level
"is being processed" is now decorative.

Retirement test:
- **Does removing the typestate lose any property `&mut self`
  doesn't enforce?** If no, it's redundant.
- **Does the trait method's signature already constrain ordering?**
  If yes, the typestate documents what the signature already pins.
- **Are the typestate's per-phase fields intermediate state that no
  longer needs naming?** If yes, inline them into the trait method's
  locals.

Typestate stays valuable when the invariant crosses borrow
boundaries the language can't see:
- **Async lifecycle phases across `.await`** — the borrow checker
  doesn't track resource ownership across suspension; the typestate
  does.
- **Durability transitions across syscalls** — after `fsync` the
  data is durable; before, it isn't. No borrow rule captures this.
- **Cross-thread state machines** — when ownership transfers via
  channel, the typestate documents the receiver's phase.

When introducing a typestate, identify the invariant it carries and
check whether `&mut self`, `&self`, move semantics, or lifetime
bounds already enforce it. If yes, drop the typestate; the trait
surface is the honest representation.

## Domain values are types, not primitives

If a value has identity beyond its bits, it gets a newtype. A
content hash is not a `String`. A node name is not a `String`. A
file path used as an identifier is not a `Path`.

```rust
// Wrong
pub fn details(&self, md5: &str) -> Result<Item, Error> { … }

// Right
pub struct Md5([u8; 16]);
pub fn details(&self, md5: &Md5) -> Result<Item, Error> { … }
```

**The wrapped field is private.** A `pub` field exposes the
primitive and defeats every reason to wrap it: callers construct
unchecked values and read the raw bytes back out.

```rust
// Wrong — pub field, the type is just a label
pub struct NodeName(pub String);

// Right — field private; construction and access go through methods
pub struct NodeName(String);

impl NodeName {
    pub fn new(s: impl Into<String>) -> Self { Self(s.into()) }   // or TryFrom if validated
}

impl AsRef<str> for NodeName {
    fn as_ref(&self) -> &str { &self.0 }
}
```

Construction with validation goes through `TryFrom<&str>` (or
`FromStr`) returning the crate's `Error`.

## One type per concept — no `-Details` / `-Info` companions

If you find yourself defining `Item` *and* `ItemDetails`, stop. The
`-Details` / `-Info` suffix paired with a base type is one concept
fragmented across two types because the base was designed too thin.
Fix the base type. The same applies to `-Extra`, `-Meta`, `-Full`,
`-Extended`, `-Raw`/`-Parsed` pairs, and any suffix meaning "the
real version of the thing next door."

```rust
// Wrong — two types for one concept
struct Item { md5: Md5, name: String }
struct ItemDetails { md5: Md5, name: String, size: u64, mirrors: Vec<Url>, … }

// Right — one Item, complete
struct Item {
    md5: Md5,
    name: String,
    size: u64,
    mirrors: Vec<Url>,
    …
}
```

If different *call sites* genuinely need different *projections*,
model that with a method returning a smaller view (`item.summary()`),
not a parallel type.

## Don't hide typification in strings

When a value has a typed identity, **the type system carries the
discrimination**. Don't reach for `starts_with(...)`,
`contains(...)`, or `match s.as_str()` to recover information the
type already encodes.

### Wrong: verifying type by string prefix in tests

```rust
// the field's type is already MessageId — the assertion adds nothing
assert!(messages[0].id.as_str().starts_with("m-"));
assert_eq!(messages[0].id.as_str().len(), 9);
```

`Vec<Message>::id: MessageId` already proves the kind. If the same
field can carry several kinds, that's a missing sum-type — not a
string-prefix discriminator.

### Wrong: dispatching on string prefix at runtime

```rust
fn route(id: &Id) -> Handler {
    if id.as_str().starts_with("m-") { handle_message }
    else if id.as_str().starts_with("d-") { handle_delivery }
    else if id.as_str().starts_with("a-") { handle_authorization }
    else { panic!("unknown id kind") }
}
```

That's a closed enum with extra steps. Use one:

```rust
pub enum Id {
    Message(MessageId),
    Delivery(DeliveryId),
    Authorization(AuthorizationId),
}

fn route(id: &Id) -> Handler {
    match id {
        Id::Message(_)       => handle_message,
        Id::Delivery(_)      => handle_delivery,
        Id::Authorization(_) => handle_authorization,
    }
}
```

### The system mints identity, not the agent

Even when a string ID's discriminator is type-correct, an
agent-minted prefix-encoded ID is the wrong shape because the agent
shouldn't be minting identity at all.

```rust
// Wrong — agent invents an ID
let id = format!("m-{}-{:03}", today_iso8601(), counter.next());
store.send(Message { id, sender, recipient, body }).await?;
```

The agent does clock work, maintains counter state, packs typed
values into stringly-typed form, and produces an opaque key parallel
to the slot the store assigns anyway.

```rust
// Right — the store assigns Slot<T>
let slot = store.assert(Message { recipient, body }).await?;   // returns Slot<Message>
```

The same shape applies when the agent supplies its own sender or
timestamps:

```rust
// Wrong — sender on the record body (already on the auth proof)
store.assert(Message { sender: my_principal, recipient, body }).await?;

// Wrong — commit time as a record field (transition log already stamps it)
store.assert(HarnessObservation {
    subject,
    state,
    observed_at: Utc::now().to_rfc3339(),    // string, agent-minted
}).await?;

// Right — agent supplies only content; infrastructure stamps the rest
store.assert(Message { recipient, body }).await?;
store.assert(HarnessObservation { subject, state }).await?;
```

The unifying test: ***could the system supply this value without
asking the agent?*** If yes, the agent must not supply it. Identity,
commit time, sender principal — all infrastructure context. The wire
carries only what only the sender knows.

*Content* timestamps (a `Deadline`'s expiration, a scheduled
message's send-at) are different — the agent genuinely supplies
those, and they appear as a typed `Timestamp` (a bare integer,
nanos since epoch, not a string).

Once you have the typed identity, **use it** — don't drop back to
string operations to recover what the type already proved.

## One object in, one object out

Method signatures take at most one explicit object argument and
return exactly one object. When inputs or outputs need more, define
a struct.

**Anonymous tuples are not used at type boundaries** — not as return
types, parameter types, struct fields, or type aliases. The
exception is **tuple newtypes**: `struct Md5([u8; 16])`, `struct
NodeName(String)` — tuple syntax wrapping a single thing, but the
wrapper is a named type. Local destructuring (`let (a, b) = pair;`)
against a tuple-newtype's inner is fine; the rule is about
type-level appearances of unnamed tuples.

The verb is the method name; the noun is the type. Don't smuggle the
verb into the type name (`DownloadRequest` + `download_url(req)`) —
make it a method on the input (`Request::download`).

```rust
// Wrong — multi-primitive args at the boundary
fn download_url(&self, md5: &str, path_index: Option<u32>,
                domain_index: Option<u32>) -> Result<Download, Error> { … }

// Wrong — free function with tuple return
fn parse_results(html: &str) -> Result<(Vec<SearchResult>, bool), Error> { … }

// Right — input is a Request; the verb is a method on it
struct Request { md5: Md5, path_index: Option<u32>, domain_index: Option<u32> }

impl Request {
    pub fn download(&self) -> Result<Download, Error> { … }
}

// Right — one explicit object alongside self (relational operation)
impl Tree {
    pub fn merge(&self, other: Tree) -> Result<Tree, Error> { … }
}
```

`self` is implicit; the rule counts explicit arguments only. A
method takes zero or one typed object alongside `self`.

## Constructors are associated functions

`new`, `with_*`, `from_*`, `build` — never module-level free
functions.

| Name           | Use when                                                       |
|----------------|----------------------------------------------------------------|
| `new`          | default / minimal construction.                                |
| `with_<thing>` | ergonomic alt with one extra knob (`Tree::with_bits`).         |
| `from_<src>`   | conversion from a specific source type or representation.      |
| `from_input`   | conversion from a typed input struct (single-object-in style). |
| `build`        | multi-step construction with clearly-named primitive args.     |
| `Default`      | when "empty / zero" is meaningful for the type.                |
| `From<T>`      | infallible conversion from another type.                       |
| `TryFrom<T>`   | fallible conversion. Pair with `Error` enum.                   |

Prefer `TryFrom` when the conversion has one canonical source type;
prefer `from_<src>(…) -> Result<Self, Error>` when there are several
plausible sources or extra args.

## Use existing trait domains

If `core::str::FromStr` already names what you do, implement
`FromStr`, not an inherent `parse`. Same for `Display`, `From`,
`TryFrom`, `AsRef`, `Default`, `Iterator`. Inherent methods that
bypass an obvious trait domain are a smell.

```rust
use core::str::FromStr;

impl FromStr for Message {
    type Err = MessageParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> { … }
}
```

## Direction-encoded names

Prefer `from_*`, `to_*`, `into_*`, `as_*`. Avoid `read`, `write`,
`load`, `save` when a direction word already conveys the meaning.
`as_str` over `get_string`. `to_hex` over `format_hex`.
`from_bytes` over `parse_bytes`.

`get` / `put` are fine for storage interfaces (`ChunkStore::get`) —
they name the storage operation, not a conversion.

## See also

- `skills/abstractions.md` — cross-language methods-on-types rule.
- `skills/rust/errors.md` — typed `Error` enum per crate.
- `skills/naming.md` — full English words, no category suffixes.
