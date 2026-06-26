# Skill — verb belongs to noun (where behavior lives)

Every reusable verb belongs to a noun. If you can't name the noun,
the model isn't formed yet — keep looking until you can.

## The rule

Before you write a verb (a function, a method, a dispatcher), ask:
what type owns this verb? If a type exists, attach the verb as a
method. If no obvious noun exists, the model is incomplete — the
missing type is what the verb is asking you to declare.

Reusable behavior lives on a type. Free functions are for things
that genuinely belong nowhere else: a binary's `main`, a small
private helper inside one module, a pure relational operation
between values of equal status.

A free `parse_query(text: &str) -> Result<QueryOp, Error>` is a verb
floating without a type. The `text` parameter is the input the verb
wants; the verb is the affordance the *parser-state type* should own.

```rust
// Right — verb on the type that owns it
struct QueryParser<'input> { lexer: Lexer<'input> }

impl<'input> QueryParser<'input> {
    pub fn new(input: &'input str) -> Self { … }
    pub fn into_query(self) -> Result<QueryOp, Error> { … }
}
```

This applies to any language with method dispatch (Rust, Python,
Go, Java, C++, Smalltalk) and is enforced by convention in
languages without it (C's `_operations` vtables, Haskell's
typeclass-constrained free functions). The discipline is universal
even when the syntax varies.

## The forcing function

The rule's purpose is not what it makes you write; it's what it
makes you do *before* you write. It forces the question — *what
type owns this verb?* — and sometimes the answer is "no type exists
yet," which forces you to invent one. That forced invention is the
load-bearing cognitive event.

Without the rule, the verb gets written as a free function and the
noun never appears. The model develops gaps: verbs without owning
nouns, missing structural types, behavior smeared across the call
graph. Programs that "look fine" end up missing whole structural
types they ought to have.

The pattern is named in the refactoring catalogue. Martin Fowler:
**Feature Envy** is a method more interested in another class than
its own — a verb in the wrong place. **Data Class** is the same
drift from the other side — a type with no behavior because the
verbs that should live on it ended up elsewhere. **Anemic Domain
Model** is the codebase-scale form. The cure for all three: *Move
Function* / *Extract Class* — find the type, attach the verb. Do
this once, up front, instead of accumulating the debt.

## Affordances vs operations

Methods encode **affordances** — what a value of this type *can do*.
Free functions encode **operations** that happen to take arguments.
The distinction is structural.

Fruits can be eaten and clouds cannot. Code that models the world
correctly says `fruit.eat()`, not `eat(fruit)`. The method form
binds the verb to the type that owns it; the free-function form lets
the verb float, and `eat(cloud)` becomes thinkable. The vocabulary
is Gibson's: an *affordance* is what the environment offers an agent
— a property of the relationship between object and agent (a door
handle affords pulling). A method-bearing type *advertises* its
affordances at every call site; a passive record next to a
free-function library does not.

## Why this matters more for LLM agents

Humans procrastinate creating types because typing `struct
QueryParser { … }` *feels heavier* than `fn parse_query(…)`. That
tactile friction is a feature: it makes humans ask "is this type
pulling its weight?" before paying the cost.

LLMs have no such friction. Generating `struct QueryParser` and `fn
parse_query` cost the same tokens and produce no felt sense of
weight, so LLMs default to whichever shape is *shorter* — almost
always the free function. The rule reintroduces, by fiat, the
friction the substrate has erased. It changes what the agent can
think by changing what it is *required* to write.

The underlying failure is **verbs without owning nouns**: naming
conventions go bad because there is no type to anchor a name to;
unused code accumulates because nothing carries a clean
responsibility.

## The naming bridge

Phil Karlton: "There are only two hard things in Computer Science:
cache invalidation and naming things." When an agent skips creating
a type, it skips the naming step entirely — the hard thing is not
avoided, it is hidden. The rule exists to make sure naming happens.

## Principled exceptions

Carve-outs, named directly. Use them honestly; they are not a back
door for skipping noun-creation.

**Local helper.** A small private helper inside one module is fine
when genuinely local — a three-line `fn hex(h: &Hash) -> String`
next to a single `Display` impl is a private fragment of one impl,
not a missing noun. The rule kicks in when the verb is *reusable*:
more than one caller might want it, it would be discoverable from
multiple sites, or its life as a free function would let it spread.

**Relational operation.** Some operations are genuinely relational
between two values of equal status with no state on either side —
`add(a, b)` over two numbers. In method-bearing languages this is
expressed via operator overloading: `a + b` desugars to `Add::add(a,
b)`, which IS a method on a type. The rule is preserved.

**Standard library.** Names inherited from well-known libraries keep
their shape. `serde_json::from_str` / `to_string` are free functions
because ecosystem convention demands them; hiding that behind methods
would surprise every user. The carve-out is narrow — only the
crate-root `from_str` / `to_string` shape; everything inside the
crate's own implementation still attaches behavior to its owning
types. Don't invent gratuitous deviations from convention, but don't
let "convention" excuse missing types.

**No methods in the language.** The rule still applies. C follows it
via vtables (`struct file_operations`, `struct inode_operations` in
the Linux kernel) — behavior attached to the type, dispatch manual.
Haskell follows it via typeclass-constrained free functions (`Eq a
=> a -> a -> Bool` is conceptually a method on `a`). Python via
`class … def …`.

