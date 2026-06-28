# INTENT — skills

What the psyche has explicitly intended for this project.
Synthesised from psyche statements; not embellished.

## Goals

- The repository named `skills` is the source repository for generated skill files.
- A skill file is a source module.
- Generated final skill files are assembled from an explicit ordered NOTA roster.
- Generated V1 skill and role outputs are governed by one active NOTA manifest with distinct `Skill` and `Role` records; presence means active output.
- The v1 pilot starts only with `intent-led-orchestration` and proves module assembly while preserving current behavior.
- All current workspace skills should migrate into the generator/module system if the full weave proves workable.
- Pi, Claude, and Codex are first-class generation targets; the `.agents/skills/<name>/SKILL.md` surface is shared by Pi and Codex.

## Constraints

- V1 uses a sidecar NOTA module dependency index for dependencies only: module id, source path, and dependency module ids.
- V1 uses one active NOTA manifest as the source of truth for active generated Skill and Role outputs, with inactive outputs omitted rather than listed.
- A skill file is a source module; modules assemble through explicit roster entries.
- Module source files may carry reusable metadata, but harness-specific frontmatter belongs in manifest records or the compatibility roster.
- Every active migrated module emits first-class `AgentsSkill` and `ClaudeSkill` surfaces unless it is explicitly internal/building-block only; `AgentsSkill` is shared by Pi and Codex.
- Command/prompt extra surfaces are not current generated invocation surfaces; `intent-led-orchestration` remains available through first-class skill surfaces only.
- Module status distinguishes active, archived, and deleted modules; archived modules live under `skills/archive/` with no emission manifests.
- `subagent-session-workflow` is obsolete and must not migrate or emit harness surfaces.
- Assembly is ordered concatenation.
- Normalization changes only the structure required for valid output: one frontmatter block, heading levels, relative links, and duplicate title handling; prose is not rewritten.
- Duplicate headings or sections fail generation.
- Generated outputs carry no provenance headers.
- Generated outputs are written into consuming workspaces and committed there.
- The generator is a Rust CLI.
- Generator inputs are NOTA, and outputs are NOTA where applicable.
- Interfaces are defined with the existing schema system that lowers into Rust.

## Targets

- Pi first-class skills: `.agents/skills/*/SKILL.md`
- Claude first-class skills: `.claude/skills/*/SKILL.md`
- Codex first-class skills: the Codex skill surface selected by the migration bead
- Primary discovery index: `skills/skills.nota`, generated from the roster, whose entries point at harness-native generated skill files
- Primary `skills/*.md` skill bodies are not emitted when no consuming harness needs them
