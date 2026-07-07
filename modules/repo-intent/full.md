# Skill — repository direction

## Rules

A repository's durable direction — what the psyche explicitly wants, rejects, values, or constrains for that project, and why the direction matters — lives in its `ARCHITECTURE.md` (or, when the direction is best expressed at a code location, a code stub with an explanatory comment). It is the per-repo guidance surface read on entry before code. There is no per-repo `INTENT.md`.

Direction is agent-written synthesis backed by psyche statements, not inference. `ARCHITECTURE.md` says what the system is and where the psyche wants it to go; keep the direction traceable to the statements that support it.

Record repo direction only when psyche-stated intent applies to the repository. A pure skeleton with no repo-specific direction may carry only structural architecture.

## Content

Include psyche-stated goals, constraints, principles, and anti-patterns for the project alongside the architecture they shape. Keep prose tight and traceable to the statement. If the psyche did not say it, do not add it.

Do not put audit history, task status, agent rationale, or speculative future project ideas into the direction prose. Put those in the owning source surface, `IDEAS.md`, or the tracker.

Use brief verbatim psyche quotes only when they are needed to preserve wording. Keep private material out of public repository surfaces.

## Maintenance

When new psyche intent affects a repo's design, implementation, tests, or operating direction, capture through the accepted intent path and manifest the repo-facing substance into `ARCHITECTURE.md` (or the relevant code stub) before the repo's guidance goes stale.

On substantial repo work, read `ARCHITECTURE.md` on entry, then check whether recent accepted intent affecting that repo is reflected there. Close gaps as part of the work, not as a separate future sweep.

Only the psyche can override repository direction. If the direction appears wrong but no superseding psyche statement exists, ask instead of rewriting by inference.