**Actor frameworks.** Some frameworks force a behavior-marker ZST
satisfying a trait shape plus a separate `State` type carrying the
data; verbs then drift onto `State`, leaving the named noun empty.
Kameo (the workspace runtime) avoids this: `Self` IS the actor and
carries fields directly. The actor type is the noun — fields,
construction, methods, and `Message<T>` impls all on the same noun,
no separate marker, no separate `State`. See `skills/kameo.md`.

## Find the noun — what it looks like

When "what type owns this verb?" is hard, that hardness signals the
model isn't fully formed. Three resolutions:

1. **The noun already exists.** You missed it. Attach the method.
2. **The noun is implicit but unnamed.** A `parse_query` free
   function already has a `QueryParser` inside it: parser state,
   cursor, error context. Name it — make the implicit explicit.
3. **The verb is genuinely relational.** Two values of equal
   status, no privileged owner. Use the relational carve-out.

If none apply, you don't have a clean model yet. Slow down; don't
paper over the gap with a free function.

## The wrong-noun trap

The verb belongs not just to *a* noun but to **the right** noun —
the one whose primary concern matches the verb's concern. Picking a
nearby noun "because it's already there and might as well own this
too" is a failure the rule's surface form misses. Adjacency of
*types* is not adjacency of *concerns*.

Example — two proc-macro crates sitting close: `text-codec-derive`
(concern: text encode/decode) and `schema-derive` (concern: schema
introspection over record types). Both touch the same record types.
The temptation is to put schema introspection into
`text-codec-derive` "because it already sees the types." That puts
the verb on the wrong noun: schema introspection is the *schema's*
concern, and the codec is downstream of the schema. The right noun
is `schema-derive`.

Diagnostic: if the answer sounds like *"this nearby type **could**
hold it,"* slow down. The merely-convenient noun has all the
maintainability problems of putting the verb on no type at all, plus
it actively hides the missing proper noun. When two crates / types /
modules have similar surface but different concerns, the verb goes
with the concern, not the surface. Same discipline at the crate
boundary lives in `skills/micro-components.md`.

## Schema-emitted nouns

When the schema-derived stack is in play, the nouns come from the
schema. Authoring a `.schema` file declares the types; `schema-rust-next`
emits the Rust declarations + codec impls + dispatch tables. The
agent's Rust code attaches **methods** to those emitted nouns.

| Layer | Provides |
|---|---|
| `.schema` file (authored) | Data objects + traits (implied by signal/nexus/SEMA interaction) |
| Emitted Rust (machine-written) | Type declarations + codec impls + headers + dispatch tables |
| Agent-written Rust (methods) | Behavior on the schema-emitted objects |

The forcing function applies sharply: the noun is almost always
already named by the schema. If you write a free function whose
arguments include schema-emitted types, the verb belongs as a method
on whichever emitted type is the primary subject.

```rust
// Right — verb on the schema-emitted noun
impl Engine {
    pub fn handle(&self, input: Input) -> Output { match input { ... } }
}
// Wrong — free function with schema-emitted types as arguments
// fn dispatch(engine: &Engine, input: Input) -> Output { ... }
```

Corollary: don't hand-edit generated data-type mirrors. To change a
data type, edit the `.schema` and regenerate; methods written against
the previous emission either compile against the new shape (good) or
surface their assumptions as compile errors (also good). The runtime
triad (Signal's Operation, Nexus's Action / Response, SEMA's stored
archive types) is all emitted; methods attach to whichever emitted
noun is the primary subject. See `skills/component-triad.md`.

## Companion disciplines

Four rules push the same direction — **the type system is the
model**:

- **Wrapped field is private.** A newtype wraps a primitive to give
  it identity; if the field is `pub` (`Slot(pub u64)`), callers
  construct unchecked values and read raw bytes back out, defeating
  the wrap. The type owns its representation.
- **Perfect specificity.** Every typed boundary names exactly what
  flows through it — no wrapper enums mixing concerns, no
  string-tagged dispatch, no generic-record fallback. The type
  carries the meaning, not stringly-typed metadata.
- **Engine logic = enum-vs-enum cross-product.** When two enums meet
  under `match`, the cross-product of their variants is the typed
  relationship — make it explicit, as a nested match or a named
  trait (`Reaches<Right>`, `Contact<Other>`). The contact point
  between two structured inputs IS a noun; naming it surfaces logic
  that would otherwise scatter into `if` chains and sentinel
  booleans. See `skills/enum-contact-points.md`.
- **Newtype per role.** A field's role *is* its type, so no struct ever
  has two fields of the same type — distinct roles are distinct types
  (dimensional correctness: `Height` and `Width` are both metres but not
  interchangeable), and repetition is a keyed collection, never repeated
  fields. This is "newtype per domain value" pushed to its endpoint: a
  newtype per role. See `skills/structural-forms.md` (Spirit `ov30`).

## See also

- `skills/naming.md` — full English words; the forced naming step.
- `skills/rust-discipline.md` — Rust enforcement (no ZST method
  holders, domain newtypes, one-object in/out).
- `skills/micro-components.md` — same discipline at the crate
  boundary.
