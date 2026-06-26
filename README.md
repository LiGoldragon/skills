# skills

Source repository for generated workspace skill surfaces.

The v1 pilot assembles the `intent-led-orchestration` skill from ordered NOTA manifests and markdown/NOTA source modules, then writes generated outputs into a caller-supplied workspace root.

Regenerate the pilot outputs:

```sh
nix run github:LiGoldragon/skills#generate-intent-led-orchestration -- <workspace-root>
```

Check generated outputs for drift:

```sh
nix run github:LiGoldragon/skills#check-intent-led-orchestration -- <workspace-root>
```

For local Cargo iteration, set roots explicitly before passing a checked-in request:

```sh
SKILLS_SOURCE_ROOT=$PWD SKILLS_WORKSPACE_ROOT=<workspace-root> cargo run -- intent-led-orchestration-check.nota
```

The checked-in manifests live under `manifests/intent-led-orchestration/`; source modules live under `modules/intent-led-orchestration/` and `modules/index/`.

The full migration model for the current primary skill inventory lives at `manifests/migration/current-skills.nota`. It records active, archived, and deleted module status, Pi/Claude/Codex first-class target sets, and entrypoint command/prompt extras without migrating every skill module in this bead.
