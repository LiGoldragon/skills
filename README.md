# skills

Source repository for generated workspace skill and role surfaces.

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

The V1 source-side active output manifest lives at `manifests/active-outputs.nota`; module paths and dependencies live at `manifests/module-dependencies.nota`. The CLI consumes that active manifest for normal generation. The compatibility roster at `manifests/skills-roster.nota` remains parseable for legacy checks and archived/deleted skill modeling.

Source skill modules live under `modules/<name>/`. Role modules live under `roles/<name>/`. Archived modules live under `skills/archive/` and are not emitted.

The active manifest records first-class `Skill` and `Role` outputs. Skill output paths are derived from module name plus target surface: `AgentsSkill` emits `.agents/skills/<name>/SKILL.md` for both Pi and Codex, and `ClaudeSkill` emits `.claude/skills/<name>/SKILL.md`.

Role outputs emit harness-native worker packets:

- Claude: `.claude/agents/<role>.md`
- Codex: `.codex/agents/<role>.toml`
- Pi: `.pi/agents/<role>.md`

Primary discovery currently emits `skills/skills.nota`, whose entries point at harness-native generated skill files. Primary `skills/*.md` skill bodies are not emitted when no consuming harness needs them.

`generate-skills` prunes generated skill directories (`.agents/skills`, `.claude/skills`) before writing. Role packet directories are not whole-directory pruned; stale role cleanup uses `skills/generated-role-outputs.nota` so only previously generated role paths are removed. `check-skills` is non-writing and reports stale generated outputs with regeneration guidance.
