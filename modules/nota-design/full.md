# Skill — NOTA design

Read this before designing a new NOTA file or schema. Designed badly, NOTA becomes JSON with extra steps — verbose, with data hiding in comments or restated as identical wrappers around every record. Designed well, the same data takes a third of the tokens and the structure itself carries category information that would otherwise be a side channel.

## Rule 1 — If there's no variant, it's a struct (no tag)

A PascalCase tag at the start of `(…)` is an enum variant. If every record in the file would carry the same tag, there is no enum — it's a struct, and structs have **no tag** in the wire form. Drop it.

`manifests/active-outputs.nota` has actual variants — `Skill` and `Role` — so
each record IS an enum variant and the tag tells the reader which kind of
generated output this is. A deployment plan with one kind of step drops the
tag: `(zeus apply 2026-05-19)`. If steps vary (`Build`, `Verify`, `Deploy`),
the variant tags appear.

The test: *can the same position carry more than one shape?* If yes, you have an enum and the variant tag names which shape this record is. If no, you have a struct and write the fields directly with no tag.

## Rule 2 — Data lives in records, not in comments

NOTA comments explain the schema — what fields mean, what variants exist, the structural contract. They do NOT organize data into categories or sections. A `;; Roles` header introducing three role records is a category surfaced as a comment: NOTA can't see it and nobody can grep it. The category IS data — make it the type.

Bad:

```nota
;; Skills
(Output component-triad component-triad Skill Architecture Apex [...])
(Output structural-forms structural-forms Skill Architecture Apex [...])

;; Roles
(Output scout role-scout Role [...] [...])
```

Two faults: every record wears a redundant `Output` wrapper, and the output
kind is duplicated as both comment and field. Fix both — the variant tag IS the
kind:

```nota
(Skill (component-triad component-triad Architecture Apex [...] [AgentsSkill ClaudeSkill]))
(Skill (structural-forms structural-forms Architecture Apex [...] [AgentsSkill ClaudeSkill]))

(Role (scout role-scout [agent-output-protocol] [...] [ClaudeAgent CodexAgent PiAgent]))
```

Same data, fewer tokens, grep-able kind. Instance names (`scout`,
`component-triad`) are lowercase/kebab-case because they're runtime instances;
the tier value (`Apex`) and target surfaces (`AgentsSkill`, `CodexAgent`) are
PascalCase because they're compile-time enum variants.

## Rule 3 — Enums get PascalCase names, not numbers

Integer codes for enum variants are a smell. `tier 1` / `tier 2` means nothing without a key; `tier Apex` / `tier Keystroke` / `tier Topic` is self-documenting and grep-able from a cold read.

Bad: `(Skill (component-triad component-triad Architecture 1 [...] [AgentsSkill]))`.
Good: `(Skill (component-triad component-triad Architecture Apex [...] [AgentsSkill]))`.

Variants are **PascalCase** because they're compile-time structural (PascalCase = compile-time structural; camelCase = instance). The parser dispatches on first-character case — a lowercase `apex` parses as an instance identifier, not a variant. Numbers are fine for actual numbers (counts, identifiers, slots, ordinals where ordering matters); not as stand-ins for named categorical distinctions. Grep for `Apex` finds every apex-tier record across every NOTA file that shares the vocabulary.

## Rule 4 — Enum payloads are choices; structs are products

When an enum variant carries data, the payload's shape follows what the data IS:

- **One axis of choice** → direct enum payload. `(Busy BusyReason)` where `BusyReason [DatabaseOverloaded ResourceDisconnected OtherBusyReason]`. Not `(Busy BusyReport)` wrapping the choice in a struct that adds nothing.
- **Product of independent facts** → struct payload. `(RecordAccepted SemaReceipt)` where `SemaReceipt { RecordIdentifier * DatabaseMarker * }` — multiple facts the reply carries together.
- **Only some choices need extra data** → nested data-carrying enum. `BusyReason [(DatabaseOverloaded RetryGuidance) ResourceDisconnected OtherBusyReason]` — guidance attaches to the variant that needs it.

