# Skill — intent clarification

The intent layer — the deployed Spirit store as the raw
psyche-statement log, and each repo's `ARCHITECTURE.md` (or a code
stub with an explanatory comment) as its durable direction — is the
source of authoritative direction. The cost of one question is
bounded; building on invented intent compounds. Inferring is the
discipline breaking; asking is the discipline working.

## When to ask vs proceed

The test: *would the psyche have a specific opinion on this if
asked?* If yes, ask. If no, decide.

Proceed without asking — agent territory:

- Routine implementation: a variable name, an unexposed helper's
  shape, a choice between two equally-good libraries with no
  documented preference.
- Tactical sequencing within a task whose goal is already clear.
- Applying documented intent literally to a case it plainly covers.

Ask first when the decision would:

- *Contradict* documented intent.
- *Extend* documented intent into territory the psyche hasn't named.
- Require *inventing a principle* to proceed — principles are
  psyche territory.
- Set a precedent on something the psyche has been ambiguous about.
- *Propose changing* a documented intent (which requires explicit
  psyche confirmation regardless).

## How to ask

Structured questions cost the psyche seconds; open-ended ones cost
minutes. Never ask without options.

1. **Surface the gap concretely.** Quote any existing psyche intent
   with the file path it's recorded under; name what's recorded and
   what's not.
2. **Propose 2-4 options, each with its cost and gain.** The psyche
   picks one or redirects.
3. **State your lean** if you have one — accepting it is faster than
   the psyche generating from scratch.

When several decisions are ready at once, batch them as a numbered
slate the psyche can answer item by item, and record the graded state
each lands in: accepted, non-rejection (explicitly not acceptance —
track for later psyche review), rejection, or hedged lean (preserve
the hedge verbatim). See `skills/management.md`.

## After the psyche answers

The psyche just gave a new statement. Classify it before capturing:
only the rare, unbending orienting will that passes the intent test
in `skills/intent-log.md` is intent; matter — defaults, mechanisms,
single-component or architectural decisions, Spirit-operation — goes
to its owning surface, not Spirit. When a statement names
implementation-specific substance (a technology, format, component,
or mechanism), extract and verify the universal principle behind it
before recording, and route the substance itself to its owning
surface. When it is intent, record it per `skills/intent-log.md`. If it supersedes prior intent, follow the
supersession protocol in `skills/intent-maintenance.md`; if it
merely extends, record under a new sub-topic. Reflect
project-specific statements into the relevant repo's
`ARCHITECTURE.md` (or a code stub with an explanatory comment).

## When the psyche is not reachable

In order of preference:

1. **Defer.** Park the work needing the answer; continue with work
   that doesn't.
2. **Pick the most conservative, most-reversible option** if
   proceeding is required.
3. **State the assumption explicitly** where the work lands:
   *"Decided X assuming Y; will revise if psyche corrects."*

Don't silently invent intent or promote agent inference to
documented intent.
