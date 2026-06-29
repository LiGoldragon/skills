# Skill — system operator

Maintaining the operating-system layer underneath the workspace.

## What this role owns

The system operator makes the system run: CriomOS, CriomOS-home, lojix
deployment, horizon projection, desktop runtime, user services, input
devices, Niri, Noctalia, and the system/home interfaces. Claim the role
through `orchestrate "(Claim (system-operator [(Path /absolute/path)] [reason]))"`
before editing the OS / platform surface; reports in
`reports/system-operator/` are exempt from the claim flow.

The owned pieces and how they fit:

- **CriomOS** — NixOS host platform: system modules, device access,
  groups, udev, kernel modules, the `nixosConfigurations.target` surface.
- **CriomOS-home** — Home Manager profile: Niri bindings, Noctalia, user
  packages, user services, desktop tools such as Whisrs.
- **lojix daemon stack** — deploy/build/activate entry points that project cluster
  proposals into the inputs CriomOS and CriomOS-home consume.
- **horizon-rs** — typed projection/schema source for horizon fields.
- **goldragon** — the cluster-proposal data lojix uses for the machines.

Preserve the system/home boundary when a task crosses it. Whisrs
packaging, keybindings, tray state, clipboard recovery, and transcript
history live in CriomOS-home; `/dev/uinput` group/module/udev access
lives in CriomOS.

## Required reading

Read these before substantive work, beyond the workspace baseline every
role reads (`ESSENCE.md`, `AGENTS.md`, the orchestrate contract, the
common skills). The emphasis is Nix and deployment discipline plus the
Rust crates that ship as host tools (lojix, horizon-rs, clavifaber,
chroma).

- `skills/operator.md` — the sister role; knows what binaries deploy.
- `skills/nix-usage.md`, `skills/nix-discipline.md`
- `skills/rust-discipline.md` (index) and its sub-files
- `skills/abstractions.md`, `skills/actor-systems.md`
- `skills/stt-interpreter.md` — STT prompts and likely transcription
  mistakes.
- CriomOS's `skills.md` — cluster domain generation, network-neutral
  NixOS module discipline, the real deploy path.

When work enters a repo, read its `AGENTS.md`, `ARCHITECTURE.md`,
`skills.md`, plus `docs/ROADMAP.md` and open BEADS for CriomOS /
CriomOS-home.

## Working pattern

Prefer the existing deployment path over one-off commands:

- Home activation goes through lojix `HomeOnly ... Activate`.
- System builds/switches go through lojix-projected CriomOS inputs.
- Push the build to origin with `--refresh` before trusting the result.
- Keep store paths in shell variables, not prose.
- Do not signal niri.

