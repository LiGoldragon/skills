# Skill — Protos syntax

## Proto-language

Protos is the shared structure behind the NOTA-family textual surfaces — schema,
NOTA, and logos. Its universal aspect is three things: how delimiters are used,
capitalization, and the typed-inner-blocks approach to parsing; schema expresses
that structure most accurately. The Rust form is a foreign raw layer, not a member
Protos stands behind. When writing any example syntax, obey these laws and quote a
real artifact; never spell an example from memory of another language.

## Positional records

Positionality is absolute; it is the first law of Protos and outranks every other
rule here. Protos records are positional and there are no field names anywhere in
Protos. A block's positions are typed by the expected type at
each boundary — the type standing there fixes slot count and meaning. Field,
argument, and variant-payload identity comes from expected type plus position, so a
block carries no JSON-like labels, ever. A construct's sections are ordered
positional slots typed by the expected type at their boundaries, never labeled
heads.

An explicit field name is completely illegal everywhere — never authored, never a
candidate, never an example, and never a codec-emitted form. There is no collision
exception: no field name is ever added to a Protos record, not even by a codec, and
same-typed fields are separated by position alone. This law bars fabrication, never
disclosure: never invent a field name and never present a named-field spelling as a
candidate, example, or real Protos. Real artifacts stay fully visible: a field name
found in a real artifact is quoted exactly when that artifact is reported, and
nothing real is ever withheld — but it is never authored, proposed, or presented as
correct Protos.

The expected type stands at every boundary: file kind, schema field, declaration
slot, generic argument, inner block. The raw layer only discovers atoms,
delimiters, and glued-dot application — it classifies nothing and never guesses
from content. Each inner block is re-read under the type expected at its position
(typed inner blocks), so the same raw shape means different things under different
expected types.

## Delimiter roles

Each delimiter carries one role; the glyph set is `. ( ) [ ] { }`:

- `{ }` — structs (positional field records); a single-element brace is a newtype.
- `[ ]` — vectors (homogeneous, where order or duplicates matter) and enum
  variant lists.
- `( )` — payloads: an application payload (`Head.( … )`), a map written
  `Map.(alpha.1 beta.2)`, or a string whose content forces the bracket.
- `(| … |)` — the literal-preserving multiline string, for content carrying
  delimiters, comment markers, or newlines; the close marker `|)` is escaped in
  the body.

A canonical string is a bare atom (`schema`); a period-joined bare chain reclaims
its dotted text (`a.b`); a string with spaces takes parentheses (`(alpha beta)`);
wrapping an already-canonical bare atom in parentheses is redundant and rejected.

## Glued-dot application

A glued period binds a head to the following payload as one right-associative
application: `Topics.Vector.Topic` reads as the head `Topics` bound to the payload
`Vector.Topic`. The dot binds only when glued on both sides: a space before
or after the period, a head with a trailing period and no payload, and a payload
with a leading period and no head each fail to parse. A period is a structural
operator, so an atom never contains one; a dotted path (`rustfmt.skip`) or a float
(`-122.3`) is an application reconstructed from its segments.

## Capitalization discipline

Types, kind heads, and enum variants are PascalCase (`Topic`, `Stream`, `Vector`,
`Decision`); canonical string atoms and map keys are lowercase bare atoms
(`schema`, `alpha`, `beta`). Capitalization is a load-bearing pillar, not
decoration: it statically distinguishes a declaration's PascalCase kind head from
lowercase data atoms. A lowercase atom labeling a positional slot would be a field
name, which is illegal everywhere.

## Positional disambiguation

Every field is positional and carries no name. When a struct holds two or more
fields of the same type, position alone assigns each its meaning: the struct's
declared field order fixes which slot is which, and the expected type standing at
each position carries identity. No name is ever added to separate same-typed fields
— not an authored one and not a codec-emitted one; the disambiguation is entirely
positional, the same rule that governs every other slot.

## Generics and newtypes

Generics resolve by kind and projection through a closed table — `Vector`,
`Optional`, `ScopeOf`, `Map`, and `Bytes` — never by an open or aliased head
string; applications dispatch on kind and projection, not on head text:
`Topics.Vector.Topic`, `RecordSet.Vector.Entry`, `Map.(alpha.1 beta.2)`. A
single-element braced form is a newtype carrying just the wrapped type and no field
name (`Summary.{ Description }`, `CommitSequence.{ Integer }`); a multi-field brace
is a struct (`Entry.{ Topics Kind Description Magnitude }`). There is no multi-field
tuple.

## Worked examples

From the `spirit-min.schema` fixture — positional structs, a single-element
newtype, generics by kind, and an enum variant list:

```
Topic.String
Topics.Vector.Topic
Summary.{ Description }
Entry.{ Topics Kind Description Magnitude }
Kind.[Decision Principle Correction Clarification Constraint]
```

Encodings witnessed by the NOTA grammar tests: struct `{(commit sequence) 4}`;
enum `Idle` / `Tick.7` / `Range.{3 9}`; option `None` / `Some.42` /
`Some.(cache entry)`; vector `[alpha beta gamma]`; map `Map.(alpha.1 beta.2)`.

## Nomos macro definition syntax is unsettled

The Nomos macro-definition surface — how a macro names its input and body and
spells substitution — is under live design and is not settled. Do not exemplify it
and do not guess its spelling. When a skill must cover this surface, name it
unsettled rather than inventing a form.
