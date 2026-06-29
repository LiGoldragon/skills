# Skill — Nix discipline

## Model services as NixOS modules

Long-running services are NixOS modules with typed options, systemd units, users, files, and firewall policy. Do not smuggle service shape into ad-hoc shell scripts or container-only definitions when the host is NixOS.

## Choose flake inputs deliberately

Use registry or remote inputs for shared dependencies. Do not commit local file inputs. For temporary integration checks, override inputs at command time instead of changing the committed flake.

Keep lock-file updates intentional and reviewable. Pin through the lock file, not duplicate hashes in `flake.nix`.

## Build and deploy from reproducible inputs

Commands that prove a change should build from the committed flake state or an explicit temporary override. Do not rely on an untracked checkout to be present on the target host.

Compiled artifacts are build outputs. Do not compile at service start. Runtime scripts may select, configure, and launch artifacts; they do not build them.

## Cargo dependencies in crane flakes

For Rust git dependencies, keep the dependency identity in Cargo metadata and let the lock file carry the revision. Do not add manual output hashes to flake code as the normal fix.

## Do not bake store paths into source

Source and docs do not depend on raw store paths. Refer to packages, derivations, options, and outputs by name. Inspect store paths with Nix commands when needed; do not search the store filesystem.

## Use Nix to get tools

If a tool is missing, run it through Nix or add it to the dev shell. Do not install ad-hoc host packages to make a build pass.

## Checks are the gate

`nix flake check` is the default pre-commit proof for Nix changes. Add narrower package, module, or VM checks when they prove the edited surface better. Stateful deployment checks must name the host, target, and rollback plan.

## Keep evaluation separate from activation

Evaluation and build checks prove the derivation graph. Activation and deployment checks prove host behavior. Do not treat a successful build as evidence that a service migrated safely.

## Module shape

Keep options typed and documented. Put defaults near option definitions. Keep assertions close to the invariant. Prefer small modules with clear imports over one large conditional module.

## Prefer data over shell

Use Nix values, options, derivations, and systemd fields for structure. Shell belongs at the edge where a program must be invoked, and it stays small enough to audit.
