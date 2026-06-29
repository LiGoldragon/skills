# Skill — repository intent file

## Rules

`INTENT.md` records what the psyche explicitly wants, rejects, values, or constrains for one repository. It is agent-written synthesis backed by psyche statements, not inference.

`ARCHITECTURE.md` says what the system is. `INTENT.md` says what the psyche wants the project to be and why that direction matters.

Create or update `INTENT.md` only when psyche-stated intent applies to the repository. A pure skeleton with no repo-specific intent may omit it.

## Content

Include psyche-stated goals, constraints, principles, and anti-patterns for the project. Keep prose tight and traceable to the statement. If the psyche did not say it, do not add it.

Do not put architecture, implementation doctrine, audit history, task status, or agent rationale in `INTENT.md`. Put those in the owning source surface.

Use brief verbatim psyche quotes only when they are needed to preserve wording. Keep private material out of public repository intent.

## Maintenance

When new psyche intent affects a repo's design, implementation, tests, or operating direction, capture through the accepted intent path and manifest the repo-facing substance into `INTENT.md` or `ARCHITECTURE.md` before the repo's guidance goes stale.

On substantial repo work, read `INTENT.md` and `ARCHITECTURE.md`, then check whether recent accepted intent affecting that repo is reflected there. Close gaps as part of the work, not as a separate future sweep.

Only the psyche can override repository intent. If the file appears wrong but no superseding psyche statement exists, ask instead of rewriting by inference.
