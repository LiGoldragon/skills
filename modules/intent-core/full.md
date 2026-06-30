# Module - intent core

## Intent Core Purpose

Intent work preserves what the psyche actually said and manifests it into the
right durable guidance. The psyche is the human author. Agent messages,
reports, implementation choices, and test failures are not psyche intent.

## Intent Capture Gate

Record only explicit durable psyche statements that still matter after the
current task is erased. Classify them as Decision, Principle, Correction,
Clarification, or Constraint. When durable meaning, kind, target record, or
privacy is unclear, ask instead of inferring.

Before writing, read the existing intent neighborhood for the same domain and
referents. Most apparent new records are duplicates, clarifications, or
supersessions of existing records. Use maintenance operations for those cases.

## Intent Spirit Surface

Spirit is the intent substrate; there is no file fallback. Use the deployed
Spirit CLI for Record, Observe, Clarify, Supersede, Retire, Remove,
ChangeRecord, ChangeCertainty, ChangeImportance, and related maintenance
operations. If the daemon is unavailable and capture is required, surface a
blocker.

The CLI takes exactly one argument: inline NOTA when the argument starts with
`(`, or a NOTA file otherwise. It replies on stdout with typed NOTA and returns
nonzero on transport, parse, or daemon errors.

Record requests carry `Entry` plus `Justification`. `Entry` fields are domain
vector, kind, agent-clarified description, certainty, importance, privacy, and
referent vector. `Justification` carries verbatim psyche testimony plus
reasoning. Descriptions may clarify; testimony quotes exactly.

```sh
spirit "(Record (([(Information Documentation)] Decision [description] Medium Minimum Zero []) ([([verbatim psyche words] None)] [reasoning])))"
```

Records are positional NOTA. Struct bodies are untagged; enum variants carry
their variant head. `Option` is `None` or `(Some <value>)`. Canonical strings
are bare atoms when legal; use bracket or pipe text only when delimiters,
whitespace, or prose require it.

Magnitude values are `Zero`, `Minimum`, `VeryLow`, `Low`, `Medium`, `High`,
`VeryHigh`, and `Maximum`. `Zero` privacy is open; private personal substance
stays off open surfaces.

Read the current canonical Spirit and signal-spirit sources when exact wire
shape matters. Do not infer from old notes.

## Intent Manifestation

Capture is incomplete until affected guidance surfaces reflect the settled
intent: workspace guidance, a repo's `ARCHITECTURE.md` (or a code stub with an
explanatory comment), skills, or repo-local guidance as appropriate. Manifest only what the psyche stated. Keep
private or personal material off public surfaces unless explicitly authorized
for that privacy level.

## Intent Maintenance

Use typed maintenance operations for removal, clarification, supersession,
retirement, certainty, and importance changes. Do not edit intent by writing ad
hoc files. Treat guardian rejection as evidence: fix testimony, warrant,
privacy, certainty, importance, duplicate handling, or non-intent routing.

Fold mistaken standalone clarifications into their targets, retire or remove
duplicates through the deployed maintenance path, and keep supersession
explicit. Do not collapse conflicting records by taste; preserve the conflict or
ask for a psyche decision.
