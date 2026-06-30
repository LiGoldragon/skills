# Skill — design quality

## The rule

If it isn't beautiful, it isn't done. Beauty is a gate alongside
correctness, not a nice-to-have. When something you've written or
are reviewing feels ugly, the discomfort is evidence the underlying
problem is unsolved. When the right structure is found, the ugliness
dissolves: special cases collapse into the normal case, repetition
resolves into a single named pattern. The structure you find *is*
the one you were missing. If you cannot make it beautiful, you do
not understand it yet.

In a typed-data workspace the aesthetic dimensions the gate tests
are: terseness, symmetry, schema-driven emission, self-describing
surfaces, interfaces-first shaping, composable boundaries. A design
that satisfies correctness but fails any of these is not yet done.

## Beauty as primary audit lens

Audits apply the beauty filter as the primary lens. Before asking
"does this work?" the audit asks "is the shape right?" When the
shape is wrong, the recommendation is the structural fix, not a
patch that preserves the ugly surface. This applies at four scales:

- **Code beauty** — the diagnostic catalogue below.
- **Capture discipline** — no duplicate durable records. A trio
  saying the same thing in three magnitudes is noise; one record at
  the right magnitude is signal.
- **Substrate cleanliness** — no hand-written code where schema can
  drive. When a hand-written enum, validate impl, trace vocabulary,
  or projection function parallels something a schema could emit,
  that parallel is the ugliness; push the substance into the schema.
- **Replaceable over additive** — design replaceably, not additively.
  When the current shape cannot do what is wanted, replace it and
  update every consumer rather than preserve the old shape behind a
  parallel compatibility path; a compatibility shape kept for its own
  sake manufactures legacy (`10pz`).

## What ugliness signals

Each item is a *signal*, not a sin. Notice it, name the underlying
problem, fix the underlying problem.

- **A name that doesn't read as English.** `pf`, `tok`, `op`. Each
  abbreviation costs the reader one mental lookup per occurrence
  forever, to save the writer three keystrokes once.
- **A `pub` field on a wrapper newtype.** `Slot(pub u64)` is a
  label, not an abstraction.
- **A free function that should be a method.** A verb that could
  attach to a noun reads as a missing model.
- **Dead code retained "for safety" or "backward compatibility."**
  Delete it; the history is in version control.
- **Special cases stacked on the normal case.** Find the rewrite
  that makes the special case disappear.
- **Stringly-typed dispatch.** `match name.as_str()` over cases that
  should be a closed enum.
- **A doc comment that explains *what* the code does.** Well-named
  code already explains what it does; the comment signals the names
  aren't carrying their weight.
- **A boolean parameter at a call site.** `frob(x, true)` reads as
  gibberish. Split into two functions or pass a typed enum.
- **A name for what something is *not*.** `non_root`, `non_empty`,
  `not_admin`. Negative names compose poorly. Find the positive name.
- **A long function with multiple responsibilities.** Split it.

## The "feels too verbose" anti-pattern

The most common slip is the verbosity objection to spelled-out
names. When `AssertOperation` "feels needlessly verbose," that
feeling is the signal to question the feeling — not to shorten the
name. The full English form reads as English; the abbreviation reads
as ceremony to be decoded. The cost of mis-naming is paid every time
the name is read; the saving of three keystrokes is paid once. The
criterion is beauty, not keystroke-economy.
