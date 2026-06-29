# skills — architecture

*Generator source for workspace skill and role surfaces.*

## TL;DR

This repository owns source modules, output manifests, and the Rust generator
that assembles harness-native skill and role files into consuming workspaces.
The active surface is manifest-driven: active outputs are listed in one NOTA
manifest, module source paths and dependencies live in a sidecar NOTA index,
and generated files are written into the workspace root passed to the CLI.

The generator treats instruction prose as reusable source material. Harness
metadata and output identity live in manifests, while markdown modules stay
focused on the instruction body they contribute to generated files.
Generated role packets are the normal runtime doctrine bundle: the role body
is emitted with curated included modules and dependency-expanded modules, so
workers do not discover doctrine through a runtime index.

## Source Surfaces

- `modules/<name>/full.md`: source modules for workspace skills.
- `roles/<name>/full.md`: source modules for generated worker role packets.
- `skills/archive/`: archived source material with no active emission.
- `manifests/active-outputs.nota`: active `Skill` and `Role` outputs; presence means active.
- `manifests/module-dependencies.nota`: module identifier, source path, dependency module identifiers, and explicit source module kind (`RuntimeSkill`, `RoleSource`, or `RoleComposition`).
- `manifests/skills-roster.nota`: compatibility input for legacy checks and archived/deleted module modeling.
- `schema/assembly.schema`: schema-authored generator interface source.
- `src/schema/assembly.rs`: generated Rust interface from `schema/assembly.schema`.

## Output Targets

Skill targets:

- `AgentsSkill`: `.agents/skills/<name>/SKILL.md`, shared by Pi and Codex.
- `ClaudeSkill`: `.claude/skills/<name>/SKILL.md`.

Role targets:

- `ClaudeAgent`: `.claude/agents/<role>.md`.
- `CodexAgent`: `.codex/agents/<role>.toml`.
- `PiAgent`: `.pi/agents/<role>.md`.

Derived inventory:

- `skills/generated-role-outputs.nota`: stale generated role cleanup inventory.

## Assembly Model

Assembly is ordered concatenation of source modules after manifest expansion.
For skills, the active skill's module expands through the dependency index. For
roles, the role body is emitted first, followed by any included modules and
their dependencies. A generated role packet is the curated runtime bundle for
normal role work; additional doctrine is named by the prompt, role packet,
dispatch envelope, or local context rather than discovered through a generated
index.

Module dependencies are typed by module identifier rather than inferred from
markdown links or filesystem layout. The dependency index also carries source
module kind. `RuntimeSkill` modules may emit as first-class skills,
`RoleSource` modules are role roots, and `RoleComposition` modules are
generator-only role packet components that may be dependency-expanded into
roles but cannot be emitted as runtime skills. Generation metadata such as
descriptions, tiers, frontmatter, target surfaces, and role output identity
live in the active manifest or compatibility roster.

## Ownership Boundaries

Source markdown owns reusable instruction body. Manifests own generated output
identity, target surfaces, descriptions, tiers, and harness metadata.

Generated outputs carry the harness-required frontmatter or TOML wrapper, but
they carry no provenance header. The source repository is the provenance.

Archived modules stay in `skills/archive/` and have no active manifest entry.
Deleted modules are modeled by compatibility checks and emit no surfaces.
`subagent-session-workflow` is obsolete and remains deleted.

## Constraints

- The generator is a Rust CLI.
- Generator inputs are NOTA where practical, including the active manifest and module dependency index.
- Generator outputs are NOTA where applicable, including generated-role inventory files.
- Interfaces are schema-authored in `schema/assembly.schema`; Rust schema types are generated, not hand-authored in parallel.
- Normalization changes only structure required for valid output: one frontmatter block, heading levels, relative links, and duplicate-title handling.
- Prose is preserved through generation.
- Duplicate headings or sections fail generation.
- Generated outputs carry no provenance headers.
- Generated outputs are written into consuming workspaces and committed there.
- Role packet target directories are path-owned rather than directory-owned; stale role cleanup removes only paths listed in `skills/generated-role-outputs.nota`.

## Code Map

- `src/assembly.rs`: manifest loading, module expansion, generated output planning, cleanup inventory, and rendering orchestration.
- `src/markdown.rs`: markdown normalization and relative-link rebasing.
- `src/schema/assembly.rs`: generated Rust schema interface.
- `tests/generation.rs`: generation, stale cleanup, manifest, dependency, and validation witnesses.

## See Also

- `skills.md` — how to work in this repository.
- `README.md` — command entry points and generated surface overview.
