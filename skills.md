# Skill — skills repo

## Role

This repo owns source modules, the NOTA roster, and the Rust CLI that assembles generated skill surfaces for consuming workspaces.

## Working rules

- Keep module order explicit in the NOTA roster; v1 has no imports, dependencies, or conditionals.
- Put harness-specific frontmatter metadata in the roster, not in reusable module prose.
- Model migration status explicitly: active modules emit configured target surfaces, archived modules live under `skills/archive/` with no emission, and deleted modules do not emit.
- Treat `AgentsSkill` and `ClaudeSkill` as first-class target surfaces; `AgentsSkill` is the shared `.agents/skills/<name>/SKILL.md` surface used by Pi and Codex. Command/prompt extras are not current generated invocation surfaces.
- Preserve prose. Normalization is limited to frontmatter placement, heading structure, relative links, and duplicate title handling.
- Treat duplicate headings as generation failures, not warnings.
- Do not add provenance headers to generated outputs.
- Regenerate all configured outputs with `nix run github:LiGoldragon/skills#generate-skills -- <workspace-root>`; check drift with `nix run github:LiGoldragon/skills#check-skills -- <workspace-root>`.
- `generate-skills` may prune only generated harness skill surfaces (`.agents/skills`, `.claude/skills`). `check-skills` remains non-writing and reports stale archived/deleted skill outputs with update/regenerate/rerun guidance.

## See also

- `INTENT.md` — project-specific intent and v1 constraints.
- `README.md` — command entry points and pilot file locations.