Wrong shape: inventing a `<Variant>Report` struct wrapper around a single enum. The semantic root is the variant; the choice axis is the payload enum; no wrapper. The notation must truthfully represent the data shape — empty wrappers are a smell.

### Schema enum sugar

Prefer the shortest schema spelling that still names the reusable type at the
right level. Root input/output headers should usually name operation objects,
with payload aliases or inline operation bodies declared one level below.

When the variant name and payload type are the same, write the self-tagged
signature:

```schema
CommandSemaWrite [(Record) (Remove) (ChangeCertainty)]
```

Use the explicit `(Variant PayloadType)` spelling only when the two names
intentionally differ, such as `(Rejected SignalRejection)`.

Root headers may list exported payload aliases directly:

```schema
[Record Observe]
[RecordAccepted RecordsObserved]
{
  Record Entry
  Observe Query
  RecordAccepted SemaReceipt
  RecordsObserved RecordSet
}
```

The payload can also be declared inline at the root position when the
operation-owned shape is shallow:

```schema
[(Record { Topic { String } Description { String } })]
[]
{}
```

Direct fields in a root inline payload export their PascalCase field
declarations — `Topic { String }` declares `Topic` (a newtype over `String`)
and uses it as a field. Later inline payloads and the trailing namespace may
reuse those types by **bare reference** — a positional `Topic` or `Description`.
Duplicate declarations are an error; do not declare `Topic` again in the
namespace after introducing it inline.

A schema **struct body** is a *positional list of types*: each object is one
field whose name is derived from its type (a bare `Topic` is the field `topic`).
A field whose name must differ from its type uses the dot differentiator
`count.Integer`. The earlier `*` shorthand and the `field Type` name-value form
are **retired and rejected** (`SchemaError::RetiredStructFieldSyntax`) — see
`skills/structural-forms.md` for the full struct/field grammar. (This applies to
schema struct bodies only; the namespace map and NOTA wire records below are
unaffected.)

Nested payload enums can be declared at the variant position:

```schema
Output [
  RecordAccepted
  RecordsObserved
  (Busy [DatabaseOverloaded ResourceDisconnected OtherBusyReason])
  Rejected
]
```

The header stays a homogeneous vector of variant-signature objects:
`RecordAccepted` and `(Busy [...])` and `Rejected` are each signatures. The
inline bracket body declares the payload enum locally instead of forcing a
separate namespace declaration. The lowered form is equivalent to:

```schema
Output [RecordAccepted RecordsObserved (Busy BusyReason) Rejected]
BusyReason [DatabaseOverloaded ResourceDisconnected OtherBusyReason]
```

### Type-table variant resolution

The header can list variant names without spelling whether each is unit or data-carrying; the schema reader resolves against the local type table:

```schema
Output [RecordAccepted RecordsObserved Busy Rejected]
```

If `RecordAccepted` is a declared type, the variant carries it; if the name is not a declared type, it is a unit variant. The explicit `(Variant PayloadType)` form remains available when the variant name differs from the payload type name (e.g. `(Rejected SignalRejection)`). Same-name resolution defaults to data-carrying when a type exists.

## Rule 5 — A comfortable shorthand is a terser sibling variant; options are a vector of option-variants

NOTA forbids tail-omission — every schema position is present in the text — so you **cannot** make an interface "comfortable" by leaving fields off or sprinkling labeled optional settings. The two NOTA-idiomatic ways to make an authoring surface terse:

- **Shorthand = a terser sibling variant the consumer lowers.** When the common case wants fewer fields, add a *second variant* for it, never an under-filled struct. The deployed precedent is `TestRequest [(Run TestRun) (Check QuickCheck)]` — `Check` is the terse variant and the daemon's `lower` expands it to the full `Run`. So `Hermetic` (bare) and `(HermeticVm HermeticVmProfile)` are two `ContainedTarget` variants: the bare one carries defaults, the full one exposes the knobs. A shorthand is *variant selection*, never field omission.

- **Optional settings = a `(Vec OptionEnum)` of option-variants.** When a thing has many independently-optional knobs, model them as an enum of option-variants collected in a vector, not as labeled struct fields. `(MaximumGuests 3)` and `(NetworkIsolation TapLayer3)` are *variants* of an option enum; an empty `[]` means all-defaults; order is free; each present option is one typed variant. This is homogeneous at the schema level — every element is the same option enum — even though it looks varied.

