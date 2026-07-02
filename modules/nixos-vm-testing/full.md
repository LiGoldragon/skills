# Module - NixOS VM testing

## NixOS VM Testing Purpose

VM tests in this workspace boot real CriomOS guests under `pkgs.testers.runNixOSTest` (NixOS test driver + QEMU). The canonical test repo is `CriomOS-test-cluster` (`github:LiGoldragon/CriomOS-test-cluster`), worktree at `wt/github.com/LiGoldragon/CriomOS-test-cluster/criome-cluster-test`.

## How Tests Are Structured

Tests live in `criome-cluster-test` as flake checks under `checks.x86_64-linux.<name>`. Three generators compose them from cluster projection data — never hand-stubbed node facts:

- `lib/mkVmTest.nix` — single-guest: boots one CriomOS Pod node (OS, size, accel, network address all derived from its `fixtures/horizon/<node>.json` projection) and runs a `testScript`.
- `lib/mkCriomeClusterTest.nix` — single-guest criome daemon test: boots one node, starts `criome-daemon`, runs a witness binary over the socket. Stage A (1-of-1 quorum) is built; Stage B (multi-node quorum, cross-guest fan-out) is not yet implemented.
- `lib/mkDeployTest.nix` — two-guest deploy smoke: boots a deployer node and a target node on the same test network; the deployer runs the real lojix daemon and deploys a CriomOS toplevel into the target.

Auto-pickup generates one `vm-<node>` check per Pod node declared on the fieldlab cluster's VmHost (currently: `atlas`). Adding a node to the cluster data produces a check without flake edits.

Two check altitudes exist:
- Pure/eval checks (e.g. `projections-match-fieldlab`, `cluster-contracts`, `full-module-contracts`): build a NixOS config and assert static attributes; no VM boot.
- Booted VM checks (e.g. `vm-mercury`, `vm-edge-desktop`, `lojix-deploy-smoke`): start one or more QEMU VMs and assert runtime behavior.

## Running VM Tests Locally

Run a single named check from a local worktree checkout:

```sh
nix build "<repo-path>#checks.x86_64-linux.<check-name>" \
  --no-link --print-build-logs
```

Example — the lean TestVm boot check (17 s locally, KVM via remote builder):

```sh
nix build "github:LiGoldragon/CriomOS-test-cluster#checks.x86_64-linux.vm-mercury" \
  --no-link --print-build-logs
```

Run all checks via `nix flake check` (builds the full suite including desktop and deploy tests; heavy). The remote builder `prometheus.goldragon.criome` has KVM and serves as NixBuilder; the daemon routes heavier checks there automatically.

## Running On Prometheus (Remote Path)

The `run-on-prometheus` script in `criome-cluster-test/scripts/` pushes the current main bookmark and runs `nix flake check "$repo" --refresh` inside a sandboxed `systemd-run --user` unit on prometheus:

```sh
nix run /path/to/criome-cluster-test#run-on-prometheus
```

The `nspawn-dune-on-prometheus` and `nspawn-spirit-upgrade-on-prometheus` scripts build a specific toplevel on prometheus and boot it as a `criomos-nspawn` container (systemd-nspawn) for stateful smoke tests. These are operator-facing e2e paths, not hermetic flake checks.

## KVM Acceleration

KVM availability is cluster-data-decided (`VmHost.kvm Available` on atlas and prometheus). Tests run with KVM when the builder host declares it available; TCG software emulation is the fallback when `kvm Absent`. The `vm-mercury` and `vm-edge-desktop` checks run with KVM when built on prometheus.

## Multi-Node Capability

`runNixOSTest` supports multiple named nodes on a shared test network. The `lojix-deploy-smoke` check demonstrates this: it declares `nodes.deployer` and `nodes.mercury`, connects them via the test network, and proves deployer-to-target store copy and remote activation. Multi-node VM tests for arbitrary service pairs follow the same `nodes.<name>` pattern.

For Spirit state mirroring across two nodes, use `mkCriomeClusterTest` as the template and declare both nodes in `nodes`. Stage B of the criome cluster test (cross-guest quorum fan-out) is not yet implemented; a Spirit-specific two-node mirror test would be new work following the `lojix-deploy-smoke` model.

## State Persistence

`runNixOSTest` VM disks are ephemeral per run; no state carries between check invocations. The `criomos-nspawn` path on prometheus creates and destroys containers per script run. Cross-run state persistence is not provided by the existing infrastructure; it requires either persistent volumes on a dedicated host or a stateful service on a real CriomOS node.

## Prerequisites

- A NixOS/CriomOS builder with KVM available for fast VM tests (prometheus serves this role).
- The remote builder at `prometheus.goldragon.criome` must be reachable; the Nix daemon on the local host must have it in `builders`.
- The Rust toolchain pinned in `channel-rust-stable.toml` must match the current fenix channel; a hash mismatch in that fixed-output derivation blocks checks that build Rust components.
