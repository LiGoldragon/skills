# Skill — intent maintenance

## Clarification is an edit, not another record

A psyche clarification means the existing record or manifested guidance
was ambiguous, incomplete, or misworded. The default action is therefore
to edit the target, not to append a fresh `Record`.

Required sequence:

1. Find the target. Query by referent, domain, keywords, or identifier,
   then `Lookup` the record whose meaning is being clarified.
2. If the target's core meaning survives and only its wording/scope
   changes, use `Clarify` with the target identifier and corrected
   description.
3. If the target should be replaced by one or more entries, use
   `Supersede`: retire the old identifier(s) and provide replacement
   `Entry` values in the same operation.
4. If the target should stop being active without replacement, use
   `Retire` or the removal flow below, depending on whether lineage
   should remain visible.
5. Update the manifested surface (`ESSENCE.md`, `AGENTS.md`,
   `INTENT.md`, repo `INTENT.md`, skills, or architecture docs) in the
   same work cycle.

Do **not** use `Record` for "clarification — ..." when an earlier record
already holds the arrow. That creates two active records and forces every
future reader to reconcile them manually. The only time a clarification
becomes a fresh `Record` is when, after reading the neighborhood, no
existing record is its target and the psyche is adding a genuinely new
meaning.

## ResolveClarification — fold a bad clarification back into its targets

Sometimes the mistake already happened: an agent added a standalone
`Clarification` record instead of editing the record(s) being clarified.
The cleanup operation is **`ResolveClarification`**.

A Spirit-maintenance worker starts by searching or inspecting the relevant
Spirit domain and referent records. It decides whether the psyche answer is a
clarification, supersession, new record, or non-Spirit material before it writes
anything. Non-Spirit material is anything that fails the intent gate: matter (a
single-mechanism, single-component, or architectural decision, or an instruction
about operating Spirit itself) and non-intent (information that directs nothing,
private or personal substance, ephemeral chatter). It belongs in code,
`ARCHITECTURE.md`, skills, or a private report — not the intent log
(`skills/intent-log.md` §"Intent is rare"). When resolving a mistaken standalone
clarification:

1. Lookup the mistaken clarification record and preserve its full text in the
   maintenance report or commit notes.
2. Find every target record it clarified. There may be more than one; search by
   referent, domain, keywords, testimony, and the clarification's own reasoning.
3. Use `ResolveClarification` to fold the useful substance into the target
   records and remove the standalone clarification in one operation.
4. Update manifested docs in the same pass.

The invariant: no active "clarification about a record" remains after
resolution; the truth lives on the records being clarified.

## Supersession is always explicit — only the psyche supersedes

A new psyche statement is the only thing that can override a prior psyche
statement. An agent encountering documented intent that seems wrong does
NOT supersede on its own authority — it asks the psyche
(`skills/intent-clarification.md`). This protection is load-bearing
against agent hallucination passing as psyche intent.

When a new psyche statement contradicts a prior recorded entry:

1. **Surface the contradiction inline, before recording.** Quote the
   prior verbatim and its certainty:

   > *"You said earlier (Spirit topic `<topic>`):*
   > *— `<prior verbatim quote>` — certainty `<prior certainty>`, record `<prior identifier>`*
   >
   > *Now you're saying `<new summary>`. Override the prior, or am I misreading?"*

2. **Wait for confirmation.** Three outcomes:

   | Psyche says | Action |
   |---|---|
   | "Yes, override" | Use `Supersede`: retire the prior identifier and provide the replacement entry in the same operation |
   | "No — the prior still applies, this refines it" | Use `Clarify` on the prior identifier; prior stays as the target, not as a sibling to a new active record |
   | "Both apply in different contexts" | Add the new entry; prior stays |

3. **On confirmed override:** use Spirit's typed supersession surface and
   name the supersession in the commit message
   (`intent: <topic> — psyche supersedes prior <slug>`). Spirit is the
   sole substrate. Do not simulate supersession by appending a
   free-standing `Correction` record beside the old active record.

## Negation shape

Negation is supersession where the psyche says a prior record is
*invalid*, not merely refined.