The recurring mistake (caught repeatedly, including by the psyche on the lojix test-authoring surface): treating `(MaximumGuests 3)` / `(Lease 900)` as omittable *named fields* of a struct — the `(key value)` shape NOTA forbids. They are option-enum variants in a vector, or distinct sibling variants. If you want "easy to use," design the variant ladder and the option enum; do not invent labeled optionals. Ease-of-use in NOTA is achieved by the *type design*, not by bending the syntax.

## The canonical example

The skills repo manifests are stable examples of NOTA designed well. Open
`manifests/active-outputs.nota` before designing a new file with multiple
record kinds. Notice: `Skill` and `Role` are real variants with different
payloads; tier and target values are PascalCase variants (`Apex`, `Keystroke`,
`AgentsSkill`, `CodexAgent`); comments only explain the schema. Open
`manifests/module-dependencies.nota` when you need the contrasting
single-record-shape example: each dependency record is a struct in a homogeneous
vector, so it has no per-record wrapper tag.

## Grammar facts that catch the recurring mistakes

These are the language's actual grammar, not design rules. The source of truth is `nota/README.md`; restated here so the discipline skill carries the load.

### The mental model — three cases for PascalCase, one for the rest

Every PascalCase token falls into one of three cases:

1. `(VariantName fields…)` — **data-carrying enum variant**. An opening `(` immediately followed by a PascalCase token means you're at an enum variant carrying data; everything after the name is its positional fields.
2. `(fields…)` without a leading PascalCase token — **struct**. No tag; the schema position says what struct this is.
3. Bare `VariantName` with no preceding `(` — **unit variant** (no payload). Like `None`, `Maximum`, `Apex`.

Everything else is a primitive (strings, numbers, bools, bytes), a sequence `[…]` which is `Vec<T>` (every element the same schema type), or a map `{…}` which is a flat key/value stream.

The corollary: when you write a record, ask *can this position hold more than one shape?* If yes, it's an enum — tag the variant (case 1) or write a unit (case 3). If no, it's a struct — write fields directly with no tag (case 2). Structs are untagged, enum variants own PascalCase tags, map keys are key text by delimiter position.

### Strings are bare until they need delimiters

Bare atoms are the canonical string form whenever the content can be scanned
without delimiters. A bare string atom may use broad punctuation (`@`, `*`,
`&`, `^`, `%`, `<`, `>`, `:`, `/`, and a single `;`) and stops only at
whitespace, structural delimiters, `;;` comment start, or pipe-close sequences
such as `|]`. Quotation marks are not bare string content.

- `content` — **bare string atom**: canonical for delimiter-free content.
  `schema@next`, `required*`, `a&b`, `x>y`, `host:port`, and `a;b` are all
  bare strings in a typed `String` position.
- `[content with spaces]` — **inline bracket string**: single-line content that
  needs delimiters because it contains whitespace. Cannot contain literal `[` or
  `]` because those are structural delimiters.
- `[|content with [brackets]|]` — **pipe text**: multi-line and
  delimiter-sensitive content. Use this for bracket-bearing text, newlines,
  `;;`, and pipe-close markers.

Typed `String` decoding rejects redundant delimiters. `[schema]` and
`[|schema|]` are errors in a `String` position; write `schema`. Brackets are for
strings that need delimiters, not an optional spelling of every string.

The parser still sees ordinary square brackets as structural vectors; the
expected type decides whether a square-bracket block is a `Vec<T>` or a
space-joined `String` body.

The encoder structurally cannot emit a quotation mark: `write_string` has three branches (bare identifier, `[|...|]` block, `[...]` inline) and no quote branch. Legacy `"..."` quoted strings are accepted as **migration input only** (a `read_legacy_quote_string` path); a legacy → canonical round-trip sheds the quotation marks. Legacy acceptance is removed once all emitter sites migrate.

### Embedding-safety is the load-bearing consequence

