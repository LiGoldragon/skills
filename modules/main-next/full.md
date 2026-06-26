# Skill — main and next branches

## Scope — code repos only, NOT primary

This model applies only to the code repositories under `/git/github.com/LiGoldragon/` (`horizon-rs`, `lojix`, `CriomOS`, the component triads, and so on). It does NOT apply to **primary**, the workspace coordination repository at `/home/li/primary`. On primary everyone works on `main` directly — edit, commit, push straight to `main`, with no `next` and no per-feature branches. See `skills/jj.md` §"Primary is always main — no branches, ever".

## The model

Each code repo keeps two long-lived branches.

- **`main`** — the integrated, canonical line. The operator owns it: creates, maintains, and integrates `next` into it. It is the line deploys and other repos pin.
- **`next`** — the development line. The designer works here. It is long-lived: one `next` per repo, not one branch per feature. The operator pulls `next` into `main` when the work is ready, and `next` continues from the new `main`.

## How the designer works

- **The designer's home is `next`.** Starting work on a repo, check out `next` in a `~/wt` worktree (`skills/feature-development.md`). If `next` does not exist, create it from `main` (`jj bookmark create next -r main@origin`, or branch off `main` in the worktree) and push it.
- **When `main` is locked or busy** — the operator is integrating, or another lane holds the claim — the designer keeps working on `next` and never blocks. That availability is the whole point.
- **When `main` is free**, the designer may use `main` directly for a small, safe change, but the default home is `next`.
- Commit to `next` and push; the operator integrates `next` → `main`. Inline jj messages only (`skills/jj.md`).

## How the operator works

- **The operator owns `main`** and integrates `next` into it when the work is ready. After integration, `next` advances from the new `main`.
- One `next` per repo carries the designer's in-flight work, so there are no scattered per-feature concept branches for the operator to hunt down. Concept branches still exist for genuinely-isolated experiments; the steady-state designer line is `next`.

## Why

Two coexisting targets keep the work surface clear: `main` is always the integrated truth, `next` is always where development lives. The designer never blocks on a locked `main`, and the operator always knows where to integrate from. When a component is busy or not production-ready, the designer makes the change work on `next` rather than stalling.

## See also

- `skills/feature-development.md` — the `~/wt` worktree mechanics `next` lives in.
- `skills/jj.md` — commit / push / integrate mechanics; inline messages.

The deployment-slot vocabulary (a repo named `<x>-next`, or a `next` release slot) names an in-flight authored *release line* — related but distinct from this per-repo `next` development *branch*.
