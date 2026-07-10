# skills

Source repository for generated workspace skill and role surfaces.

The generator assembles active modules from a NOTA output manifest and a module
dependency index, then writes generated outputs into a caller-supplied workspace
root.

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

The source-side active output manifest lives at
`manifests/active-outputs.nota`; module paths, dependencies, and source module
kind live at `manifests/module-dependencies.nota`; target-specific overlays live
at `manifests/target-module-insertions.nota`. Canonical model support lives in
`manifests/model-catalog.nota`; every active role has one profile in
`manifests/role-model-assignments.nota` and one optional-skill list in
`manifests/role-optional-skills.nota`. The CLI consumes these inputs for normal
generation. The compatibility roster at
`manifests/skills-roster.nota` remains parseable for legacy checks and
archived/deleted skill modeling.

Source skill modules live under `modules/<name>/`. Role modules live under `roles/<name>/`. Archived modules live under `skills/archive/` and are not emitted. Generated headings strip source composition prefixes such as `Skill -`, `Module -`, and `Role -`; visible runtime titles are the human title only. A source section headed exactly `## Source Maintenance Notes` is source-only and is stripped from that heading through the end of the fragment.

The active manifest records first-class `Skill` and `Role` outputs. Skill output paths are derived from module name plus target surface: `AgentsSkill` emits `.agents/skills/<name>/SKILL.md` for both Pi and Codex, and `ClaudeSkill` emits `.claude/skills/<name>/SKILL.md`. Source module kinds are explicit: `RuntimeSkill` modules may emit as skills, `RoleSource` modules are role roots, and `RoleComposition` modules are generator-only role packet components that can be included in roles but cannot emit as skills.

Target module insertions append extra modules for a named base module only when
the generator is producing the named output surface, such as a Claude-only
overlay for `ClaudeSkill` or `ClaudeAgent`.

Repository guidance stays repo-local. Keep ordinary operating rules in
`AGENTS.md`, architecture and ideal direction in `ARCHITECTURE.md`, overview in
`README.md`, and required workaround debt in `NON_IDEAL_AGENTS.md`.

Role outputs emit harness-native worker packets:

- Claude: `.claude/agents/<role>.md`
- Codex: `.codex/agents/<role>.toml`
- Pi: `.pi/agents/<role>.md`

Generated role packets carry role source plus curated preloaded modules,
dependency-expanded modules, provider-specific model and effort fields, and a
validated list of optional skills. Optional skill bodies are loadable but not
preloaded. Claude receives `model` and `effort`; Pi receives a
provider-qualified `model` and `thinking`; Codex receives `model` and
`model_reasoning_effort`. Primary `skills/*.md` skill bodies are not emitted
when no consuming harness needs them.

`generate-skills` prunes generated skill directories (`.agents/skills`, `.claude/skills`) before writing. Role packet directories are not whole-directory pruned; stale role cleanup uses `skills/generated-role-outputs.nota` so only previously generated role paths are removed. `check-skills` is non-writing and reports stale generated outputs with regeneration guidance.
