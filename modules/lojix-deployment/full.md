# Skill — lojix deployment

## Rules

Use the daemon-based Lojix stack for CriomOS deploy work. Do not use `lojix-cli`; it is archived and not the current operator surface.

Deploy only pushed, reproducible inputs. Use a pinned CriomOS flake revision for effect-bearing deploys when branch resolution or cache freshness is uncertain. Name the target cluster, node, deployment kind, action or mode, builder choice, and rollback expectation before activating a host.

`meta-lojix` accepts privileged deploy requests on the owner socket. A `(Deployed ...)` reply means the daemon accepted the job; it does not prove build, copy, or activation success. Poll the ordinary read surface and daemon evidence before claiming a deploy is current.

## Commands

Read current generations:

```sh
lojix "(Query (ByNode (<cluster> <node> None)))"
```

Submit a system deploy:

```sh
meta-lojix "(Deploy (System (<cluster> <node> FullOs <proposal-source> <criomos-flake-ref> <action> <builder> [] None)))"
```

Submit a home deploy:

```sh
meta-lojix "(Deploy (Home (<cluster> <node> <user> <proposal-source> <criomos-flake-ref> <mode> <builder> [])))"
```

Use `FullOs` for normal live CriomOS desktop deploys. `OsOnly` omits home and broad firmware materialization; use it only when that omission is intended and safe. System actions are `Eval`, `Build`, `Boot`, `Switch`, `Test`, and `BootOnce`. Home modes are `Build`, `Profile`, and `Activate`. A builder is `None` or `(Some <builder-node>)`.

## Post-deploy checks

After submit, query the node until the expected kind and closure become current or a rejection/failure is visible:

```sh
lojix "(Query (ByNode (<cluster> <node> None)))"
```

For `Boot`, verify the boot profile separately from the live system. For `Switch`, verify activation and task-specific systemd or user units. For home activation, verify the target user's profile and live session state; reboot persistence still depends on a system generation that pins the same home input.