Because NOTA never contains a `"`, a complete NOTA expression embeds escape-free inside any host whose string syntax uses double quotes — JSON, Rust string literals (including raw `r"..."`), Nix attribute values, YAML scalars, TOML strings, shell double-quote arguments, HTTP bodies, database string columns, env-var values, XML attributes. JSON-in-JSON requires escape cascades; NOTA-in-anything-with-double-quote-strings is escape-free. Design new emitters and storage paths to take advantage of this.

### Shell invocation uses outer double quotes

When NOTA is passed as an inline CLI argument, wrap the whole object in shell double quotes:

```sh
spirit "(Record ([(Information Documentation)] Correction [description text] Medium Minimum Zero []))"
```

This is why authored strings use `[text]` and `[|text|]`, not `"` delimiters: the shell keeps `"` as the outer argument boundary. Single quotes are not the normal inline form — they make natural apostrophes painful and undercut the bracket-string design.

### Inline NOTA — no `\n` escape sequences

Inline NOTA in any single-line string literal context (Rust string, shell argument, markdown inline example, test fixture, doc example) MUST NOT contain `\n` escape sequences. NOTA is whitespace-insensitive — the parser treats any run of whitespace (space, tab, newline) identically between tokens. A `\n` inside a single-line literal adds nothing semantically and produces a hybrid that pretends to be multi-line while being one source line: ugly to read, ugly to grep.

```rust
// Wrong:
let source = "(State [Statement])\n{ Topic [Text] }\n";
// Right (single-line, spaces between tokens):
let source = "(State [Statement]) { Topic [Text] }";
```

For genuinely multi-line NOTA — long fixtures, multi-record sources, schemas with many declarations — use actual newlines in authored `.nota` / `.schema` files loaded via `include_str!`, or a multi-line raw string literal:

```rust
let source = r#"{}
(Input ((Record Entry)))
(Output ())
{
  Topic [Text]
}"#;
```

Single-line for one or two records; file or multi-line raw string when the structure benefits from layout.

### Map keys

Maps use their own delimiter:

```nota
{host localhost port 8080 User 100}
```

Inside `{ }`, odd positions are key text and even positions are values. The schema chooses the scalar key type (`String`, `Path`, or a string-like newtype such as `NodeName`). A bare PascalCase key is allowed because the map delimiter already says this token is key text, not a value. Keys with whitespace are invalid, even when bracket-delimited.

### Schema namespaces use the brace-map rule

In a `.schema` namespace, write `Name body` pairs directly inside `{ }`:

```nota
{
  Entry [Topics Kind Description Magnitude]
  Kind (Decision Principle Correction Clarification Constraint)
}
```

Do not wrap namespace entries as `(Entry [...])` or `(Kind (...))` — the brace already supplies the key/value structure. Conceptually the namespace is a DYNAMIC ENUM where each key is a variant tag and each value is the variant payload, stored as a key/value map for composition and APPEND-ONLY so existing positions stay stable.

### Bare `Path`

Where the schema expects `Path` (not `String`), the bare alphabet widens to include `/` and `.` for filesystem-shaped values. A bare `skills/operator.md` at a `Path` position parses; the same token at a `String` position is a typed error.

### No tuples

NOTA has vectors, structs, enums, and key/value maps. Tuples are poorly specified structs — they carry position but not field names, and field names are information. Use a named-field struct so the schema states what each position means.

### Optional values

`Option<T>` is a normal data-carrying enum. Absence writes bare `None` (case 3); presence writes `(Some inner)` (case 1). Tail omission is **not** a compatibility shape: every position in the text carries every position in the schema, always. `#[nota(default = …)]` is **forbidden**. A record short on tokens is a typed error, not a silent zero-fill.

### Multi-field unnamed structs are forbidden

`struct Pair(i32, i32)` has no field-name mapping; NOTA rejects it at serialize time. Single-field unnamed structs are transparent newtypes only — the inner value emits at the schema position. For heterogeneous positional data, use a named-field struct, which emits as an untagged struct record.

### Sigils

