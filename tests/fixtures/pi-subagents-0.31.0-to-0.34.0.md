# Pi extension update reconciliation fixture

## Canonical ledger

- Path: `CriomOS-home/packages/pi-subagents/fork-delta-ledger.md`.
- SHA-256: `4b04ab7982ac18b1eacdd0fff268b4ad58663b9cb266c1dd8b8164d875731083`.
- Executable evidence: `CriomOS-home/packages/pi-subagents/reconciliation/verify-0.34.0.sh`.
- Candidate patch: `CriomOS-home/packages/pi-subagents/reconciliation/0.31.0-to-0.34.0.patch`, SHA-256 `0427ab1331dc35cfb2af82ecf489e55c406a7b8f05b8a2090e4413e0fc414291`.
- Nix witnesses: sibling `0.34.0.nix` and `0.34.0-check.nix`.

This fixture snapshots the ledger semantics for isolated skills-repository tests. The generation test hashes the canonical ledger through `PI_SUBAGENTS_CANONICAL_LEDGER` when supplied and otherwise hashes the retained canonical snapshot. A canonical-ledger edit requires updating its digest and snapshot.

## Immutable candidate

- Canonical upstream: `nicobailon/pi-subagents`.
- Packaged base: npm 0.31.0, Git `e4f06282d0c95856b36b7ec2893f4fd294ebfefe`.
- Local package snapshot: CriomOS-home `8a6c5b154f7df63b65c6027ba41ea7c6496d60db`, `packages/pi-subagents/default.nix` and six sibling patches.
- Update target: npm 0.34.0, Git `12a157d2a70b2f4cbc004c020c5f9213b6d8eea8`.
- No package version or activation changed in this evidence pass.

## Delta records

Run every exact witness command below from the CriomOS-home repository root. Each exits 0 only when the recorded pristine or reconciled component results match. Status enum is `fully absorbed | partially absorbed | still absent | deliberately divergent | unknown`. Decision enum is `rebase | reimplement | drop | escalate`. State enum is `final | provisional`.

| Delta | Rationale commit | Local implementation | Status | Decision | Pristine witness command | Pristine result | Reconciled witness command | Reconciled result | State |
|---|---|---|---|---|---|---|---|---|---|
| `agent-chain-clarify-opt-in.patch` | `1dd3f033b8b322c31609e1c56c4da4b99a62bc25` | `schemas.ts @@ -246`; `chain-execution.ts @@ -504` | partially absorbed | reimplement | `packages/pi-subagents/reconciliation/verify-0.34.0.sh witness clarify pristine` | `exit=0; clarify-code=0; clarify-schema=1` | `packages/pi-subagents/reconciliation/verify-0.34.0.sh witness clarify reconciled` | `exit=0; clarify-code=0; clarify-schema=0` | provisional |
| `slim-parent-skill.patch` | `23665920be8e76c9029a546d1841654d68e39e54` | `skills/pi-subagents/SKILL.md @@ -10` | still absent | reimplement | `packages/pi-subagents/reconciliation/verify-0.34.0.sh witness compact-skill pristine` | `exit=0; compact-skill=1; lines=918` | `packages/pi-subagents/reconciliation/verify-0.34.0.sh witness compact-skill reconciled` | `exit=0; compact-skill=0; lines=136` | provisional |
| `detached-runner-peer-isolation.patch` | `6bf5e7ec700a00f33b19fe0c24d63e93f9ea61ce` | `src/shared/utils.ts @@ -5, @@ -16` | fully absorbed | drop | `packages/pi-subagents/reconciliation/verify-0.34.0.sh witness peer-isolation pristine` | `exit=0; peer-isolation=0` | `packages/pi-subagents/reconciliation/verify-0.34.0.sh witness peer-isolation reconciled` | `exit=0; peer-isolation=0` | provisional |
| `async-runner-stderr.patch` | `60528d041c0ad784ba069781c17035ba9cafc5bc` | `async-execution.ts`, old lines 215 and 238 | partially absorbed | reimplement | `packages/pi-subagents/reconciliation/verify-0.34.0.sh witness stderr-compaction pristine` | `exit=0; stderr-capture=0; stderr-post-close-compaction=1` | `packages/pi-subagents/reconciliation/verify-0.34.0.sh witness stderr-compaction reconciled` | `exit=0; stderr-capture=0; stderr-post-close-compaction=0` | provisional |
| `full-child-extension-bridge.patch` | `bff854e76bf17457a201f643622ae3dc0334e2fe` | `intercom-bridge.ts`, old 238/365; `pi-args.ts`, old 125 | partially absorbed | reimplement | `packages/pi-subagents/reconciliation/verify-0.34.0.sh witness child-extension pristine` | `exit=0; child-package-inheritance=1` | `packages/pi-subagents/reconciliation/verify-0.34.0.sh witness child-extension reconciled` | `exit=0; child-package-inheritance=0` | provisional |
| `acceptance-read-only-evidence.patch` | `df85cb32f687bb4dde1401d5cdfc6e75076c01f2` | `acceptance.ts`, old 631; new acceptance test | still absent | reimplement | `packages/pi-subagents/reconciliation/verify-0.34.0.sh witness read-only-evidence pristine` | `exit=0; read-only-evidence=1` | `packages/pi-subagents/reconciliation/verify-0.34.0.sh witness read-only-evidence reconciled` | `exit=0; read-only-evidence=0` | provisional |

## Applicability evidence

`verify-0.34.0.sh applicability` runs `patch --dry-run --forward --batch --verbose`; its retained summary is `0.34.0-applicability.txt`. It records one exit-0 patch, five exit-1 patches, reversed hunks for clarify, peer isolation, and bridge, and failed hunks for compact skill and stderr. Reversed notices are never counted as application.

## Evidence gates

| Gate | Raw result | Acceptance result |
|---|---|---|
| Six pristine/reconciled witness pairs | every command exit 0 with expected component exits | passed |
| Focused acceptance, async, Pi-argument, and chain tests | exit 0; 108 passed, 0 failed | passed |
| Pristine `npm run test:all` | exit 1; 981 total, 978 passed, 3 failed | failing gate; decisions provisional |
| Reconciled `npm run test:all` | exit 1; 985 total, 982 passed, same 3 failed | failing gate; decisions provisional despite baseline equivalence |
| Nix candidate package build | exit 0 | passed |
| Nix package-content witness | exit 0 | passed |
| Nix Pi RPC extension-load witness | exit 0; no load error; successful `get_commands` response | passed |
| Complete `verify-0.34.0.sh all` | exit 0 after verifying all expected raw statuses | evidence runner passed; decision state remains provisional |

## Decision status

This is a mixed six-delta reconciliation: one delta selects `drop`; the four originally identified remainder-analysis deltas select `reimplement`; and the cleanly applicable acceptance patch separately selects `reimplement` because unchanged replay regresses writer evidence.

No decision is final. Both pristine and reconciled full suites have raw exit 1, and baseline-equivalent failures remain failing gates. The stderr candidate is only best-effort post-close compaction; it does not claim a live 64 KiB bound or guaranteed callback execution after parent exit. The remaining work is technical, not a psyche authority, privacy, or value decision.
