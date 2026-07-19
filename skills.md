# Skill — skills repo

## Role

This repo owns source modules, the active output manifest, the module dependency index, and the Rust CLI that assembles generated skill and role surfaces for consuming workspaces.

## Working rules

- Keep active generated outputs in `manifests/active-outputs.nota`. It lists only active `Skill` and `Role` outputs; absent means inactive.
- Keep source paths, module dependencies, and module kind in `manifests/module-dependencies.nota`. `RuntimeSkill` may emit as a skill, `RoleSource` is a role root, and `RoleComposition` is a generator-only role packet component.
- Keep target-specific module overlays in `manifests/target-module-insertions.nota`. Route by generated `OutputSurface`, not by model choice.
- Keep broad doctrine every role receives in `manifests/universal-role-modules.nota`. Do not repeat universal modules in every `Role` record.
- Keep supported model+effort profiles and their explicit total-order strengths in `manifests/model-catalog.nota`, exactly one model profile per active role in `manifests/role-model-assignments.nota`, and exactly one optional-skill record per active role in `manifests/role-optional-skills.nota`.
- Keep typed nested-role minimum models and exclusive leaf-role edges in `manifests/nested-role-relations.nota`. Manager stays the root Manager outside this relation.
- Reference optional skills by active skill output identifier. Optional means listed and loadable, not preloaded; every referenced skill supports every harness surface of its role.
- Keep generation metadata out of role prose. Skill and role source markdown carries reusable instruction body; manifests carry output identity, descriptions, tiers, target surfaces, model profiles, and optional skills.
- `manifests/skills-roster.nota` remains the Rust CLI compatibility input for legacy checks and archived/deleted skill modeling, but normal generation is driven by `manifests/active-outputs.nota` plus `manifests/module-dependencies.nota`.
- Put harness-specific frontmatter metadata in manifest records or the compatibility roster, not in reusable module prose.
- Model inactive source state outside the active manifest: archived modules live under `skills/archive/` with no emission, and deleted modules do not emit. Model active role-only components as `RoleComposition` instead of first-class skills.
- Treat `AgentsSkill` and `ClaudeSkill` as first-class target surfaces; `AgentsSkill` is the shared `.agents/skills/<name>/SKILL.md` surface used by Pi and Codex.
- Preserve prose. Normalization is limited to frontmatter placement, heading structure, relative links, and duplicate title handling.
- Treat duplicate headings as generation failures, not warnings.
- Do not add provenance headers to generated outputs.
- Regenerate all configured outputs with `nix run github:LiGoldragon/skills#generate-skills -- <workspace-root>`; check drift with `nix run github:LiGoldragon/skills#check-skills -- <workspace-root>`.
- Update hardcoded generation expectations in `tests/generation.rs` when active module membership, role composition, profiles, optional skills, nested-role relations, or universal role modules intentionally change.
- `generate-skills` may prune generated harness skill surfaces (`.agents/skills`, `.claude/skills`). Role packet target directories are path-owned rather than directory-owned: stale role cleanup uses `skills/generated-role-outputs.nota` and must preserve files that were not listed as generated role outputs. `check-skills` remains non-writing and reports stale archived/deleted skill outputs with update/regenerate/rerun guidance.

## Harness placement

Classify instruction before placement. General doctrine is independent of a
runtime API. Harness-specific doctrine depends on one harness's API, wrapper, or
interaction; concrete API fields belong only in a target module selected for that
harness, never in a shared base or role-composition module.

`manifests/active-outputs.nota` is authoritative for emitted outputs;
`manifests/module-dependencies.nota` maps source and dependency expansion; and
`manifests/target-module-insertions.nota` is authoritative for target modules.
The active skill targets are `AgentsSkill` at `.agents/skills/<name>/SKILL.md`
and `ClaudeSkill` at `.claude/skills/<name>/SKILL.md`. The active role targets
are `ClaudeAgent` at `.claude/agents/<role>.md`, `CodexAgent` at
`.codex/agents/<role>.toml`, and `PiAgent` at `.pi/agents/<role>.md`.

The current insertion index routes `harness-placement` into the
`skill-editor` `AgentsSkill` and `ClaudeSkill` outputs, the Claude-specific
`claude-management` module into `management` on `ClaudeSkill` and
`ClaudeAgent`, and the Codex-specific `codex-skill-loading` module into
`agent-feedback-loop` on `CodexAgent`. It has no active `PiAgent` insertion.
The generator expands dependencies, then appends matching target modules while
assembling each generated file.

Before adding a harness rule, verify its emitted target and insertion in those
manifests. After generation, verify that it appears in its scoped target and is
absent from general outputs; run `check-skills` for drift. When the required
surface does not exist, omit the rule and return the missing target support as a
placement gap. Do not invent a target, overlay, or example.

## See also

- `ARCHITECTURE.md` — generator structure, source surfaces, output targets, and invariants.
- `README.md` — command entry points and pilot file locations.
