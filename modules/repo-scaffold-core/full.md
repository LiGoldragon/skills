# Module - repo scaffold core

## Scaffold Core Purpose

Repository scaffolding creates the starting shape for a new project or a named
structural rework. It establishes conventions that let later workers implement
inside the repo without guessing.

## Scaffold Project Boundary

A new repository is for a genuinely distinct project. Major rewrites,
experiments, mockups, repros, and alternate versions of an existing project use
a branch or worktree in the existing repository. Public is the default
visibility unless secrets, private data, unpublished third-party code, or an
explicit confidentiality constraint require private.

Use `ghq` for finding or fetching clones and `gh` for GitHub repository objects. Use `jj` for local history and pushing.

## Scaffold Initial Shape

Create only the guidance and build surfaces the accepted brief needs:
`AGENTS.md`, `INTENT.md` when psyche-stated project intent exists,
`ARCHITECTURE.md` when architecture is already known, repo-local `skills.md`
when the repo has specific working rules, build metadata, source layout, and
test entry points.

Do not invent product behavior, public APIs, storage schemas, deployment
promises, or role authority. Leave TODOs only for real downstream work that the
brief accepts.

## Scaffold Language Fit

Prefer the ecosystem already implied by the repo or brief. Rust scaffolds follow
the workspace Rust shape. Nix scaffolds expose checks through the flake rather
than ad hoc scripts. Names use full English words unless the surrounding
ecosystem has a canonical exception.

## Scaffold Handoff

Run the narrow scaffold check available: parser check, formatter, flake
evaluation, test discovery, or generated-output check. If the scaffold is
intentionally incomplete, name the missing piece and the first command expected
to pass once it exists.
