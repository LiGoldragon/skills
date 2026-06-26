# Skill — Structural Forms

## The idea

`if`, a function call, a struct, generics — every construct of a language is a
*shape* the compiler recognizes. Conventional languages keep that set of shapes
frozen and poorly-saved inside the compiler. Here the set is kept as **data**:
typed `#[shape(...)]` node definitions, recognized by form, recursively. That
makes the language open (a new construct is new data, not a compiler change) and
far easier for an LLM to read, write, and reason about. Per Spirit `7c71`
(Principle, VeryHigh): [a language is a set of structural macros; keep the set as
data so it is open and LLM-legible]; `2zed`: [everything is data conforming to a
schema-defined type; a macro is a value of a struct]; `my86`: [the file is a
typed tree of nodes; the grammar is the type, with no hand-written parser].

Only a **tiny seed** stays hand-written: the NOTA block parser plus one derive.
Everything above the seed is data or derived. (Concept: report `627`.)

## The shape vocabulary (the nota-next derive)

`#[derive(StructuralMacroNode)]` turns a `#[shape(...)]`-tagged type into a
matcher + decoder + encoder. At the nota-next HEAD it covers **enums only** —
each variant is a shape; the derive rejects structs and unions
(`StructuralMacroNode supports enums only`). The seven shapes:

| `#[shape(...)]` | matches | fields |
|---|---|---|
| `pascal_atom` | a PascalCase atom `Topic` | 1 |
| `keyword = "X"` | the literal atom `X` | 0 |
| `head = "H", arity = N` | `(H a …)` fixed count | N-1 |
| `head = "H", atom` | `(H 32)` head + one atom (via `FromStr`) | 1 |
| `head = "H", body` | `(H a b …)` any count | 1 (a body) |
| `pascal_head, arity = N` | `(Foo a …)` captured head, fixed | N |
| `pascal_head, body` | `(Foo a b …)` captured head, any count | 2 (head + body) |

`head` takes exactly one of `arity`/`body`/`atom`; `pascal_head` exactly one of
`arity`/`body`. Enum variants carry **positional (unnamed/tuple) fields**, mapped
to shape captures by declaration order; the HEAD derive rejects named-field
variants (`variants carry unnamed fields, not named fields`). Schema
**declaration bodies** — plain structs, and the `Family`/`Stream` frames below —
are resolved by schema-next's own source codec during lowering, not by this
derive. (Named-field structural variants and a struct-level derive are *tracked
but not landed* at the nota-next HEAD — tasks #411/#416 reference them; reconcile
the tracker against the code before relying on either.)

## Positional struct syntax (Spirit `adnn`)

Per Spirit `adnn` (Decision): [schema struct declarations are positional lists of
types, mirroring how the data reads in NOTA, not name-value pairs; the field name
is derived from the type by default; a field needing an explicit name uses the
dot-prefix differentiator `key.TypeReference`; the name-value form and the `*`
shorthand are retired].

```
Entry { Topics Kind Description }              ;; bare type — role derived from type
ImportDeclaration { Name source.TypeReference } ;; dot — explicit role on a plain reference
Query { (Topics (Vector Topic)) (Limit (Optional Integer)) } ;; paren — explicit role on a composite
Detail { Field { String } }                    ;; PascalCase + block = inline decl
```

A struct-field spec is one of three role forms plus the inline-declaration form: a
**bare PascalCase type** derives its role from the type (`Topics`); a **dot-prefixed**
`role.TypeReference` gives an explicit role to a *plain* reference
(`source.TypeReference`); a **parenthesized** `(role compositeType)` gives an explicit
role to a *composite* reference (`(Topics (Vector Topic))`) — a composite cannot take
the bare or dot form, so it is wrapped. The retired `field Type` name-value form and
`*` shorthand are **rejected loudly** (`SchemaError::RetiredStructFieldSyntax`): a bare
struct-field atom must name a type (PascalCase or scoped). A *redundant* explicit role
whose name equals the type-derived name is also rejected
(`SchemaError::RedundantExplicitFieldRole`): `topic.Topic` must be the bare `Topic`
(closing intent `i3p0`). All three behaviors are on **schema-next main** as of
2026-06-18: `af3705c` (retired-pair reject), `95f1ee7` (redundant-role reject), and
`1de72dde` (explicit structural field roles — the parenthesized composite form). The
positional/explicit-role body is now the *only* accepted struct form.

## The dimensional principle (Spirit `ov30`)

