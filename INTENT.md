# INTENT — skills

What the psyche has explicitly intended for this project.
Synthesised from psyche statements; not embellished.

## Goals

- The repository named `skills` is the source repository for generated skill files.
- A skill file is a source module.
- Generated final skill files are assembled from explicit ordered manifests.
- The v1 pilot starts only with `intent-led-orchestration` and proves module assembly while preserving current behavior.

## Constraints

- V1 has no module-declared dependencies or imports.
- V1 uses separate manifests per generated output and no conditionals.
- Assembly is ordered concatenation.
- Normalization changes only the structure required for valid output: one frontmatter block, heading levels, relative links, and duplicate title handling; prose is not rewritten.
- Duplicate headings or sections fail generation.
- Generated outputs carry no provenance headers.
- Generated outputs are written into consuming workspaces and committed there.
- The generator is a Rust CLI.
- Generator inputs are NOTA, and outputs are NOTA where applicable.
- Interfaces are defined with the existing schema system that lowers into Rust.

## Targets

- `skills/*.md`
- `skills/skills.nota`
- `.agents/skills/*/SKILL.md`
- `.claude/commands/*.md`
- `.codex/prompts/*.md`
- `.codex/commands/*.md`
