# Skill — NOTA schema docs

A tiny convention for documenting NOTA record schemas so agents don't
have to read Rust to know what a record looks like.

## The convention

To document a NOTA record's shape, write a pseudo-NOTA form where
**placeholders go in angle brackets** and **optional fields end in
`?`**:

```nota
;; Bug: a concrete defect with an incident timestamp and a fix path.
(Bug <title> <description> <severity> <incident-at?> <reproduction?>)
;;   title         : ShortText
;;   description   : TextBody
;;   severity      : Catastrophic | High | Normal | Low
;;   incident-at?  : Timestamp
;;   reproduction? : TextBody
```

First line: typed-record name + positional placeholders. Lines below:
name → type, one per placeholder.

Rules:

- **Placeholders use `<kebab-case>`**, spelled out: `<incident-at>`,
  not `<inc>`.
- **Optional fields end in `?`** in the form-line *and* the type list.
- **Closed enums use `|`** between variants: `Catastrophic | High`.
- **Lists wrap their inner type in square brackets**: `[<label>]`.
- **Nested records use the same form**: `(Identity <name> <ssh-fingerprint?>)`.
- A `;;` line is a NOTA comment; use comments to name each field's type
  and the rule's *why* when it isn't obvious.

## Why pseudo-NOTA, not the real syntax

Real NOTA is positional — field names live in the Rust schema. A
reader of `(Bug [Whisrs hangs] [...])` can't tell what the second
string is for without reading the Rust. The pseudo-NOTA form surfaces
the field names inline so the reader doesn't chase the schema crate.

This form is teaching material for docs (skill files, ARCH files,
design reports), not the authoritative wire shape. The canonical truth
is the Rust schema plus `tests/round_trip.rs` and
`tests/canonical_examples.rs`; pseudo-NOTA is orientation that lets an
agent draft a payload without reading the Rust.

## Example — a memory variant set

```nota
;; Persona-mind memory variants. Closed enum.
;; Common fields apply to every variant; kind-specific extensions follow.

(Memory <common> <kind>)
;;   common : (Common <id> <title> <body> <created-at> <created-by> <status> <priority> <labels>)
;;   kind   : Task | Bug | Feature | Epic | Decision | Migration | Discipline | Investigation

(Task <acceptance-criteria> <spec-id?> <progress-notes>)
;;   acceptance-criteria : TextBody
;;   spec-id?            : ReportPath
;;   progress-notes      : [(Note <timestamp> <author> <body>)]

(Bug <severity> <incident-at?> <reproduction?> <discovery-path?>)

(Feature <branch> <repos>)
;;   branch : BranchName
;;   repos  : [RepoName]

(Epic <children> <required-skills?>)
;;   children        : [TypedThoughtId]
;;   required-skills : [SkillName]
```

## See also

- `skills/nota-design.md` — designing the records you actually write
  (this skill is about *documenting* schemas in markdown).
- `manifests/active-outputs.nota` — a stable manifest example of well-designed
  NOTA.
