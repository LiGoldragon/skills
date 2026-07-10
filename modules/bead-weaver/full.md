# Module - bead weaver

## Rules

Use beads only after intent is aligned enough to decompose into independently actionable work. Do not file speculative beads to force unresolved design shape or split a clear routine linear operation that one implementation worker can complete.

A weave is a dependency graph of discrete jobs. Each bead needs a clear goal, definition of done, evidence signal, constraints, and out-of-scope boundary. Do not file beads for permanent disciplines, broad concerns, or unresolved decisions; land those in the owning guidance or architecture surface.

Build from outcomes backward:

1. Name the final observable outcome.
2. Name the smallest proof that shows it works.
3. Name prerequisites that can ship independently.
4. Put architecture or schema decisions before implementation beads that would otherwise guess.
5. Put verification beads after the build beads they witness.

Prefer a thin first slice over a broad backlog.

## Filing

Create descriptive titles and wire dependencies explicitly:

```sh
bd create "<title>" -t task -p <priority> -d "<description>"
bd dep <blocker-bead> --blocks <blocked-bead>
```

File blockers first so dependency commands read in work order. Read the graph back with `bd show` or `bd list` and fix unclear descriptions immediately.

Run `bd` commands sequentially, not through parallel tool calls. If embedded
Dolt reports the exclusive `.beads/embeddeddolt` lock, wait for the owning
operation to finish and retry the same command; do not spawn concurrent retries.

Do not claim `.beads/`. Treat an Orchestrate `.beads/` claim as invalid agent policy state; force-release or remove it instead of treating it as a lock. If you begin working a bead after filing it, claim the task if the workspace uses claims; filing alone is not a claim.
