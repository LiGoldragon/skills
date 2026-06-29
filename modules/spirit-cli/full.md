# Skill — spirit CLI

## Rules

Use `spirit` to capture and observe psyche intent. Spirit is the intent substrate; there is no file fallback. If the daemon is unavailable and capture is required, surface a blocker.

The CLI takes exactly one argument: inline NOTA when the argument starts with `(`, or a NOTA file path otherwise. It replies on stdout with typed NOTA and returns nonzero on transport, parse, or daemon errors.

```sh
spirit "(Record (([(Information Documentation)] Decision [description] Medium Minimum Zero []) ([([verbatim psyche words] None)] [reasoning])))"
spirit ./record.nota
```

Read the deployed schema from the canonical Spirit and signal-spirit sources when exact wire shape matters. Do not infer from old notes.

## Encoding

Records are positional NOTA. Struct bodies are untagged; enum variants carry their variant head. `Option` is `None` or `(Some <value>)`. Canonical strings are bare atoms when legal; use bracket or pipe text only when delimiters, whitespace, or prose require it.

The intent `Record` request is `Entry` plus `Justification`.

`Entry` fields, in order:

1. domain vector;
2. kind: `Decision`, `Principle`, `Correction`, `Clarification`, or `Constraint`;
3. agent-clarified description;
4. certainty magnitude;
5. importance magnitude;
6. privacy magnitude;
7. referent vector.

`Justification` carries testimony plus reasoning. Testimony quotes the psyche verbatim and may include an antecedent question or context. Do not paraphrase testimony.

Magnitude values are `Zero`, `Minimum`, `VeryLow`, `Low`, `Medium`, `High`, `VeryHigh`, and `Maximum`. `Zero` privacy is open/public; private personal substance never goes there.

## Capture discipline

Capture only directive, durable, universal psyche intent. Matter about one component, one architecture, a task, or Spirit operation belongs in the owning code, docs, task tracker, or skill source instead.

Before recording, check for an existing record on the same topic. Clarify, supersede, retire, or change the existing record when that is the truthful operation; do not create duplicates because it is easier.

Use the guardian rejection as evidence. If it rejects, fix the testimony, warrant, privacy, certainty, importance, duplicate handling, or non-intent routing instead of retrying blindly.

## Observe and maintenance

Use public read surfaces for ordinary open intent reads and private read surfaces only when the task is authorized for elevated privacy. Use lookup when an identifier is known. Use count/search surfaces to scope a maintenance pass before changing records.

Use typed maintenance operations for removal, clarification, supersession, retirement, certainty, and importance changes. Do not edit intent by writing ad hoc files.

State the Spirit operation run and the returned identifier or blocker in the worker evidence.
