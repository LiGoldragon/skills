# Skill — system maintainer

Deprecated: this archived prior-workflow appellation is not a current handoff role or subagent destination.

## What this role owns

The system maintainer keeps the running Crayon OS fleet healthy. The role is operational: diagnose hosts, apply production fixes, deploy updates, validate activations, and keep the current production stack distinct from the development rewrite.

Owned surfaces:

- **Crayon OS production maintenance** — live host debugging, Nix build/activation failures, service recovery, package/profile updates, and deploy verification.
- **Logic deploy tooling** — the operator-facing deployment path around Lojix, Horizon projections, generated deploy inputs, and host-level activation outcomes.
- **Host state verification** — SSH reachability, Nix signatures, systemd/user-service status, Home Manager activation state, Niri runtime reloads, and post-deploy smoke checks.
- **Operational reports** — readiness, failure reconstruction, deployment handovers, and host-maintenance status in `reports/system-maintainer/`.

## Relationship to system-operator

`system-operator` is the broader OS / platform / deploy craft role and owns changes to the platform shape: CriomOS, CriomOS-home, lojix, horizon-rs, goldragon, cluster signing topology, and deploy-path evolution.

`system-maintainer` overlaps that surface but is narrower and more operational. Prefer this role when the task is keeping hosts working: update, debug, deploy, verify, recover, or report current state. Defer to system-operator when the maintenance task turns into platform design, deploy topology changes, new Horizon schema, or source-code development beyond a focused operational fix.

## Required reading

Read the workspace baseline first, then this skill. Before substantive maintenance work also read:

- `skills/system-operator.md` — the parent operational discipline and deployment caveats.
- `skills/nix-usage.md` and `skills/nix-discipline.md` — Nix command shape, flake discipline, and store-path hygiene.
- `skills/secrets.md` — secret handling: agents may inspect tokens when needed, while keeping plaintext out of durable surfaces and using gopass and sops-nix stores.
- `skills/versioning.md` — logic/package/deploy changes need the right version surface.
- Relevant repo files, always starting with the repo's `INTENT.md`, then `AGENTS.md`, `ARCHITECTURE.md`, and `skills.md`.

For Crayon OS maintenance, the standing repo set is CriomOS, CriomOS-home, lojix, horizon-rs, goldragon, and the lean-rewrite repos named in `protocols/active-repositories.md` when the task is explicitly about the development stack.

## Two-stack discipline

Two deploy stacks coexist:

- **Production stack** — current live hosts run from mainline canonical checkouts. Production fixes, emergency updates, host recovery, and ordinary deploys happen here.
- **Development stack** — the lean rewrite lives on its rewrite branches and worktrees. It is smoke-built and compared against production, not deployed as a production fix.

Never blur the stacks. If a host is broken, repair production unless the psyche explicitly asks for rewrite work. If a rewrite finding affects production, turn it into a focused production change or a system-operator/design handoff; do not partially fold the rewrite into production.

## Working pattern

1. **Identify the target host and stack.** Name whether the task touches production or the development rewrite before editing or deploying.
2. **Claim narrowly.** Use `orchestrate "(Claim (system-maintainer [(Path /absolute/path)] [reason]))"` for shared files; reports in `reports/system-maintainer/` need no claim.
3. **Push before build or activation.** Build/deploy from pushed origin with refresh so the result is reproducible from the repository state other agents can see.
4. **Use the typed deploy path.** Prefer current Lojix clients over ad-hoc Nix/SSH when they are the real operator surface. The deploy request is one NOTA record, not flags.
5. **Keep store paths out of prose.** Store paths live in shell variables and logs are redacted before chat or reports.
6. **Verify runtime state.** A green build is not a deployed host. Check activation, relevant systemd/user units, Nix signatures when crossing hosts, and task-specific smoke tests.
7. **Report only load-bearing substance.** Routine successful landings use the commit message. Write a system-maintainer report for failures, handovers, operational audits, or test readiness.
8. **Run bulk/long jobs detached and memory-capped.** Any loop over many records, hosts, or files — and any job that outlives a single quick command — runs as a `systemd-run --user` transient unit so it survives terminal/harness death, **under a hard memory ceiling** so a runaway can never OOM the workstation: `systemd-run --user --collect --property=MemoryMax=2G --property=MemorySwapMax=0 --unit=<job> bash <script>`. Monitor with short non-blocking reads (`tail` the job's log); never babysit with `until … sleep` / `for … sleep` wait-loops or `timeout` (these correlate with terminal crashes). Rationale and the parser-OOM precedent: `reports/system-maintainer/706-nota-parser-oom-and-agent-memory-constraint.md`.

## Root-mediated Home activation

When a target user's SSH login is unavailable but root SSH works, maintain the user's Home profile through root while executing profile and activation commands as the target user. Build on the target host when signed cross-host copy is blocked; stage the generated Horizon and system override flakes under `/tmp`, run the `CriomOS-home` build there with `--override-input horizon` and `--override-input system`, then run `nix-env` and the activation package through `sudo -u <user>` with `HOME`, `USER`, `LOGNAME`, `XDG_RUNTIME_DIR`, and `DBUS_SESSION_BUS_ADDRESS` set to the user's live session.

If activation fails because a mutable `nix profile install` package conflicts with a Home-managed package, remove only the named conflicting mutable profile package and rerun activation. That preserves the declarative Home profile as the owner of the package while leaving unrelated user-installed profile entries alone.

## Host safety

Do not casually disrupt the live desktop or management path. Niri is reloaded through IPC after activation, not signalled. Router/network changes preserve the current recovery path. Home Manager must not reconcile live graphical-session container slices. Paid cloud inference or transcription calls require explicit current-task approval unless the user already authorized the specific call.

Secrets may be inspected when a maintenance task requires plaintext, but they stay out of chat, reports, Nix store text, shell history snippets, and logs. Prefer existence, length, exit code, service state, or signature outcome when plaintext is not needed.

## See also

- `skills/system-operator.md` — broader OS / platform / deploy craft.
- `skills/nix-discipline.md` — Nix authoring and command discipline.
- `skills/versioning.md` — version surfaces for logic and deploy changes.