Niri config changes are not live when they land in a repo or after a
successful build. For changes to `programs.niri.settings` that must
affect the running session: push the CriomOS-home commit, activate the
home profile through lojix `HomeOnly ... Activate`, then reload with
`niri msg action load-config-file` (Niri's IPC reload action — allowed).
SIGHUP and other process signals remain forbidden. Do not claim a new
window rule, keybind, or runtime setting is tested until activation and
IPC reload have both happened.

Secrets stay out of Nix and broad process environments. For paid cloud
inference: local model first, then ask before using a paid key unless
the user explicitly authorized that call in the current task. Keys come
from `gopass` at the daemon-wrapper layer; private key bytes never reach
stdout, logs, reports, the Nix store, or fixtures.

## Just-do-it operations

These follow inevitably from earlier work in the same session; stopping
to ask produces friction without a decision. Do them without confirming.

- **Downstream `flake.lock` bumps after upstream commits.** When you
  push a change to `lojix`, `horizon-rs`, `nota-codec`,
  `nota-derive`, or any repo consumed via flake-input by `CriomOS-home`,
  update `CriomOS-home/flake.lock` to the new commit and redeploy. The
  chain is `nix flake update <input> → commit → push → HomeOnly
  Activate`. Rule of thumb: if you said "use the new version" earlier in
  this session, the lock bump is already authorized.
- **Re-deploying after activation-affecting home changes.** When a
  CriomOS-home commit changes activation behavior (new module, service,
  or `home.activation` hook), run `HomeOnly Activate` against the local
  node. Don't leave a green commit and a stale generation.
- **Re-deploying after CriomOS-home flake-input bumps.** Same shape: if
  the input bump is the whole point of the change, the deploy is part of
  the change.

If something goes wrong mid-procedure (build failure, signature
rejection), surface it — the obstacle is the question, not whether to
proceed. This covers the standard happy path following from the work
just done, not pushing through real errors silently.

## Runtime interfaces

The system operator gives the user working interfaces, not just
packages: keybindings that work in Niri; visible status for long-running
actions; user services that restart through Home Manager activation;
recovery paths for fragile desktop input; logs that expose operational
state without leaking private content.

## Operator interface — Nota only

Cluster deploy requests flow through the daemon-based `lojix` clients, and
the operator surface is exactly one Nota record. The clients take no flags
and no subcommands. New deploy behavior lands as typed contract fields,
never as a flag, env-var dispatch, or custom argv parser. The Nota record
is the operator's surface and the audit trail.

The same shape applies cluster-wide: cluster proposals
(`goldragon/datom.nota`), horizon projections, and any future
operator-facing data live as typed Nota records read by `nota-codec`.
New fields are positional in source-declaration order; reordering or
renaming is a breaking change. Use current Lojix source for per-repo
specifics.

## Cluster Nix signing

CriomOS wires daemon-attached Nix signing only on **cache nodes**
(`isNixCache = true`): `services.nix-serve.secretKeyFile` in
`modules/nixos/nix.nix`. Non-cache nodes have no
`nix.settings.secret-key-files` and no signing private key on disk. The
paths they build are `ultimate`-trusted locally but carry no
transferable signature.

Trust direction is wired correctly: every node's `trusted-public-keys`
is rolled up from datom by horizon-rs (`lib/src/horizon.rs`, filter on
`nix_pub_key_line`).

How signed paths flow: only `nix-serve` signs, and only over HTTP at
request time. Direct nix-daemon-to-nix-daemon transfer over `ssh-ng`
carries whatever signatures the source path already has — locally built
paths on non-cache nodes have none.

To bridge that gap, the deploy copy path passes `--substitute-on-destination`
to `nix copy`. The target prefers
substituting each path from its own substituters (the cluster HTTP
cache) over receiving the raw path from the source. When the cache has
the closure, the target gets it signed and verified; when the cache
misses, the copy falls back to the unsigned ssh-ng path and fails.

**Practical consequence:** deploys must route the build through a cache
node so the cache has the closure to serve. Use `builder = <cache-node>`
in the Nota request — e.g. `(FullOs goldragon zeus … Switch
prometheus)`. The cache builds, nix-serve signs on serve, the target
substitutes signed. `builder = None` is broken for cross-host deploys:
the dispatcher builds locally, nothing in the cluster has the closure,
substitution misses, ssh-ng fallback delivers unsigned paths, the target
rejects.

**Diagnostics:**

- Local sig: `nix path-info --sigs <path>` — `ultimate` without a `Sig:`
  means unsigned local build.
- Cache sig: `curl
  http://nix.<cache>.<cluster>.criome/<storehash>.narinfo` and read the
  `Sig:` line.
- Reproduce push failure: `nix copy --to ssh-ng://root@<target> <path>`.
- Confirm fix: `nix copy --substitute-on-destination --to
  ssh-ng://root@<target> <path>` — if the cache has the path you'll see
  `copying path '…' from 'http://nix.<cache>.<cluster>.criome'` and the
  target accepting.

**Generating per-node signing keys** (partially landed): on each host,
generate at `/etc/nix/secret-key`:

```sh
ssh root@<host> '
  nix-store --generate-binary-cache-key <host>.<cluster>.criome \
    /etc/nix/secret-key /etc/nix/secret-key.pub &&
  chmod 400 /etc/nix/secret-key &&
  chmod 444 /etc/nix/secret-key.pub
'
```

Then read `/etc/nix/secret-key.pub` and replace the matching node's
`NodePubKeys.nix` field in `goldragon/datom.nota`. Push goldragon.
Redeploy each updated host so its trust list reflects the new pubkeys
(use `builder = prometheus` because non-cache nodes still don't sign).

The keys are inert until CriomOS wires `nix.settings.secret-key-files`
in `modules/nixos/nix.nix` — the still-pending fix that would let
non-cache nodes' daemons sign locally-built paths and let `builder =
None` deploys produce verifiable closures.

## Lanes

A session lane carrying the system-operator discipline loads this
skill's required reading, owned area, and beads label; the lane's
session-intent name gives it its directory and claim string (mechanism
in `skills/session-lanes.md`). Several system-operator-discipline lanes
can run at once. A lane scoped to system topics may instead carry the
designer discipline when the work is system architecture — see
`skills/designer.md`.

Good system-lane work has a concrete boundary: one CriomOS or
CriomOS-home module slice, a focused audit of recent commits, a
self-contained host-tool slice (Whisrs packaging, Clavifaber typed-record
additions, chroma instrumentation), a Nix flake hygiene pass, or a
deploy-affecting doc update from already-settled platform work. Lane
scope mirrors system-operator's surface but stays within already-decided
shapes.

Defer to system-operator on cluster-effecting changes — cluster Nix
signing topology, signing-key generation, deploy-graph topology, host
activation orchestration. The just-do-it operations above apply to lanes
too, since those are inside the standing happy path; a change that
*modifies* the path itself is system-operator authority.

If a host change reveals a structural gap (a missing actor plane, a
subscription primitive that doesn't exist, a NOTA-vs-Signal boundary
that was wrong), write an implementation-consequences report and let
system-operator or designer answer rather than deciding inside the
implementation pass.

## See also

- CriomOS's `skills.md`, CriomOS-home's `skills.md`
- `skills/stt-interpreter.md`, `skills/session-lanes.md`
- `skills/operator.md`, `skills/designer.md` — sister role skills
