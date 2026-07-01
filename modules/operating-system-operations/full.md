# Skill — operating system operations

## Rules

Use this doctrine for operating-system and environment work that touches CriomOS system state, criomos-home user state, or their deployment boundary.

Operate from pushed, reproducible inputs. Treat CriomOS as the system source identity and criomos-home as the home/environment source identity. Use pinned flake revisions for effect-bearing deploys when branch resolution or cache freshness is uncertain.

Before changing a host, name the target cluster, node, artifact kind (`CompleteHost`, `BaseHost`, or `UserEnvironment`), requested deploy action, source revision policy, builder choice, rollback expectation, and post-activation evidence.

Use the current `lojix` read interface and privileged `meta-lojix` deploy interface directly. Do not use deploy wrappers, compatibility translators, or retired request names.

## Lojix interface

Read current generations:

```sh
lojix "(Query (ByNode (<cluster> <node> None)))"
```

Submit a complete host deploy from a CriomOS flake revision:

```sh
meta-lojix "(Deploy (Host (<cluster> <node> CompleteHost <proposal-source> <criomos-flake-ref> <host-action> RequireImmutable <builder> [] None)))"
```

Submit a base host deploy from a CriomOS flake revision when the host closure intentionally omits embedded user environment materialization and broad all-firmware materialization:

```sh
meta-lojix "(Deploy (Host (<cluster> <node> BaseHost <proposal-source> <criomos-flake-ref> <host-action> RequireImmutable <builder> [] None)))"
```

Submit a user-environment deploy from a criomos-home input through the selected CriomOS flake revision:

```sh
meta-lojix "(Deploy (UserEnvironment (<cluster> <node> <user> <proposal-source> <criomos-flake-ref> <user-environment-action> RequireImmutable <builder> [])))"
```

Use `RequireImmutable` for pinned flake revisions. Use `ResolveAndRecord` when the daemon should resolve a mutable source and record the resolved revision before building. A builder is `None` or `(Some <builder-node>)`. Extra substituters are explicit typed records in the request; there is no host-label shorthand.

`meta-lojix` returns when the daemon accepts a request. A `DeployAccepted DeployHandle` reply is admission evidence only; it does not prove build, copy, activation, or profile success.

## Activation checks

After submit, query the node until the expected artifact and closure become current or a rejection/failure is visible:

```sh
lojix "(Query (ByNode (<cluster> <node> None)))"
```

For boot-profile host actions, verify the boot profile separately from the live system. For live host activation, verify activation and task-specific systemd or user units. For user-environment activation, verify the target user's profile and live session state; reboot persistence still depends on a system generation that pins the same home input.

Niri configuration reload is an explicit operator procedure after a successful user-environment activation when the task requires live compositor config refresh:

```sh
niri msg action load-config-file
```

Do not hide Niri reload inside deploy tooling.
