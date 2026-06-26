# Skill — NOTA-as-comments

## The frame

A signal record can be encoded as bytes (rkyv), as a NOTA string, or as
text inside a comment. The third form is the same signal, just sitting in
source code rather than a wire frame or `.nota` file. The comment uses
NOTA-in-text because the file *is* text — NOTA is the encoding that fits a
text edge.

Two payoffs:

- **The why is machine-readable.** A `(Why …)` comment is a typed record
  Mind can index and auditor can read — not prose needing natural-language
  parsing.
- **The why lives next to the code.** A report retires; the code outlives
  it. A `(Why …)` records the decision where the code lives, so future
  agents see the rationale the moment they read the function, without
  chasing a report that may be gone.

## The shape

Open with `(Why "<short summary>" …)`. The summary is a one-line phrase
(the rephrasing Mind indexes). Positional sub-records follow — type first,
fields in declared order, no labeled fields, no `(key value)` pairs (per
`nota-design.md`):

```rust
// (Why "fix recovery path on transient failure"
//   (caused-by "the completion-failure path left state un-flipped")
//   (alternatives-considered (RetryWithoutFlip ColdStart))
//   (chosen-because "preserves the no-downtime precedent"))
fn handle_completion_failure(&mut self) -> Reply {
    // ...
}
```

- `(caused-by "<trigger>")` — what occasioned the edit (a bug, an audit, a
  psyche intent).
- `(alternatives-considered (Variant1 Variant2 …))` — the closed set of
  approaches weighed. PascalCase variants, searchable later ("did anyone
  consider `ColdStart` here?").
- `(chosen-because "<rationale>")` — the why-of-the-why.

Sub-records are themselves positional NOTA: bare PascalCase tokens at
variant positions, strings quoted, no labeled fields.

## Multi-line comments aggregate into one record

A `(Why …)` spans as many comment lines as it needs; the parser
concatenates the payload across consecutive `//` lines (Rust) or `#` lines
(shell, Nix, Python). The comment-prefix character and indentation are
language-syntax noise the indexer strips — the NOTA record is what
survives. The shape is identical across languages:

```nix
# (Why "pin persona-spirit before persona-mind in deploy order"
#   (caused-by "mind queries spirit at startup")
#   (alternatives-considered (PinSpiritFirst PinMindFirst RunBoth))
#   (chosen-because "mind needs spirit's wire shape resolved first"))
persona-mind = { after = [ "persona-spirit.service" ]; };
```

## Where the `(Why …)` goes

It sits where the next reader will see it the moment they look at the code
it describes:

- **On a function / type** — opening lines above the `fn`, `struct`,
  `enum`, or `trait`. Readers see the why before the implementation or
  field layout.
- **On a tricky line or block** — immediately above the expression it
  shapes.
- **On a module** — first lines of the file (after any file-header
  comment).

A function's why is above the function, not at the top of the file.

## `Why` vs intent records

Both carry "why-ness" but live at different layers:

- **Intent records (Spirit)** — what the psyche stated, in the author's
  voice, typed (Decision / Principle / Correction / Clarification /
  Constraint). The authoritative source. See `intent-log.md`.
- **`(Why …)` comments** — the editor's per-edit rationale: why I picked
  this variant, what I weighed, what prior decision I'm honoring. The
  comment is the editor's witness, not the psyche's voice.

When the editor's choice rests on a psyche intent, `(chosen-because …)`
cites that intent by topic. The two layers are linked but separate; the
cite is the link. A psyche statement that classifies as an intent type
goes to Spirit via the spirit CLI, not into a comment.

## When a `(Why …)` is worth writing

The test: *would a future agent benefit from seeing the why before
touching this code?*

Write one when:

- **Substantive choice** — the edit picked one of several plausible
  variants; the reasoning isn't obvious to someone who didn't watch the
  deliberation.
- **Citation needed** — the choice rests on an intent record or prior
  decision future readers need to know.
- **Surprising shape** — the code looks odd for a non-obvious reason, and
  a future agent might "fix" it and undo the precedent.

Skip it for routine renames, mechanical translation, and code whose shape
is obvious from context. A `(Why …)` on the obvious is noise.

## What this skill is NOT for

- **Documenting what the code does.** That's the code itself; doc-comments
  (`///`) explain the API. `(Why …)` is the rationale layer above.
- **Replacing reports.** Reports name the bigger arc (a multi-function
  pass, a cross-repo synthesis); the `(Why …)` names the per-edit
  rationale at the line.
- **Replacing Spirit intent records.** See the boundary above.
- **Free-form prose with parentheses around it.** `(Why …)` is a
  positional NOTA record; the same grammar applies. If prose doesn't fit
  positional NOTA, write an ordinary prose comment — don't dress free text
  in fake NOTA.

## Mind integration

Once persona-mind ships its code-indexer, the discipline pays off: the
same NOTA codec used for wire frames parses `(Why …)` out of comments
(the comment-prefix strip is the only difference), and each becomes a
typed memory keyed by `(repo, file, line-range)`. Then agents can query
"what alternatives were considered for the recovery path?" before making
conflicting edits, and the auditor reads accumulated whys to refine or
reverse a flag when a recorded rationale already explains the shape. Until
that lands, the whys still serve human and agent readers who open the
file; the indexer is the upside, not the prerequisite.

## See also

- `nota-design.md` — positional-record discipline every `(Why …)` follows.
- `intent-log.md` — Spirit intent records; the why-vs-intent boundary.
- `reporting.md` — reports name the arc; whys name the per-edit rationale.
