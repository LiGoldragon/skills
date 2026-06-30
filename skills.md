# Skill — skills repo

## Role

This repo owns source modules, the active output manifest, the module dependency index, and the Rust CLI that assembles generated skill and role surfaces for consuming workspaces.

## Working rules

- Keep active generated outputs in `manifests/active-outputs.nota`. It lists only active `Skill` and `Role` outputs; absent means inactive.
- Keep source paths, module dependencies, and module kind in `manifests/module-dependencies.nota`. `RuntimeSkill` may emit as a skill, `RoleSource` is a role root, and `RoleComposition` is a generator-only role packet component.
- Keep generation metadata out of role prose. Skill and role source markdown carries reusable instruction body; manifest records carry output identity, descriptions, tiers, and target surfaces.
- `manifests/skills-roster.nota` remains the Rust CLI compatibility input for legacy checks and archived/deleted skill modeling, but normal generation is driven by `manifests/active-outputs.nota` plus `manifests/module-dependencies.nota`.
- Put harness-specific frontmatter metadata in manifest records or the compatibility roster, not in reusable module prose.
- Model inactive source state outside the active manifest: archived modules live under `skills/archive/` with no emission, and deleted modules do not emit. Model active role-only components as `RoleComposition` instead of first-class skills.
- Treat `AgentsSkill` and `ClaudeSkill` as first-class target surfaces; `AgentsSkill` is the shared `.agents/skills/<name>/SKILL.md` surface used by Pi and Codex.
- Preserve prose. Normalization is limited to frontmatter placement, heading structure, relative links, and duplicate title handling.
- Treat duplicate headings as generation failures, not warnings.
- Do not add provenance headers to generated outputs.
- Regenerate all configured outputs with `nix run github:LiGoldragon/skills#generate-skills -- <workspace-root>`; check drift with `nix run github:LiGoldragon/skills#check-skills -- <workspace-root>`.
- `generate-skills` may prune generated harness skill surfaces (`.agents/skills`, `.claude/skills`). Role packet target directories are path-owned rather than directory-owned: stale role cleanup uses `skills/generated-role-outputs.nota` and must preserve files that were not listed as generated role outputs. `check-skills` remains non-writing and reports stale archived/deleted skill outputs with update/regenerate/rerun guidance.

## See also

- `ARCHITECTURE.md` — generator structure, source surfaces, output targets, and invariants.
- `README.md` — command entry points and pilot file locations.
