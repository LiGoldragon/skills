# Skill — language-design instincts

The load-bearing commitments when designing a text notation, request
language, schema notation, or query surface.

## What this skill is for

Apply this before writing a parser for any text surface. The syntax
substrate is NOTA. Nexus is a NOTA-using request/message surface, not
a second syntax — when the workspace says "text" for requests or
messages, it usually means Nexus records written in NOTA. Some
configs and convenience CLIs use NOTA directly.

If you're only *using* a notation already defined, read `nota`'s
`README.md` or the relevant schema instead. These principles are
upstream of every NOTA text surface; they do not authorize additional
syntax.

## The instincts

### 0. NOTA is the only text syntax — no new text formats, ever

All human-facing and harness-facing text uses NOTA syntax. Nexus is a
NOTA user: it defines records, verbs, and interpretation over NOTA,
but it is not a second grammar. There are no other text formats in
this workspace.

If a discussion floats a name like "PersonaText", "MessageLang",
"AgentSpeak", or any "text language for X", stop and refuse:

- **NOTA** is the syntax (lexer, parser, codec, canonical encoding —
  owned by `nota-codec` + `nota-derive`).
- **Nexus** is the typed text content (records, verbs, interpretation
  rules) written in NOTA. It is the text the workspace uses end-to-end.
- **CLI sugar** (e.g. `message foo` desugaring to
  `(Assert (Message foo))`) is convenience over often-used Nexus
  sub-vocabularies. Sugar is not a new language; it must desugar 1:1
  to a Nexus expression.

This is load-bearing because once a second text format exists, every
consumer must know which format the producer used, and the union
grows forever. Always extend Nexus — add a typed record, verb, or
sub-vocabulary — rather than introduce a new text surface. When you
see a candidate text format named in any report, contract, or bead,
edit it out.

### 1. Delimiter-first

Every construct has explicit opening and closing delimiters. The
parser knows what it is reading from the first token — no fallback
rules, no multi-token lookahead, no scanning the interior to decide
what kind of thing this is. `(Foo …)` is a record; `[…]` is a
sequence. If a notation needs lookahead to classify a fragment, the
grammar has a missing surface — extract it as a separate DSL with its
own outer delimiter.

### 2. No keywords beyond truth values

Closed sigil and delimiter budgets. The parser dispatches on position
and head identifier, never on a reserved word. The only literals
meaningful to the parser are `true`, `false`, and `None` (the
absent-value sentinel for `Option`). New features land as new
delimiter-matrix slots or new PascalCase records — never as keywords
or sigils. A feature that seems to demand a keyword is asking for a
typed record.

### 3. Position defines meaning

The same delimiter means different things in different positions; the
parse position is the sole authority. `()` at record-head means "this
is a record"; inside a record body it might mean "a typed field";
inside a body block it might mean "call arguments." Readers learn the
position rules once. Position carrying meaning is what removes the
need for keywords.

### 4. PascalCase = type, camelCase = instance

The parser dispatches on first-character case. `Foo` is a type,
variant, or structural name; `foo` is a field name or local instance.
The rule is enforced at parse time in record-head position — a bare
lowercase identifier in head position is a parse error, not a schema
mismatch. Benefits cascade: identifiers carry their kind visibly, the
lexer dispatches without consulting the schema, and the reader knows
what they're looking at without context.

### 5. Names are meaningful

No pointer names — no `T`, `X`, `A`, `B` for type parameters; no `x`,
`n`, `tmp`, `buf` for locals. Every name describes what the thing IS.
Type parameters use semantic role names: `$Value`, `$Output`,
`$Failure`, `$Left`, `$Right`. Two different parameters always have
different names — `$LeftValue` and `$RightValue` differ even when they
share qualities. Name IS identity. The cost is a few characters at the
declaration site; the gain is that every use site reads
unambiguously.

### 6. Every value is structured — no opaque strings

If a name or type is stored as a flat string, the ontology is
incomplete. Names are typed domain variants; types are structured node
trees. Opaque-string smells: `kind: String` where `kind` should be a
closed enum; `name: String` where it should be a typed identifier
newtype; `metadata: HashMap<String, String>` where metadata is hidden
control flow. Strings are transitional — each one is a placeholder for
a typed record not yet specified, and the schema's job is to grow them
into types.

### 7. Newlines are not significant

Whitespace (including newlines) is only a token separator; parsing is
purely token-based. `(Foo a b c)` and the multi-line indented form
parse identically. Indentation-sensitive grammars (Python, YAML) get
this wrong: a whitespace-dependent parser can't recover from
formatting, while a delimiter-driven one can. Structure lives in the
delimiters.

### 8. Text is flat; trees come from the compiler

Text is a left-to-right, top-to-bottom medium — every text-based
language is a flat sequence of tokens. Trees are what the compiler
constructs from the flat input. Keep grammar rules flat; let the
compiler do the structural work. Don't try to make grammars
hierarchical.

### 9. Content-addressing by canonical encoding

When a notation needs identity beyond the moment of writing, identity
is the hash of the canonical encoding. The canonical form is itself
defined: field order, whitespace, optional emission, string quoting —
all specified, all stable. Mutable handles (slot-refs, named
bindings) sit on top: the hash is the immutable identity, the slot is
the mutable handle.

### 10. No shortcuts in compiler work

No raw-text passthrough, no "skip for now" stubs, no partial
grammars. When you hit a language limitation, extend the language
properly. Self-hosting requires the full grammar — the same grammar
that parses also reconstructs (bidirectional), and shortcuts break
round-tripping. The cost of a clean extension is paid once; the cost
of a passthrough is paid every time the gap is encountered.

