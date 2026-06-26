# Skill — Nix usage

Use this skill when running Nix commands: inspecting daemon configuration,
forcing a build through the remote builder, capturing outputs, or deciding which
command surface counts as evidence.

## Inspect the active configuration

Ask Nix what the daemon sees; never search the store for configuration. If a
value is Nix-controlled, inspect the source checkout, evaluate the option, or
ask the daemon.

```sh
nix config show | rg '^(builders|builders-use-substitutes|max-jobs|substituters|trusted-public-keys|trusted-users)'
```

`builders = @/etc/nix/machines` means the daemon reads its remote builder list
from that file; inspect it directly when you need the machine line.

## Push first; build from github

The normal build/check/deploy witness is a pushed flake reference, not the local
checkout. A local-checkout evaluation only proves your own machine; it is not
evidence another machine can reproduce the result, and it is not a deploy
surface. Commit, move the bookmark (usually `main` for operator integration
work), push, then build the `github:` ref with `--refresh`.

```sh
jj describe -m '<message>'
jj bookmark set main -r @
jj git push --bookmark main
nix build --refresh --no-link --print-out-paths \
  github:LiGoldragon/CriomOS-home/main#homeConfigurations.li.activationPackage
```

Local evaluation is for inner-loop diagnosis only. If a build is slow or needs a
remote builder, still push first, then add remote-builder options to the
`github:` installable.

## Remote builder smoke test

To force a build onto the remote builder, set local build slots to zero on the
command. With `max-jobs = 0`, a build that cannot reach a remote builder fails
rather than falling back to local work — which is what makes it a real probe.

```sh
result=$(nix build <installable> --no-link --print-build-logs \
  --option max-jobs 0 \
  --option builders '@/etc/nix/machines' \
  --print-out-paths)
```

Use `--rebuild` or an uncached small derivation to prove the builder path
instead of accepting a substitute. The success witness is Nix's own build log:
it names the build happening on the remote builder
(`ssh-ng://nix-ssh@prometheus.goldragon.criome`), then copies the result back.

`max-jobs = 0` is a per-command remote-only lever; leave the daemon default
alone unless the host should never build locally.

Use daemon-scheduled `nix build` as the smoke test, not a direct user-shell
probe (`ssh nix-ssh@…` or `nix store info --store ssh-ng://…`). Those check the
caller's SSH credentials, not the daemon's machine entry, and can fail even when
remote builds work.

## Store paths stay in variables

Keep returned store paths in a shell variable; never paste raw store paths into
chat, reports, skills, commit messages, or architecture docs. Store hashes drift
on rebuild, so any prose that freezes a path is stale immediately.

```sh
result=$(nix build <installable> --no-link --print-out-paths)
ls "$result"
```

## Which command is evidence

- `nix build` — when the build result or closure is the evidence.
- `nix flake check` — when a repo's pure test suite is the evidence.
- `nix run .#<app>` — when the repo exposes a stateful runner or one-shot tool
  as an app.
- `nix run nixpkgs#<package> -- <arguments>` — when a tool is missing from `PATH`.

A direct build of a CriomOS `nixosConfigurations.target` without lojix-projected
inputs is not a real deployment check. For that path, use lojix and the target
repo's skill.

## See also

- `skills/nix-discipline.md` — flake inputs, lock discipline, missing tools,
  `nix flake check`.
- `skills/testing.md` — Nix-backed test surfaces.
- CriomOS's `skills.md` — host deployment and NixOS module rules.
