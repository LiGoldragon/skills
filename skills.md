# Skill — skills repo

## Role

This repo owns source modules, NOTA manifests, and the Rust CLI that assembles generated skill surfaces for consuming workspaces.

## Working rules

- Keep module order explicit in each manifest; v1 has no imports, dependencies, or conditionals.
- Put harness-specific frontmatter in the output manifest, not in reusable module prose.
- Model migration status explicitly: active modules emit skills, archived modules live under `skills/archive/` with no emission manifests, and deleted modules do not emit.
- Treat Pi, Claude, and Codex as first-class targets; entrypoint command/prompt extras are separate from first-class skill emission.
- Preserve prose. Normalization is limited to frontmatter placement, heading structure, relative links, and duplicate title handling.
- Treat duplicate headings as generation failures, not warnings.
- Do not add provenance headers to generated outputs.
- Regenerate all configured outputs with `nix run github:LiGoldragon/skills#generate-skills -- <workspace-root>`; check drift with `nix run github:LiGoldragon/skills#check-skills -- <workspace-root>`.

## See also

- `INTENT.md` — project-specific intent and v1 constraints.
- `README.md` — command entry points and pilot file locations.