### 11. The parser stays small

Adding new typed kinds is the central activity of evolving a schema;
adding parser rules is rare. New syntactic territory becomes a new
DSL surface — its own outer delimiter, its own per-position semantics
— not new parser logic in the existing surface. The number of
grammars grows; the complexity of each one doesn't.

### 12. Mutable is marked

Immutability is the default; mutation is always visible at the
declaration site. A mutability sigil (`~` in aski, `mut` in Rust)
attached to a name says "this can change"; absence says "this is
fixed." Readers don't scan ahead to discover mutability. Marks
compose: `~&self` is "mutable borrow of self," combining the mutation
marker with the borrow sigil — each piece a separate decision, their
combination the expressed shape.

### 13. No multi-field unnamed structs

Unnamed positional grouping (Rust's `(A, B, C)` tuple) loses each
field's name. If two values travel together they have roles, and
those roles deserve names — use a named-field struct. Single-field
newtypes (`struct Md5([u8; 16])`) are allowed: they wrap one thing
with a type marker, and the marker IS the name. Multi-field tuple
structs are forbidden. The gain: `result.quotient + result.remainder`
reads unambiguously where `result.0 + result.1` does not.

### 14. Records are positional; field names live in the schema

Records on the wire are positional — no field names appear in the
text; the schema (in code) names the positions. This keeps the wire
form short and stable; renaming a field is a schema change, not a
wire-format change. Tail-omitted optionals are a compatibility
read-shape: a decoder may accept a record missing trailing optional
fields, but canonical encoders emit explicit `None` for absent
optionals. Reordering fields IS a wire-format change (positions
shift); plan accordingly.

### 15. Domains come from data — never hand-maintained

Every list of names, enum variants, or dispatch table in source code
is a bug. Types are derived from declarative data, never hand-written.
When a domain changes, the change lands in one place — the data — and
propagates. Hand-maintained dispatch tables drift silently: a new
variant added to one table is missed in another; a renaming breaks
something three call-sites away. The fix is generation, not vigilance.

### 16. Pure binary means pure binary

When a notation says "binary," it means actual byte values — not hex
strings, not JSON arrays of integers, not text-representations of any
kind. The bytes ARE the protocol. Hex, JSON, and base64 are distinct
projections of binary, useful at boundaries; they are never the
canonical form.

### 17. Defined inputs and outputs

Every pipeline component has explicit, declared inputs and outputs. A
component can take multiple inputs of different kinds and produce
multiple outputs; what matters is that every input and output is named
and typed. No "passthrough metadata," no "context object," no "carry
this along for the next step." If a step needs information, it's an
input; if it produces information, it's an output. The plumbing is
part of the schema.

### 18. Delimiters earn their place

A delimiter pair belongs in the grammar only when records and
sequences (the universal structural primitives) cannot express the
structural shape it would denote. The test: can the wire form be made
shorter or clearer for an expressive case that records + sequences +
primitives can't handle? If no, the delimiter stays out.

Two failure modes a free delimiter pair tempts:

- **Cosmetic distinctions.** Set vs. ordered list, map vs.
  sequence-of-pairs — these differ at the receiving type's level, not
  the wire's level. A delimiter for a cosmetic distinction grows the
  parser without expressive gain; the schema-typed receiver already
  encodes the distinction.
- **Verb-shaped uses.** "Schema declaration" vs. "data"; "governance
  record" vs. "domain record"; "with-context" wrapper. These encode
  operations into the delimiter — the opposite of *position defines
  meaning*. Verbs go on record head identifiers, never in delimiter
  pairs.

The structural minimum is records (`( )`) and sequences (`[ ]`). A
third delimiter pair (e.g. curly braces) earns its place only when the
language has a load-bearing structural shape genuinely outside this
minimum. Agents looking at a free delimiter reach for ways to *use*
it; the discipline is to ask whether it would express something
records and sequences can't, and when the answer is no, leave it free.
The grammar earns simplicity through subtraction.

## Branches and leaves — the typed-data-tree vocabulary

Typed records (the structs and enums in the sema database) form trees:

- **Leaf**: a fixed-size terminal — typically an enum of unit
  variants only (`Bool` as `True`/`False`, the seven-variant
  `Magnitude`, the five-kind `Kind` taxonomy). Final leaves have no
  variable size.
- **Branch**: variable-size or data-carrying content — vectors,
  data-carrying enum variants, structs containing variable-size
  members. Branches are themselves trees (recursive).

A vector field is a dynamic-leaf in one reading (it occupies a single
field position) but practically a branch because it points into
another tree. A data-carrying enum variant is a branch because its
payload doesn't fit as a single tree node.

## Where these instincts apply

| Surface | Instincts |
|---|---|
| nota | 1, 2, 3, 4, 6, 7, 8, 14, 16 |
| nexus | 1, 2, 3, 11 (extension surface), 14 |
| signal | 6, 9, 14, 15, 16 |
| persona-message records | 4, 5, 6, 14 |

## When the rules feel restrictive

Friction usually means the schema is incomplete. The right response is
"find the typed shape that makes this flow," not "add a string field"
or "bend the rule once." Fixed restrictions push design pressure into
structure; that pressure is the point.

If a real exception is needed, it gets a name and lives in the
relevant grammar spec — the way nota's "bare-identifier strings"
carve-out is documented. Carve-outs are explicit and narrow; absence
of a documented carve-out means there isn't one.

## See also

- `nota`'s `README.md` — grammar spec for the canonical NOTA syntax.
- `skills/abstractions.md` — verb belongs to noun; same discipline at
  the Rust-type level.
- `skills/naming.md` — full English words, applied inside notations as
  well as code.
