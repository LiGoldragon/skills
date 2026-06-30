# Skill — operating system operations

## Rules

Use this doctrine for operating-system and environment work that touches CriomOS system state, criomos-home user state, or their deployment boundary.

Operate from pushed, reproducible inputs. Treat CriomOS as the system source identity and criomos-home as the home/environment source identity. Use pinned flake revisions for effect-bearing deploys when branch resolution or cache freshness is uncertain.

Name the target cluster, node, system or home kind, action or mode, builder choice, rollback expectation, and post-activation evidence before changing a host.

Do not use deprecated `lojix-cli`; use the current `lojix` read interface and privileged `meta-lojix` deploy interface.

## Lojix interface

Read current generations:

```sh
lojix "(Query (ByNode (<cluster> <node> None)))"
```

Submit a system deploy from a CriomOS flake revision:

```sh
meta-lojix "(Deploy (System (<cluster> <node> FullOs <proposal-source> <criomos-flake-ref> <action> <builder> [] None)))"
```

Submit a home/environment deploy from a criomos-home input through the selected CriomOS flake revision:

```sh
meta-lojix "(Deploy (Home (<cluster> <node> <user> <proposal-source> <criomos-flake-ref> <mode> <builder> [])))"
```

Use `FullOs` for normal live CriomOS desktop deploys. `OsOnly` omits home and broad firmware materialization; use it only when that omission is intended and safe. System actions are `Eval`, `Build`, `Boot`, `Switch`, `Test`, and `BootOnce`. Home modes are `Build`, `Profile`, and `Activate`. A builder is `None` or `(Some <builder-node>)`.

`meta-lojix` returns when the daemon accepts a request. A `(Deployed ...)` reply does not prove build, copy, activation, or profile success.

## Activation checks

After submit, query the node until the expected kind and closure become current or a rejection/failure is visible:

```sh
lojix "(Query (ByNode (<cluster> <node> None)))"
```

For `Boot`, verify the boot profile separately from the live system. For `Switch`, verify activation and task-specific systemd or user units. For home activation, verify the target user's profile and live session state; reboot persistence still depends on a system generation that pins the same home input.
