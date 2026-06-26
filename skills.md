# Skill — skills repo

## Role

This repo owns source modules, NOTA manifests, and the Rust CLI that assembles generated skill surfaces for consuming workspaces.

## Working rules

- Keep module order explicit in each manifest; v1 has no imports, dependencies, or conditionals.
- Preserve prose. Normalization is limited to frontmatter placement, heading structure, relative links, and duplicate title handling.
- Treat duplicate headings as generation failures, not warnings.
- Do not add provenance headers to generated outputs.
- Regenerate `intent-led-orchestration` with `cargo run -- intent-led-orchestration-generate.nota`; check drift with `cargo run -- intent-led-orchestration-check.nota`.

## See also

- `INTENT.md` — project-specific intent and v1 constraints.
- `README.md` — command entry points and pilot file locations.
