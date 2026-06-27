# skills

Source repository for generated workspace skill surfaces.

The generator assembles active skill modules from a single NOTA roster and writes generated outputs into a caller-supplied workspace root.

Regenerate all configured outputs:

```sh
nix run github:LiGoldragon/skills#generate-skills -- <workspace-root>
```

Check generated outputs for drift:

```sh
nix run github:LiGoldragon/skills#check-skills -- <workspace-root>
```

For local Cargo iteration, set roots explicitly before passing a checked-in request:

```sh
SKILLS_SOURCE_ROOT=$PWD SKILLS_WORKSPACE_ROOT=<workspace-root> cargo run -- skills-check.nota
```

The checked-in roster lives at `manifests/skills-roster.nota`; source modules live under `modules/<name>/`. Deprecated role modules live under `skills/archive/` and are not emitted.

The roster records active, archived, and deleted module status, target skill surfaces, skill-index metadata, and entrypoint command/prompt extras. First-class output paths are derived from module name plus target surface: `AgentsSkill` emits `.agents/skills/<name>/SKILL.md` for both Pi and Codex, and `ClaudeSkill` emits `.claude/skills/<name>/SKILL.md`. Entrypoint extras derive `.claude/commands/<name>.md`, `.codex/prompts/<name>.md`, and `.codex/commands/<name>.md` from the entrypoint module name.

`generate-skills` prunes generated skill directories (`.agents/skills`, `.claude/skills`) and configured generated entrypoint extras before writing. `check-skills` is non-writing and reports stale generated outputs with regeneration guidance.
