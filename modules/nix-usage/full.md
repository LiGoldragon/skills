# Skill — Nix usage

## Rules

Ask Nix what Nix knows. Inspect source, evaluate options, or query the daemon; do not search the store for configuration.

Use local checkout evaluations for diagnosis. Use a fetched or pinned installable when the evidence must prove another machine can reproduce the build. When a flake wrapper locks the dependency under test, use `--override-input <name> <local-path>` for quick local checks.

Keep store paths in variables. Do not paste raw store paths into chat, commits, skills, or docs; hashes drift and freeze stale state into prose.

## Command shapes

Inspect daemon-visible settings narrowly:

```sh
nix config show | rg '^(builders|builders-use-substitutes|max-jobs|substituters|trusted-public-keys|trusted-users)'
```

Build and keep the output path transient:

```sh
result=$(nix build <installable> --no-link --print-out-paths)
ls "$result"
```

Dry-run before a heavy build to read the miss/rebuild surface and catch remote-builder degradation early:

```sh
nix build <installable> --dry-run
```

Use a remote-builder smoke test only when that path is the claim. Force local slots to zero so the command fails instead of silently building locally:

```sh
nix build <installable> --no-link --print-build-logs   --option max-jobs 0   --print-out-paths
```

Use an uncached small derivation or `--rebuild` when substitutes would hide the builder path.

## Evidence

- `nix build` proves the build result or closure.
- `nix flake check` proves the repo's pure checks.
- `nix run .#<app>` proves a repo-exposed runner or one-shot tool.
- `nix run nixpkgs#<package> -- <arguments>` supplies a missing tool without mutating the environment.

Name the exact installable and whether the result is local diagnosis, fetched-build evidence, or remote-builder evidence.
