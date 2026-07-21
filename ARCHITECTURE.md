# skills — architecture

*Generator source for workspace skill and role surfaces.*

## TL;DR

This repository owns source modules, output manifests, and the Rust generator
that assembles harness-native skill and role files into consuming workspaces.
The active surface is manifest-driven: active outputs are listed in one NOTA
manifest, module source paths and dependencies live in sidecar NOTA indexes, and
generated files are written into the workspace root passed to the CLI.

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
- `manifests/active-outputs.nota`: active `Skill`, `FullAgentSkill`, and `FullAgentRole` outputs; presence means active.
- `manifests/module-dependencies.nota`: module identifier, source path, dependency module identifiers, and explicit source module kind (`RuntimeSkill`, `RoleSource`, `RoleComposition`, or `SharedComposition`).
- `manifests/target-module-insertions.nota`: target-specific module overlays keyed by base module and output surface.
- `manifests/universal-full-agent-modules.nota`: ordered `SharedComposition` modules included in every full-agent skill and role output.
- `manifests/model-catalog.nota`: canonical Claude and ChatGPT-family model+effort profiles with explicit total-order strengths.
- `manifests/role-model-assignments.nota`: exactly one Claude and one shared ChatGPT-family profile per active role.
- `manifests/role-optional-skills.nota`: validated active skill identifiers available for each role to load without preloading their bodies.
- `manifests/nested-role-relations.nota`: typed nested roles, target-relative minimum models, and exclusive allowed leaf-role edges.
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

The active source surface is manifest-owned: one active-outputs manifest lists
generated `Skill`, `FullAgentSkill`, and `FullAgentRole` outputs, where presence
means active. `Skill` is an ordinary visible skill. `FullAgentSkill` is the
visible full-agent skill surface, currently only `management`. `FullAgentRole`
is the role packet surface; every active role uses it. Sidecar indexes map module
identifiers to source paths, dependencies, target overlays, and universal
full-agent modules. Role sidecars assign validated model profiles and optional
skills. Nested-role relations add validated target-relative minimum models and
exclusive leaf-role delegation without changing Manager's root identity.

Assembly is ordered concatenation of source modules after manifest expansion.
A full-agent skill or role emits its root first, then the ordered universal
full-agent shared compositions, then role-specific included modules and
dependencies, generated rosters, and optional skills. Ordinary visible skills
do not receive universal compositions. Optional skill bodies remain outside the
packet until loaded. The catalog's typed model+effort strength determines the
strongest assignment; ordinary assignment wins an equal-strength minimum-model
tie, and a stronger nested minimum prevents downgrade.

Module dependencies are typed by module identifier rather than inferred from
markdown links or filesystem layout. `RuntimeSkill` modules may emit as
first-class skills, `RoleSource` modules are role roots, `RoleComposition`
modules are source-only and role-only, and `SharedComposition` modules are
source-only components accepted by full-agent skills and roles. A composition
can never emit standalone. The universal-full-agent manifest accepts only
`SharedComposition` entries. The generator retains identifier, path, and kind
through expansion: composition source prose is heading-free and receives a
deterministic generator-owned level-two heading. Visible skill and role roots
retain their source-owned root heading and receive no duplicate synthesized
name heading. Target insertions are data, not model choice: a base module,
output surface, and inserted module list determine which overlay appears in a
generated harness surface. Generation metadata such as descriptions, tiers,
frontmatter, target surfaces, role output identity, model profiles, optional
skills, nested-role edges, and minimum models live in manifests or the
compatibility roster.

## Ownership Boundaries

Source markdown owns reusable instruction body. Manifests own generated output
identity, target surfaces, descriptions, tiers, harness metadata, model
profiles, and optional-skill lists.

Generated outputs carry the harness-required frontmatter or TOML wrapper, but
they carry no provenance header. The source repository is the provenance.

Archived modules stay in `skills/archive/` and have no active manifest entry.
Deleted modules are modeled by compatibility checks and emit no surfaces.
`subagent-session-workflow` is obsolete and remains deleted.

## Constraints

- The generator is a Rust CLI.
- Generator inputs are NOTA where practical, including the active manifest,
  module dependency index, target module insertion index, and universal full-agent module manifest.
- Generator outputs are NOTA where applicable, including generated-role inventory files.
- Interfaces are schema-authored in `schema/assembly.schema`; Rust schema types are generated, not hand-authored in parallel.
- Normalization changes only structure required for valid output: one frontmatter block, root heading levels, relative links, and duplicate-title handling.
- Composition prose is preserved through generation; composition headings are generator-owned.
- Duplicate headings or sections fail generation.
- Generated outputs carry no provenance headers.
- Generated outputs are written into consuming workspaces and committed there.
- Role packet target directories are path-owned rather than directory-owned; stale role cleanup removes only paths listed in `skills/generated-role-outputs.nota`.

## Code Map

- `src/assembly.rs`: manifest loading, validation, module expansion, generated output planning, cleanup inventory, and rendering coordination.
- `src/markdown.rs`: markdown normalization and relative-link rebasing.
- `src/schema/assembly.rs`: generated Rust schema interface.
- `tests/generation.rs`: generation, stale cleanup, manifest, dependency, and validation witnesses.

## See Also

- `AGENTS.md` — repository operating rules.
- `README.md` — command entry points and generated surface overview.
