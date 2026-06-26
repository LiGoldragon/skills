# skills

Source repository for generated workspace skill surfaces.

The v1 pilot assembles the `intent-led-orchestration` skill from ordered NOTA manifests and markdown/NOTA source modules, then writes generated outputs into `/home/li/primary`.

Regenerate the pilot outputs:

```sh
SKILLS_UPDATE_SCHEMA_ARTIFACTS=1 cargo run -- intent-led-orchestration-generate.nota
```

Check generated outputs for drift:

```sh
cargo run -- intent-led-orchestration-check.nota
```

The checked-in manifests live under `manifests/intent-led-orchestration/`; source modules live under `modules/intent-led-orchestration/` and `modules/index/`.