Two are reserved at the syntax layer: `;;` for line comments, `#` for byte literals. A single `;` is ordinary bare atom text. Sigils such as `@`, `!`, `?`, `*`, `&`, `^`, `%`, `<`, `>`, and `:` are legal inside broad bare string atoms unless a higher schema layer gives a narrower type its own rules. `=` is reserved.

## Before you sketch any NOTA record

Before producing any new NOTA shape — in a report, chat, or proposal — do four things:

1. **Open `manifests/active-outputs.nota` and read three records.** That's your template for a multi-kind manifest. Re-read `nota/README.md` if you haven't recently — these grammar facts are easy to misremember.
2. **Name the wrapping type that carries the most useful distinction** (Rule 1). Never a generic `Item`, `Entry`, or `Record` when the file already says so. The variant test: if you can't name another type that could go in this position, the wrapper is superfluous — drop it.
3. **Heterogeneous positional structure is a record (struct), not a sequence.** Lists are homogeneous; mixed-type positional structure is a struct with positional fields, and the struct's type name is not written as a tag. A PascalCase token immediately after `(` is an enum variant tag; otherwise fields come directly.
4. **Sketch fields positionally — no `(key value)` pairs, no nested wrappers when every record has the same inner shape.** Positional means `(Decision [description] Maximum)`, not `(Decision (description [...]) (magnitude Maximum))` and not `(Decision (Description [description]) Maximum)` when `Description` is the only thing ever in that slot. Variants are PascalCase (`Maximum`, not `maximum`); date and time are two bare positional fields (`2026-05-19 01:23`), not one bracket string.

Most agent NOTA mistakes are the same mistake — labeled fields. The fix is the same: read the stable manifest example, and let the wrapping type carry the schema.

## When you fight the rules

You'll want to wrap every record in `(Item ...)` "for safety" — don't, the file is the safety. You'll want to group records under section comments — don't, surface the category as the type. You'll want integer codes "because they're shorter" — they're not, once you count the lookup table that decodes them; names win.

If the same structural decision recurs across many NOTA files (a shared enum vocabulary, a shared identity newtype, a shared date-shape), that's a real workspace primitive. Document it once in the relevant repo's `skills.md` or `ARCHITECTURE.md` and reference by name; don't restate it in every preamble.

## When to hand-write the codec instead of deriving

`#[derive(NotaDecode, NotaEncode)]` is the right default for record types. The shared string codec now emits broad bare atoms whenever possible and rejects redundant brackets on decode, so ordinary `String` fields no longer need a hand-written codec just to avoid `[Entry]` noise.

Hand-write the codec only when the field's domain is narrower than ordinary `String`: for example, a schema type name, a lowercase topic atom, or an identity that forbids punctuation even though broad NOTA strings allow it. In those cases the newtype validates its own vocabulary and still delegates ordinary formatting to `NotaString`.

The fix: hand-write `NotaDecode` + `NotaEncode` on the newtype to inspect content and choose the emission form, using `nota_next::AtomClassification`:

```rust
impl NotaEncode for Name {
    fn to_nota(&self) -> String {
        if self.qualifies_as_symbol_name() {
            self.as_str().to_owned()
        } else {
            NotaString::new(self.as_str()).format()
        }
    }
}

impl NotaDecode for Name {
    fn from_nota_block(block: &Block) -> Result<Self, NotaDecodeError> {
        NotaBlock::new(block).parse_string().map(Self::new)
    }

    fn qualifies_as_symbol_name(&self) -> bool {
        AtomClassification::classify(self.as_str()) == AtomClassification::SymbolCandidate
    }
}
```

The decode side should not accept redundant brackets for a broad bare string; canonicality is part of the typed codec. Anywhere a narrower domain wants stricter syntax than broad `String`, the hand-written impl belongs on that newtype — on schema-in-Rust source nouns and on emission-target newtypes alike.

## See also

- `skills/nota-schema-docs.md` — pseudo-NOTA convention for documenting record schemas in markdown (angle-bracket placeholders, optional `?`, enum `|`).
- `manifests/active-outputs.nota` — a stable multi-kind manifest example.
- `manifests/module-dependencies.nota` — a stable single-record-shape manifest example.
- `nota/README.md` — the language grammar source of truth.
