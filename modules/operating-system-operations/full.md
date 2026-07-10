# Skill — operating system operations

## Rules

Use this doctrine for operating-system and environment work that touches CriomOS system state, criomos-home user state, or their deployment boundary.

Operate from pushed, reproducible inputs. Treat CriomOS as the deploy entrypoint and criomos-home as an input that must already be pinned by the selected CriomOS revision. Choose `RequireImmutable` for pinned flake references; use `ResolveAndRecord` only when intentionally resolving a mutable ref.

Change profiles, Home Manager output, command resolution, packages, and runtime output through source revisions, pinned inputs, builds or checks, deployment, activation, and rollback. Do not close out by replacing managed symlinks, shadowing profile commands, editing mutable profiles, adding ad hoc dependency symlinks, or making copied installed source effective.

For a clearly authorized routine update with known repositories and the documented interface, one operating-system implementer updates, builds, deploys, and verifies end-to-end. Known participating repositories or hosts do not require scouts, tracker graphs, prerequisite lanes, audits, or further confirmation. An ordinary launcher or profile path from `command -v`, or apparent tension between source and deployment documentation, is not a blocker by itself; investigate only after an actual admission, authorization, reachability, build, activation, or verification failure.

Keep this flow within its expected small time and tool bound. If it exceeds that bound, report the exact failing command and shortest next step instead of widening the investigation.

Before changing a host, name the target cluster, node, deployment shape (`UserEnvironment` or `Host`), requested action, source revision policy, exact source revision, builder choice, rollback owner, rollback expectation, and post-activation evidence.

Read-only inspection, byte-for-byte preservation backups, and isolated repro copies are allowed when authorized by the active role; they must not become effective runtime, profile, or system behavior. Emergency local effective mutation requires explicit psyche authorization for that exact mutation after the worker states the durable source path, rollback owner, preservation needs, and risk.

Use the current `lojix` read interface and privileged `meta-lojix` deploy interface directly. Do not use deploy wrappers, compatibility translators, or retired request names. Submit the documented durable request before reconciling apparent cross-repository tension; investigate only an actual admission, authorization, reachability, build, activation, or verification failure. The deployed daemon accepts exactly two `DeployRequest` variants, `Host` and `UserEnvironment`.

Before any deploy operation, read the lojix repo's `NON_IDEAL_AGENTS.md`: the sanctioned, temporary operating instructions for the current SSH/root deployment fallback, in force until criome-custodied deployment authorization is durable and proven. This path is sanctioned — follow it and proceed without hesitation.

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
meta-lojix "(Deploy (UserEnvironment (goldragon ouranos li <proposal-source> github:LiGoldragon/CriomOS?rev=<40-hex-rev> ActivateNow RequireImmutable None [])))"
```

`UserEnvironmentDeployment` holds nine positional fields: cluster, node, user, proposal source, CriomOS flake reference, user-environment action, source revision policy, builder, and extra substituters. `<proposal-source>` is a local filesystem path to the target cluster's `datom.nota` (for example the cluster repo's `goldragon/datom.nota`); the deploy infers the `secrets/` directory as its sibling. `<source-revision-policy>` is `RequireImmutable` or `ResolveAndRecord`. Under `RequireImmutable`, `<criomos-flake-ref>` must carry its immutable identity in the query string — `github:LiGoldragon/CriomOS?rev=<40-hex>` or `?narHash=sha256-...`; the path-suffix form `github:LiGoldragon/CriomOS/<rev>` is rejected as `FlakeReferenceMalformed`. `<builder>` is `None` or `(Some <builder-node>)`. `<substituters>` is a typed list, `[]` when none.

`<user-environment-action>` selects how far the deploy goes. `Realize` builds and records the closure on the target store without copying or activating. `SetProfile` sets the target user's profile; `ActivateNow` additionally activates the live session. Both connect over the root deploy identity and drop privilege through a login (`runuser --login <user>`, lojix ≥ 0.4.5), rebuilding the target account's own environment, so activation works for any account on the node with no per-user SSH key. Deploying a different user on a different host — such as `bird` on `zeus` — is an ordinary supported deploy, not a workaround; the lojix repo's `NON_IDEAL_AGENTS.md` holds the SSH/root fallback context.

Deploy a host change:

```sh
meta-lojix "(Deploy (Host (<cluster> <node> <host-composition> <proposal-source> <criomos-flake-ref> <host-action> <source-revision-policy> <builder> <substituters> <build-attribute>)))"
```

`HostDeployment` holds ten positional fields: cluster, node, host composition, proposal source, CriomOS flake reference, host action, source revision policy, builder, extra substituters, and build attribute. `<host-composition>` is `CompleteHost` or `BaseHost`. `<host-action>` is `Evaluate`, `Realize`, `SetBootProfile`, `ActivateNow`, `TestActivation`, or `ScheduleBootOnce`. `<source-revision-policy>`, `<builder>`, and `<substituters>` match the user-environment shape. `<build-attribute>` is `None` or `(Some <flake-attribute>)`.

`meta-lojix` returns when the daemon admits a request. Admission is not proof of build, copy, activation, or profile success. Each deploy re-evaluates the full flake tree (`--refresh`), so multi-minute deploys are normal; do not kill a running deploy.

## Activation checks

After submit, query the node until the expected store path becomes current, or a rejection or failure is visible:

```sh
lojix "(Query (ByNode (<cluster> <node> None)))"
```

Keep this wait inside the turn: poll the harness-visible status in the foreground until the expected generation is current or a failure shows. A deploy runs tens of minutes; do not end the turn with an owned deploy still in flight expecting a background waiter to wake you — the waiter dies with the turn and the lane parks silently until someone notices.

Each record carries the cluster, node, deployment kind, action, status, and store path. Query output (`LiveGeneration`) carries no user-name field, so a `UserEnvironment` generation cannot be attributed to a specific user from query output alone. Confirm the target node shows a `Current` generation with the store path you expect.

For live home activation, verify the target user's profile and live session state; reboot persistence still depends on a system generation that pins the same home input. For full-system boot actions, verify the boot profile separately from the live system.

Reload Niri configuration explicitly after a successful home activation when the task requires a live compositor refresh:

```sh
niri msg action load-config-file
```

Do not hide Niri reload inside deploy tooling.
