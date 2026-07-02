# Skill — operating system operations

## Rules

Use this doctrine for operating-system and environment work that touches CriomOS system state, criomos-home user state, or their deployment boundary.

Operate from pushed, reproducible inputs. Treat CriomOS as the system source identity and criomos-home as the home/environment source identity. Pin the exact revision in the flake reference you deploy; the deployed daemon carries no revision-policy field to resolve a branch for you.

Before changing a host, name the target cluster, node, deployment shape (`Home` or `System`), requested action, the exact source revision, builder choice, rollback expectation, and post-activation evidence.

Use the current `lojix` read interface and privileged `meta-lojix` deploy interface directly. Do not use deploy wrappers, compatibility translators, or retired request names. The deployed daemon accepts exactly two `DeployRequest` variants, `Home` and `System`, and rejects `Host`, `CompleteHost`, `BaseHost`, and `UserEnvironment`.

## Lojix interface

Read current generations for a node:

```sh
lojix "(Query (ByNode (<cluster> <node> None)))"
```

Deploy a home/environment change. This is the standard path for shipping a component such as spirit:

1. Push the changed component to its remote at the intended revision.
2. Repoint the criomos-home input for that component to that exact revision and push criomos-home. Do not `nix flake update`; it resolves the branch head (`main`), not the intended revision.
3. Submit the home deploy against the pushed criomos-home revision:

```sh
meta-lojix "(Deploy (Home (<cluster> <node> <user> <proposal-source> github:LiGoldragon/CriomOS-home/<rev> <home-mode> <builder> <substituters>)))"
```

Concretely:

```sh
meta-lojix "(Deploy (Home (goldragon ouranos li <proposal-source> github:LiGoldragon/CriomOS-home/<rev> Activate None [])))"
```

`HomeDeployment` holds eight positional fields: cluster, node, user, proposal source, criomos-home flake reference, home mode, builder, and extra substituters. `<home-mode>` is `Activate` to build and activate, or `Build`. `<builder>` is `None` or `(Some <builder-node>)`. `<substituters>` is a typed list, `[]` when none.

Deploy a full system change:

```sh
meta-lojix "(Deploy (System (<cluster> <node> <deployment-kind> <proposal-source> <criomos-flake-ref> <system-action> <builder> <substituters> <trailing-option>)))"
```

`SystemDeployment` holds nine positional fields: cluster, node, deployment kind, proposal source, CriomOS flake reference, system action, builder, extra substituters, and a trailing option. `<deployment-kind>` is `FullOs` or `HomeOnly`. `<system-action>` is `Switch`. `<builder>` and `<substituters>` match the home shape. The trailing option is `None` or `(Some <value>)`.

`meta-lojix` returns when the daemon admits a request. Admission is not proof of build, copy, activation, or profile success.

## Activation checks

After submit, query the node until the expected store path becomes current, or a rejection or failure is visible:

```sh
lojix "(Query (ByNode (<cluster> <node> None)))"
```

Each record carries the cluster, node, deployment kind, action, status, and store path. Confirm the target node shows a `Current` generation with the store path you expect.

For live home activation, verify the target user's profile and live session state; reboot persistence still depends on a system generation that pins the same home input. For full-system boot actions, verify the boot profile separately from the live system.

Reload Niri configuration explicitly after a successful home activation when the task requires a live compositor refresh:

```sh
niri msg action load-config-file
```

Do not hide Niri reload inside deploy tooling.
