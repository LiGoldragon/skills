# Skill — intent manifestation

Spirit captures raw psyche statements; guidance files are how those
statements actually shape agent behavior. Manifestation is the
bridge: capture is necessary but not sufficient — intent only shapes
what agents do once it lands in a file agents read.

The work: query Spirit, find entries whose substance hasn't yet
appeared in the right guidance file, edit the file to absorb it.
This is currently a periodic manual sweep; the eventual target is
`persona-spirit` querying the intent layer and surfacing unmanifested
entries as work. The discipline doesn't change when that lands — only
the trigger does (agent-initiated to spirit-surfaced).

## The destinations

| Guidance file | What lands there |
|---|---|
| `ESSENCE.md` (workspace) | Highest-certainty universal psyche statements — founding rules. Bar is high. |
| `AGENTS.md` | Per-keystroke hard overrides. Short, every-session-read. The "buck the bad agent habits" stuff. |
| `INTENT.md` | Workspace intent in prose, synthesised from Spirit; verbatim psyche quotes in italics. Read once on start; consult by topic. |
| `skills/<name>.md` | Topic- or workflow-specific discipline. Read when the topic comes up. |
| `<repo>/INTENT.md` | Per-repo prose synthesis of psyche intent — like ARCHITECTURE.md but for intent. |
| `<repo>/ESSENCE.md` (when exists) | Per-repo essential intent. |
| `<repo>/ARCHITECTURE.md` | The repo's structural shape; architectural decisions land here. |
| `<repo>/skills.md` | Per-repo capabilities and invariants. |

## The decision tree

For each intent record, ask in order:

1. **Universal-and-maximum psyche intent** — a statement foundational
   enough to stand as a rule of the whole way of working? →
   `ESSENCE.md`. Bar is high; few qualify.
2. **Per-keystroke override** — a rule agents need every session to
   buck a bad habit? → `AGENTS.md` Hard Overrides.
3. **Onboarding-shaped** — context for a fresh agent, read-once-on-
   start rather than per-keystroke? → `INTENT.md`.
4. **Topic-specific discipline** — applies when working a specific
   area (jj, NOTA, components, reports)? → the relevant
   `skills/<topic>.md`. If no skill exists, create one.
5. **Project-specific** — intent about one repo's direction? → that
   repo's `INTENT.md`.
6. **Architectural decision for one repo** → that repo's
   `ARCHITECTURE.md`.

One record can land in multiple destinations (a statement about NOTA
grammar lands in both `skills/nota-design.md` and the relevant nota
repo's docs).

## How to walk through

1. **Query the topic's Spirit records** in chronological order. The
   deployed Spirit store is the sweep substrate.
2. **For each record**, scan the guidance files you'd expect to carry
   its substance. If the destination already says what the intent
   says, it's manifested — nothing to do. Otherwise it needs work.
3. **Manifest by editing the destination**, carrying the substance
   into its prose with the verbatim-quoting convention below. Keep
   the destination's voice.
4. **Don't track 'manifested' explicitly.** No flag, no sibling file.
   The sweep is idempotent — re-running on already-manifested entries
   is a no-op.

A sweep can be by-topic (all records under one topic in one pass) or
by-destination (sweep into `AGENTS.md` from every topic). Pick by
what's in current attention.

## The verbatim-quoting convention

In `INTENT.md` and ESSENCE files, the body is prose composed mostly
of intent-log summaries, plus **verbatim psyche quotes** inline where
the exact wording is load-bearing. Mark verbatim quotes with markdown
italics — single asterisks:

```markdown
Mind owns state; orchestrate owns machinery. *"The orchestration
part, where it manages locks the way we manage locks now, really
belongs in orchestrate, so that doesn't need to go up to the mind.
But the memories actually do belong there."*
```

The italicised span is the psyche's words (post STT correction); the
surrounding prose is agent-composed from the log summaries. This
italicised verbatim is not the wholly-verbatim `quote` field of an
intent record (that lives in the NOTA log) — italics here flag
load-bearing pieces of original wording surviving the synthesis.

For multi-paragraph verbatim, use a blockquote with italics — the
only reliable way to carry italics across paragraph breaks, since
plain `*…*` closes at any blank line in CommonMark:

```markdown
> *first paragraph of verbatim, wrapped in italics inside a
> blockquote.*
>
> *second paragraph stays italicised because the blockquote wraps
> both.*
```

## Voice of each destination

Match the destination's voice when manifesting — the same statement
lands imperative in AGENTS.md and as descriptive synthesis in
INTENT.md.

- **`ESSENCE.md`** — declarative. *"Intent is primordial."*
- **`AGENTS.md` Hard Overrides** — imperative, terse. *"Spell every
  identifier as a full English word."*
- **`INTENT.md`** — descriptive, contextual synthesis.
- **Skills** — imperative discipline.
- **`<repo>/INTENT.md`** — descriptive synthesis of the project's
  intent.

## When to skip manifestation

Some records don't manifest into a guidance file:

- **Brainstorm-in-flight** (Medium / Minimum certainty) — wait until
  the psyche settles.
- **Acknowledgement / confirmation** records that re-state existing
  intent without new substance.
- **Single-conversation agent behavior** that doesn't generalise past
  the session.

The bar: does this record carry a rule, principle, decision, or
correction that future agents need? If yes, manifest. If no, leave it
in the log as historical record.

## When the destination is missing

- **New per-repo `INTENT.md`** — create per `skills/repo-intent.md`.
- **New skill** — create per `skills/skill-editor.md` if the intent
  is a discipline area not yet covered.
- **New per-repo `ESSENCE.md`** — propose to the psyche first. The
  psyche promotes to essence; the agent doesn't decide independently.

If the destination is unclear or the intent fits no existing
guidance-file shape, surface to the psyche per
`skills/intent-clarification.md`.

## See also

- `skills/intent-log.md` — capture discipline.
- `skills/intent-maintenance.md` — supersession + sweep cleanup.
- `skills/repo-intent.md` — per-repo INTENT.md shape.