Per Spirit `ov30` (Principle, High): [a struct field's role is its type, so no
struct has two fields of the same type; distinct roles are distinct types
(dimensional correctness — Height and Width are both metres yet cannot be
interchanged or multiplied as alike); repetition is a keyed collection (a
`Vector` is a `Map` over an ordinal index), never repeated fields; field-name
equals type-name is the default, and an explicit field name signals a missing
type or a collection]. This is the [newtype per domain value] rule
(`skills/abstractions.md`) pushed to its endpoint: **a newtype per role**. Full
type-level enforcement is aspirational; the principle guides design now. (Report
`639`.)

## Streams and families are positional special forms, not structs

Streams and families are **not structs**: they are closed typed records
with fixed heterogeneous slots, declared as a frame application over a
**labeled-brace body** — `{ fieldname value … }` — because the slots are
heterogeneous-by-role and the role names carry meaning position alone
does not:

```nota
EntryFamily (Family { record Entry table entries key Domain })
ObservationFamily (Family { record Observation table observations key Identified })
IntentEventStream (Stream { token SubscriptionToken opened SubscriptionStarted event IntentEvent close SubscriptionToken })
```

For `Family` the fields are `record` (the record type), `table` (the
lowercase table literal), and `key` (the key kind — only `Domain` or
`Identified`). For `Stream` the `token`/`opened`/`event`/`close` fields
are distinct roles even when two share a concrete type
(`SubscriptionToken` here), so the brace body names them rather than
relying on position. (Reports `645`, `647`, `649`. The family-body reader
is still hand-parsed above the seed — a tracked `v0n6` cleanup, not a
syntax question.)

## Pipe delimiters: generics and traits/impls

The pipe delimiter family is assigned as schema-level structural syntax:

```nota
[| text |]             ;; bracket-safe / multiline string
Name (| [T] body |)    ;; generic declaration; params scope the nested body
{| Trait Target |}     ;; trait/impl structural form, simplest marker shape
```

Generic **use** stays ordinary application: `(Head Arg ...)`. A use site
does not take `{| |}` just to say it is generic; the declaration makes
`Head` a known generic, the same way built-in `(Vector T)` is understood.
This keeps `{| |}` assigned to traits/impls.

An impl is one pipe-brace object, never a map key/value split. Anything
scoped by binders must live inside the same structural object. The
matcher may structurally sugar the optional ends:

```nota
{| Trait Target |}                    ;; marker impl, non-generic
{| [T] Trait (Target T) |}             ;; marker impl, generic
{| Trait Target [ (deref ...) ] |}     ;; method-bearing impl, non-generic
{| [T] Trait (Target T) [ (f ...) ] |} ;; method-bearing impl, generic
```

The leading parameter list is optional and recognized by square-bracket
shape before the trait. The trailing body list is optional and recognized
after the target. A marker trait may have no body. A method-bearing impl
must carry its function bodies as data; there is no ad hoc `method`
keyword. Function/signature forms are their own construct and should be
designed explicitly when that layer is implemented.

## The self-host boundary

The derive automates the *shape* layer. Meaning stays hand-written: a
`TypeReference`'s top-level decode is registry- and context-aware (declared
macros, inline declarations) — a permanent border, not a seed to shrink.
`TypeReference` has exactly one codec (the structural-macro grammar: built-in
head fast path + the `pascal_head, body` `ApplicationNode` seam + the
`head="Bytes", atom` `HeadedAtom` seam); `NotaDecode`/`NotaEncode` are thin
delegators. The honest rule: keep the seed small; each remaining hand-written
seam is shape↔meaning boundary, not debt. (Reports `631`, `635`.)

## See also

- Concept + naming: `reports/designer/627`. Dimensional principle:
  `639`. Positional syntax: `640`/`643`. Streams/families: `645`/`649`.
  Generic and pipe-delimiter assignment: `655`/`658`.
- Intents: `7c71` `2zed` `my86` `wqdi` (thesis); `ov30` (dimensional principle);
  `adnn` (positional syntax).
- `skills/abstractions.md` (newtype-per-role endpoint), `skills/nota-design.md`
  (NOTA records; defers struct-body grammar here), `skills/component-triad.md`.
- Code: nota-next `derive/src/lib.rs`, schema-next `src/declarative.rs` +
  `src/source.rs` — **landed on schema-next main** (`af3705c` `RetiredStructFieldSyntax`,
  `95f1ee7` `RedundantExplicitFieldRole`, `1de72dde` explicit structural field roles),
  no longer the `next/structural-forms` epic branch.
