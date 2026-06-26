# Skill — repository intent file

Every repository carries an `INTENT.md` at its root: agent-written prose capturing what the psyche has explicitly intended for that project, 100% backed by psyche statements, no inference.

## What it is, against ARCHITECTURE.md

`ARCHITECTURE.md` says what the system IS — shape and invariants. `INTENT.md` says what the psyche wants this project to BE — stated goals, constraints, and principles for this specific project. Architecture docs say WHAT exists; `INTENT.md` says WHY. Decisions made months apart need a common reference for the psyche's vision, and an agent starting cold on a repo needs the project-specific intent before reading code.

## Where it lives

`<repo>/INTENT.md` — repository root, alongside `ARCHITECTURE.md`, `AGENTS.md`, `skills.md`. Uppercase to match the siblings.

## What goes in

- Project goals the psyche has explicitly stated.
- Project constraints the psyche has explicitly stated.
- Project principles the psyche has stated as applying specifically to this project.
- Things the psyche has explicitly said NOT to do for this project.

Each item is clear prose derived directly from psyche statements. The agent's role is synthesis without embellishment — restate what was said, do not elaborate what was implied.

Verbatim psyche quotes go in markdown italics (`*verbatim text*`) inline within the prose; for multi-paragraph verbatim, wrap italicised paragraphs in a blockquote (plain `*…*` italics close at a blank line in CommonMark, so the blockquote carries the span). The italics flag "the psyche's own words"; the surrounding prose is the agent's synthesis. See `skills/intent-manifestation.md`.

## What does NOT go in

- Agent inference. If the psyche didn't say it, it doesn't belong.
- Architectural shape — that's `ARCHITECTURE.md`.
- Implementation discipline — that's skills.
- Reports, decisions, audits — those live in `reports/`.
- Source verbatim quotes with full context — those live in Spirit intent records; `INTENT.md` carries terse rephrasing for clarity.

## Shape

Markdown prose, sections grouping intent by theme. A starter template (sections are illustrative — reshape to the actual statements):

```markdown
# INTENT — <project-name>

What the psyche has explicitly intended for this project.
Synthesised from psyche statements; not embellished.

## Goals
- <terse statement of a psyche-stated goal>

## Constraints
- <terse statement of a psyche-stated constraint>

## Principles
- <terse statement of a psyche-stated principle for this project>

## Anti-patterns
- <terse statement of what the psyche has said NOT to do>
```

## How to derive from psyche statements

1. Read Spirit intent records for entries mentioning this project.
2. For each, classify: goal, constraint, principle, or anti-pattern. Place it.
3. Restate clearly and tightly to what was said.
4. Draw no conclusions. If the psyche said "X is important" without saying why, record "X is important" — don't add "because Y" unless the psyche said Y.

## When to update

- A new psyche statement about this project lands in the intent log — update accordingly.
- An existing statement is contradicted by new intent (supersession per `skills/intent-maintenance.md`) — update after the supersession is confirmed.
- Periodic sweep — check every statement still aligns with the recorded psyche statements.

Only the psyche can override `INTENT.md`. An agent encountering content that seems wrong does NOT edit on inference; they consult the psyche (`skills/intent-clarification.md`).

## Continuous manifestation discipline

Intent must be manifested into per-repo files AT ALL TIMES, not just at the workspace level. This is the load-bearing rule for repo-scope intent.

The work-cycle obligation: when any agent starts work in a repo, the first verification is whether recent psyche intent affecting that repo is reflected in its `INTENT.md` (and, where architectural, its `ARCHITECTURE.md`). If not, manifest before proceeding or as part of the work cycle — never defer to a "later pass".

This applies whenever ANY new intent affects a specific repo's design, implementation, or test direction — not only intent explicitly scoped to that one repo. A workspace-wide rule (e.g. "every Rust function is a method on a non-ZST", "no `\n` escape in inline NOTA") that changes how this repo's code is authored MUST land in this repo's `INTENT.md` / `ARCHITECTURE.md`, so an agent reading only the repo's files knows the rule applies.

### At psyche-prompt time

When a psyche prompt lands containing intent that affects repo R:

1. Capture the intent through Spirit first (per the `AGENTS.md` hard override).
2. Identify whether the intent affects R's design, implementation, or test direction.
3. If yes, edit R's `INTENT.md` (and `ARCHITECTURE.md` if the intent has architectural shape) on a designer feature branch in `~/wt/github.com/<owner>/R/<branch>/`, alongside or immediately after the Spirit capture.
4. Don't gate on whether the prompt's primary subject was R — the discipline is about the repo's correctness as an agent-context surface, not the prompt's topic.

### On entering a repo

For any substantial work in a repo (worktree, edit, audit):

1. Read its current `INTENT.md` and `ARCHITECTURE.md`.
2. Query recent Spirit records (last session or two) for any affecting this repo.
3. Cross-check records against the files. Any record whose substance is missing, or whose framing has drifted from the record's text, is a manifestation gap.
4. Close the gap on the same feature branch as the work — manifestation is part of the work cycle, not a separate task.

### Failure mode this prevents

If intent lives only in Spirit, chat, and reports, an agent opening the repo sees stale framing in `INTENT.md` and codes to the stale shape. The repo's `INTENT.md` / `ARCHITECTURE.md` are the canonical agent-context surface for that repo; they must reflect current intent or they actively mislead.

## When to skip

A repo without psyche-stated intent doesn't need an `INTENT.md`. The file appears when the first psyche intent specific to the project lands — not before. A pure-skeleton repo, or one whose only purpose is mechanical (build artifact, codec, no psyche input on direction), can stay without one.

## See also

- `skills/intent-log.md` — workspace-level recording; Spirit is the source of new statements.
- `skills/intent-maintenance.md` — supersession protocol.
- `skills/architecture-editor.md` — the parallel ARCH discipline; ARCH for shape, INTENT for stated direction.
