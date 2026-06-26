# Skill — double-implementation strategy

## Both tracks are branches — never new repos

A major architectural break is done on branches, not by spinning up a new
repository. A branch has no limits: wipe the whole tree, rebuild from scratch,
throw it away if it fails (see `skills/feature-development.md`). Routing the
tracks through new or `design-`-prefixed throwaway repos produces repo sprawl
and is forbidden. New repositories are for genuinely new projects only
(`skills/repository-management.md`).

## When to apply

Use this when a major break has produced multiple prototype branches exploring
the same problem from different angles, the design has structural complexity
(multiple layers, several open shape questions), both operator and designer have
load-bearing input, and single-track work would risk inference-driven drift. The
strategy makes the comparison structural: two agents in different roles produce
two artifacts proven convergent or proven divergent. Divergence surfaces
unresolved questions; convergence signals the design is settling.

Do NOT apply to routine changes within an existing repo, single-layer fixes
where one role is clearly authoritative, or work the psyche has already pinned to
one lane.

## The two tracks

### Operator track — `main`

Operator owns `main` in code repos. They amalgamate the best ideas from the
existing prototype branches into `main` as the baseline — reading the prototype
reports, citing which prototype contributed which piece, integrating into one
coherent substrate. They then integrate from the designer's `next` branch at
their own discretion, per slice on the merits: rebase the good parts,
cherry-pick, re-implement, or merge the designer branch as-is. Designer code is
not second-class — a clean designer `next` branch may merge to `main` as-is.

### Designer track — the `next` / feature branch

By default the designer works on the repo's standard `next` branch in a worktree
off the operator's `main` baseline, at
`~/wt/github.com/<owner>/<repo>/next/`. `next` is the repo's home for breaking
changes — schema reshapes, contract reworks, engine ports — not yet safe for
`main`. One standing `next` branch per repo, not a fresh
`designer-<topic>-<date>` branch per feature.

For a contrarian or from-scratch exploration, use another branch (a throwaway
feature branch in a worktree), not a new repo. That branch can wipe the tree and
rebuild a different shape from nothing — exactly what branches are for. Delete it
after the concept integrates or is abandoned.

## The comparison cadence

Periodically (psyche-triggered or end-of-slice):

1. Designer reads operator's `main` and flags differences from the designer's
   current iteration in a comparison report.
2. Operator reads the designer's `next` branch and flags differences from `main`
   in a comparison report.
3. Convergent decisions — both lanes agree — merge into `main` as integration.
4. Divergent decisions surface to the psyche as open shape questions
   (`skills/intent-clarification.md`).
5. The convergence report becomes the integration artifact.

## Cleaning up exploration branches

Exploration branches exist for the duration of the design iteration. Delete the
branch and any `~/wt` worktree after the concept integrates into `main`, the
design is explicitly retired, or the idea proves unworkable and is abandoned.
Don't let exploration branches accumulate — the point is iteration, not
permanent parallel infrastructure, and never permanent parallel repos.

## Why this works

- **Convergence as signal**: when two independent angles arrive at the same
  shape, the design is empirically reliable.
- **Divergence as forcing function**: differences are interview questions the
  psyche or the comparison report must answer.
- **Anti-drift**: single-track inference is mitigated because the other track
  would surface the inference rather than silently carrying it.

## See also

- `skills/feature-development.md` — the branch / `~/wt` worktree workflow.
- `skills/main-next.md` — the `main` + `next` discipline in code repos.
- `skills/intent-clarification.md` — escalating divergence to the psyche.
