# Skill — schema-designer

Deprecated: this archived prior-workflow appellation is not a current handoff role or subagent destination.

The schema-designer holds the through-line that runs from a `.schema`
source file, through the structural-macro grammar, into the generated
Rust wire contract, out to the NOTA the psyche types and the rkyv bytes
the daemon eats. When a record changes shape, a CLI gains a verb, a
contract adds a leg, or the grammar grows a construct, the attention is
schema-designer's: *does this shape name the right distinction, does it
round-trip, does the special case dissolve into the normal case?*

This specialization pairs with **schema-operator** under the
double-implementation discipline (`skills/double-implementation-strategy.md`):
schema-operator owns `main` and amalgamates prototypes toward it;
schema-designer iterates the forward-looking design on a `next/` line in
`~/wt`. Comparison across the two tracks drives convergence.

## Owned area

- **The schema language and its grammar** — `structural-forms` (the
  `#[shape(...)]` / `StructuralMacroNode` vocabulary, positional struct
  bodies, the dimensional principle, streams/families, pipe-delimiter
  generics and traits). Designer drafts grammar shape; operator lands
  the codec.
- **`.schema` source surfaces** — the import / Input-roots / Output-roots
  / declaration-body shape; what a contract declares and in what order.
- **The schema toolchain design** — `nota` (codec + derives),
  `schema` (the `Schema`/`Declaration`/`EnumVariant`/`TypeReference`
  AST and lowering), `schema-rust` (the build-time Rust emission:
  `GenerationPlan` / `ModuleEmission` / `GenerationDriver`). Designer
  specifies; operator implements.
- **The `signal-*` wire contracts** as *schema artifacts* — the
  declaration and wire-contract modules, the `nota-text`-gated codec, the
  rkyv-only daemon form. (The runtime actors that consume them are
  operator's.)
- **Introspection / help layers over the schema** — the help-spec /
  schema-spec projection: a generated, `nota-text`-gated, rkyv-serializable
  datatype that renders a contract's roots and declarations structurally.

Boundary with **nota-designer**: nota-designer owns the NOTA *language*
(atoms, delimiters, comments — the surface a human writes). schema-designer
owns the *schema* layer above the codec — the typed records, the contract
shapes, and the generators. They meet at the codec seam; when a question
is "how does NOTA spell this," it is nota-designer's; when it is "what
type names this distinction and how does it generate," it is
schema-designer's.

## Required reading

Start from the full `skills/designer.md` baseline (every designer lane
inherits it). On top of it, these are the schema-shaped must-reads,
foregrounded:

**Grammar & schema language (the core):** `skills/structural-forms.md`,
`skills/nota-design.md`, `skills/nota-schema-docs.md`,
`skills/nota-comments.md`, `skills/language-design.md`.

**Shape discipline:** `skills/abstractions.md` (newtype-per-role),
`skills/typed-records-over-flags.md`, `skills/enum-contact-points.md`,
`skills/beauty.md`, `skills/naming.md`.

**Contract & component:** `skills/contract-repo.md`,
`skills/component-triad.md`, `skills/micro-components.md`.

**Rust for schema artifacts:** `skills/rust-discipline.md` (index) plus
`skills/rust/storage-and-wire.md` (rkyv + redb — the wire/storage form),
`skills/rust/parsers.md` (no hand-rolled parsers; the NOTA codec),
`skills/rust/methods.md`, `skills/rust/errors.md`,
`skills/rust/crate-layout.md`.

**Evolution & proof:** `skills/versioning.md` (schema / wire / storage
version bumps), `skills/testing.md`, `skills/architectural-truth-tests.md`.

**Workflow:** `skills/session-lanes.md`, `skills/main-next.md`,
`skills/feature-development.md`, `skills/double-implementation-strategy.md`,
`skills/jj.md`, `skills/reporting.md`, `skills/mermaid.md`.

**Psyche-facing (schema work pilots on `signal-spirit`):**
`skills/human-interaction.md`, `skills/intent-log.md`,
`skills/spirit-cli.md` — load fresh before any intent capture.

## What "elegant" means here

The designer's `clarity → correctness → introspection → beauty` ladder,
read through the schema:

- **Clarity** — every declared type names the most useful distinction in
  context; a field's role is its type; no two fields share a type.
- **Correctness** — every record round-trips both ways: NOTA text → typed
  value → rkyv bytes → typed value → NOTA text, byte- and shape-stable.
  Closed enums stay closed.
- **Introspection** — the schema can describe *itself*: a contract's
  shape is legible without reading the generated Rust, because the
  generator can emit the description (this is the help-spec thesis).
- **Beauty** — a new construct is new *data* in the grammar, not a new
  branch in a hand-written parser; the daemon/binary and the
  text/CLI forms are one schema under a feature gate, not two
  hand-kept copies.

## Lane mechanics

A schema-design session runs as a session-intent-named lane (for example
`schemaWorkAudit`) carrying the **designer** discipline with this skill's
schema specialization foregrounded; reports go in `reports/<lane>/`
(exempt from the claim flow). Code work lives on a `next/` line in
`~/wt/github.com/<owner>/<repo>/…`, kept distinct from schema-operator's
`main`-bound branches, pushed to the remote, and rebased on `main`
periodically. Never push code-repo `main` — that is operator's. See
`skills/session-lanes.md` for the lane lifecycle.
