# Skill — Protos syntax

## Proto-language

Protos is the shared structure behind every family member — schema, NOTA, logos,
and the Rust form. Its universal aspect is three things: how delimiters are used,
capitalization, and the typed-inner-blocks approach to parsing. Schema's structure
expresses it most accurately; NOTA is one simple member, not the base the others
subtype. When writing any example syntax, obey these laws; never spell from memory
of another language.

## Positional records

Protos records are positional, never named. A block's positions are typed by the
expected type at each boundary — the type standing there fixes slot count and
meaning. Field, argument, and variant-payload identity comes from expected type
plus position, so a block carries no JSON-like labels, ever. `Entry.{ Topics Kind
Description Magnitude }` is four typed slots in fixed order; the field names live
in the type, not the text.

The expected type stands at every boundary: file kind, schema field, declaration
slot, generic argument, inner block. The raw layer only discovers atoms,
delimiters, and glued-dot application — it classifies nothing and never guesses
from content. Each inner block is re-read under the type expected at its position
(typed inner blocks), so the same raw shape means different things under different
expected types.

## Delimiter roles

Each delimiter carries one role:

- `{ }` — structs (positional field records); a single-element brace is a newtype.
- `[ ]` — vectors (homogeneous, where order or duplicates matter) and enum
  variant lists.
- `( )` — payloads: an application payload (`Head.( … )`), a map written
  `Map.( key.Value … )`, or a string whose content forces the bracket.
- `(| … |)` — the literal-preserving multiline string, for content carrying
  delimiters, comment markers, or newlines; the close marker `|)` is escaped in
  the body.

A canonical string is a bare atom (`schema`); a period-joined bare chain reclaims
its dotted text (`a.b`); a string with spaces takes parentheses (`(alpha beta)`);
a redundant wrap such as `(schema)` is rejected.

## Glued-dot application

A glued period binds a head to the following payload as one right-associative
application: `Private.secretDigest.StateDigest` reads as visibility, then the
(name, type) remainder. The dot binds only when glued on both sides — `Head
.Payload`, `Head. Payload`, `Head.`, and `.Payload` are all errors. A period is a
structural operator, so an atom never contains one; a dotted path (`rustfmt.skip`)
or a float (`-122.3`) is an application reconstructed from its segments.

## Capitalization discipline

Types are PascalCase; field and role names are camelCase. A `name.Type` or
`role.Type` binding is a camelCase atom dot-prefixing a PascalCase type.
Capitalization is a load-bearing pillar, not decoration: it statically
distinguishes a declaration's kind head and its role atoms.

## Field-name elision

Elision is the default. A field whose type is unique in its block carries no name
and takes its type-derived name. An explicit field name is legal only where two or
more fields in the block share a type and the name disambiguates them; a name on a
uniquely-typed field is an error, not a style choice. In `DatabaseMarker.{
CommitSequence StateDigest secretDigest.StateDigest }` only the second
`StateDigest`, which collides, carries an explicit name. (Whether a meaningful
custom name may sit on an otherwise-unique single field is still open with the
psyche.)

## Generics and newtypes

Generics resolve by kind and projection through a closed table — `Vector`,
`Optional`, `Map`, `ScopeOf`, `Bytes`, `Stream` — never by an open or aliased head
string: `Topics.Vector.Topic`, `Map.(K V)`, `RecordSet.Vector.Entry`. A
single-element braced form is a newtype carrying just the wrapped type and no field
name (`Summary.{ Description }`, `CommitSequence.{ Integer }`); a multi-field brace
is a struct. There is no multi-field tuple.

## Bare atoms and the escape sigil

Write canonical strings as bare atoms. The proto-language glyph set is `. ( ) [ ]
{ }`; the Nomos extension adds `+` and `$`. The `$` dollar sigil is the only
candidate escape and rides on an atom, not a new form — its exact semantics remain
open with the psyche. The double-angle spelling `<<name>>` is rejected as not
protos-like: it mis-lexes as a single bare atom and would demand registering
`<< >>` as a new delimiter pair.

## Textual vocabulary

Structural parsing vocabulary — declaration, field, sequence, pipe text, escape —
names how the recognizer works and never appears as a type name. The sections of a
construct are ordered positional slots, never labeled heads.

## Worked examples

From a schema fixture: positional structs, a single-element newtype, generics by
kind, and enum variant lists.

```
Topic.String
Topics.Vector.Topic
Summary.{ Description }
Entry.{ Topics Kind Description Magnitude }
Kind.[Decision Principle Correction Clarification Constraint]
```

The ruled stream declaration — the two uniquely-typed legs elide, the two colliding
`SubscriptionToken` legs keep names.

```
IntentEventStream.Stream.{ token.SubscriptionToken SubscriptionStarted IntentEvent close.SubscriptionToken }
```

Derived encodings: struct `{(commit sequence) 4}`; enum `Idle` / `Tick.7` /
`Range.{3 9}`; option `None` / `Some.42` / `Some.(cache entry)`; vector `[alpha
beta gamma]`; map `Map.(alpha.1 beta.2)`.

## Labeled sections, wrong versus right

Labeled section heads and double-angle escapes are both illegal. `input.`,
`core.`, and `logos.` name positional slots — forbidden, because Protos is
positional, not named.

Wrong:

```
Macro.WireNewtype.(
  input.{ name.Name wrapped.Type }
  core.{ … }
  logos.<<name>>.Newtype.<<wrapped>>
)
```

Right — sections are ordered positional slots, each typed by the expected type at
its boundary; no labels and no `<< >>`. (The macro's substitution spelling, the
`$` sigil, is still open with the psyche; the shape below shows the positional
structure, not a settled escape syntax.)

```
Macro.WireNewtype.(
  { name.Name wrapped.Type }
  Public.Newtype.( name standardWireAttributes wrapped )
)
```
