# skills

Source repository for generated workspace skill surfaces.

The generator assembles active skill modules from ordered NOTA manifests and writes generated outputs into a caller-supplied workspace root.

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

The checked-in manifests live under `manifests/skills/`, `manifests/index/`, and `manifests/intent-led-orchestration/`; source modules live under `modules/<name>/` and `modules/index/`. Deprecated role modules live under `skills/archive/` and are not emitted.

The full migration model for the current primary skill inventory lives at `manifests/migration/current-skills.nota`. It records active, archived, and deleted module status, Pi/Claude/Codex first-class target sets, and entrypoint command/prompt extras.
