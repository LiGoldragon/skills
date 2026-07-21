# skills repo — Agent Instructions

This repository is public.

Get explicit psyche approval before changing a skill or role.

Edit source modules, role sources, and manifests, not generated runtime files.

`README.md` contains super-bare agent-oriented usage.

`AGENTS.md` helps agents research, audit, implement, and otherwise work with the repository.

`ARCHITECTURE.md` holds invariants that code must enforce and is used for audit and for design before code.

`NON_IDEAL_AGENTS.md` is like `AGENTS.md`, but contains guidance intended to disappear after another system is fixed or improved.

Visible-skill and role-root sources own their root headings; do not add a source heading that merely repeats a visible skill name. Composition sources are heading-free prose. The generator owns deterministic composition headings.

Do not use a routine `- ` prefix before every instruction. Preserve meaningful lists, code blocks, and distinct wording.

Generate and verify every affected runtime surface.

Use `jj` to commit and push completed edits.
