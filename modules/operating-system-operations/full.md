# Skill — operating system operations

## Rules

Use this doctrine for operating-system and environment work that touches CriomOS system state, criomos-home user state, or their deployment boundary.

Operate from pushed, reproducible inputs. Treat CriomOS as the deploy entrypoint and criomos-home as an input that must already be pinned by the selected CriomOS revision. Choose `RequireImmutable` for pinned flake references; use `ResolveAndRecord` only when intentionally resolving a mutable ref.

Before changing a host, name the target cluster, node, deployment shape (`UserEnvironment` or `Host`), requested action, source revision policy, exact source revision, builder choice, rollback expectation, and post-activation evidence.

Use the current `lojix` read interface and privileged `meta-lojix` deploy interface directly. Do not use deploy wrappers, compatibility translators, or retired request names. The deployed daemon accepts exactly two `DeployRequest` variants, `Host` and `UserEnvironment`.

## Lojix interface

Read current generations for a node:

```sh
lojix "(Query (ByNode (<cluster> <node> None)))"
```

Deploy a user environment change. This is the standard path for shipping a component such as spirit:

1. Push the changed component to its remote at the intended revision.
2. Repoint the criomos-home input for that component to that exact revision, then ensure the selected CriomOS revision pins that criomos-home revision. Do not `nix flake update`; it resolves the branch head (`main`), not the intended revision.
3. Submit the deploy against the selected CriomOS revision:

```sh
meta-lojix "(Deploy (UserEnvironment (<cluster> <node> <user> <proposal-source> <criomos-flake-ref> <user-environment-action> <source-revision-policy> <builder> <substituters>)))"
```

Concretely:

```sh
meta-lojix "(Deploy (UserEnvironment (goldragon ouranos li <proposal-source> github:LiGoldragon/CriomOS/<rev> ActivateNow RequireImmutable None [])))"
```

`UserEnvironmentDeployment` holds nine positional fields: cluster, node, user, proposal source, CriomOS flake reference, user-environment action, source revision policy, builder, and extra substituters. `<user-environment-action>` is `Realize`, `SetProfile`, or `ActivateNow`. `<source-revision-policy>` is `RequireImmutable` or `ResolveAndRecord`. `<builder>` is `None` or `(Some <builder-node>)`. `<substituters>` is a typed list, `[]` when none.

Deploy a host change:

```sh
meta-lojix "(Deploy (Host (<cluster> <node> <host-composition> <proposal-source> <criomos-flake-ref> <host-action> <source-revision-policy> <builder> <substituters> <build-attribute>)))"
```

`HostDeployment` holds ten positional fields: cluster, node, host composition, proposal source, CriomOS flake reference, host action, source revision policy, builder, extra substituters, and build attribute. `<host-composition>` is `CompleteHost` or `BaseHost`. `<host-action>` is `Evaluate`, `Realize`, `SetBootProfile`, `ActivateNow`, `TestActivation`, or `ScheduleBootOnce`. `<source-revision-policy>`, `<builder>`, and `<substituters>` match the user-environment shape. `<build-attribute>` is `None` or `(Some <flake-attribute>)`.

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