1. Look the prior record up by its identifier (`spirit "(Lookup abcd)"`).
2. Ask the psyche to confirm the old record is negated.
3. Use `Supersede` when a replacement truth exists, or `Retire` when the
   old record should simply stop being active. Do not delete the old
   record when lineage should stay visible.

## Removing a record — tombstone first

Spirit supports psyche-authorized removal: `spirit "(Remove abcd)"`
(the argument is the record's base36 identifier code, not a number).
Use it only for records that should **not remain at all** — mis-logged
working orders, or fully-stale records whose substance is rehomed. When
lineage should stay visible, use `Clarify`, `Supersede`, or `Retire`
instead.

Removal is **destructive and irreversible.** The record's key is
retracted from the sema-engine store and there is no undelete.

So **capture before you remove.** Before any `(Remove abcd)`, look the
record up by its identifier and record the full text into the removing
agent's report:

```sh
spirit "(Lookup abcd)"
```

Paste the resulting `RecordFound` entry into a tombstone appendix; the
report then IS the record of what was removed. An undocumented removal
once proved unrecoverable, while a tombstoned-first removal preserved
full text — capture first, then remove.

Stay conservative: when removability is uncertain, flag rather than
remove — over-removal is worse than under-removal. Lowering a record's
certainty to `Zero` marks it a removal candidate. `CollectRemovalCandidates`
archives exact-`Zero` candidates as compact summaries before retraction;
archive failure leaves records in the store. Use collection for reviewed
batches; use hard single-record `Remove` only after the tombstone is
captured.

## Verification — does the entry still apply?

Periodically — when sweeping a topic, or when an entry's substance
crosses your path — verify the recorded entry still matches reality.
Failure modes:

- **The workspace evolved past the entry.** A constraint set in a context
  that no longer exists. Ask the psyche for explicit retirement; don't
  assume.
- **The summary doesn't match the verbatim.** Agent rephrasing drift.
  Use `Clarify` or `ChangeRecord` to fix the active record to match the
  quote.
- **The certainty doesn't match the phrasing.** Re-read against
  `skills/intent-log.md` §"Certainty vocabulary"; correct if mismatched.
- **One record bundles claims of different certainty.** A record can carry
  a settled rule and a tentative design under one high-certainty summary.
  Split it with `Supersede`: retire the bundled record and replace it
  with separate entries at their earned certainties. Matters most when
  the psyche explicitly flags a clarification as low-certainty.

Corrections that fix the agent's transcription (not override psyche
intent) land directly through the edit operations — they're discipline
cleanup, not author overrides. Commit them as
`intent: clarified Spirit summary in <topic> to match verbatim`.

## Sweep — when and how

Trigger a sweep when:

- A Spirit topic grows large or query results become noisy. Small sweeps
  ride alongside `skills/context-maintenance.md`.
- An agent notices an entry that no longer matches the workspace.
- A major redesign lands — its premises likely supersede earlier intents
  and need explicit psyche confirmation.
- A context-maintenance pass finds older intent clearly contradicted by
  newer stronger intent. Such an agent may audit even old intent and
  **recommend** removal or supersession — but deletion stays reviewable
  and justified by the newer intent. Propose; do not execute unilaterally.
  The orchestrator or psyche authorizes removal after the contradiction is
  explicitly named.

How:

1. Read every Spirit entry in the topic.
2. For each: does it still apply? Summary match the verbatim? Certainty
   match the phrasing? Does one summary bundle sub-claims of differing
   certainty?
3. Entries that no longer apply: ask the psyche.
4. Transcription drift: correct the target record directly.
5. A genuinely noisy topic with two distinct sub-topics: carve a new
   Spirit topic per `skills/intent-log.md` §"When to actually split". The
   split is housekeeping, not author intent; history holds the lineage.

## When to skip recording in the first place

Some statements are too transient to log:

- "Let's try this and see" — pre-commitment exploration.
- "Maybe X, I'll think about it" — a `Minimum`-certainty note may be worth
  recording, but if the psyche commits to something else within the same
  conversation, skip the intermediate.

If you skip a borderline case and the psyche later asks "why isn't this in
Spirit?" — record it then.

## See also

- `skills/intent-log.md` — recording discipline; record shape; certainty
  vocabulary; topic granularity.
- `skills/context-maintenance.md` — workspace-wide sweep discipline.
