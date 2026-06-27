# INTENT — skills

What the psyche has explicitly intended for this project.
Synthesised from psyche statements; not embellished.

## Goals

- The repository named `skills` is the source repository for generated skill files.
- A skill file is a source module.
- Generated final skill files are assembled from an explicit ordered NOTA roster.
- The v1 pilot starts only with `intent-led-orchestration` and proves module assembly while preserving current behavior.
- All current workspace skills should migrate into the generator/module system if the full weave proves workable.
- Pi, Claude, and Codex are first-class generation targets; the `.agents/skills/<name>/SKILL.md` surface is shared by Pi and Codex.

## Constraints

- V1 has no module-declared dependencies or imports.
- V1 uses one NOTA roster/config as the source of truth for active, archived, and deleted status and target emission, with no conditionals.
- A skill file is a source module; modules assemble through explicit roster entries.
- Module source files may carry reusable metadata, but harness-specific frontmatter belongs in the roster.
- Every active migrated module emits first-class `AgentsSkill` and `ClaudeSkill` surfaces unless it is explicitly internal/building-block only; `AgentsSkill` is shared by Pi and Codex.
- Only selected entrypoint modules emit command/prompt extras; the only current entrypoint is `intent-led-orchestration`.
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
- Claude command extras for entrypoints: `.claude/commands/*.md`
- Codex prompt/command extras for entrypoints: `.codex/prompts/*.md` and `.codex/commands/*.md`
- Primary discovery index: `skills/skills.nota`, generated from the roster, whose entries point at harness-native generated skill files
- Primary `skills/*.md` skill bodies are not emitted when no consuming harness needs them
