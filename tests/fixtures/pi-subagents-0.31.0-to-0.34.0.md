# Pi extension update reconciliation fixture

## Canonical ledger

The owning ledger is `CriomOS-home/packages/pi-subagents/fork-delta-ledger.md`. Its executable target snapshot is `CriomOS-home/packages/pi-subagents/reconciliation/0.31.0-to-0.34.0.patch`, built by sibling `0.34.0.nix`. This fixture snapshots the acceptance semantics that the skills generator test guards.

## Immutable candidate

- Canonical upstream: `nicobailon/pi-subagents`.
- Packaged base: npm 0.31.0, Git `e4f06282d0c95856b36b7ec2893f4fd294ebfefe`.
- Local package snapshot: CriomOS-home `8a6c5b154f7df63b65c6027ba41ea7c6496d60db`, `packages/pi-subagents/default.nix` and six sibling patches.
- Update target: npm 0.34.0, Git `12a157d2a70b2f4cbc004c020c5f9213b6d8eea8`.
- Retained reconciliation patch: SHA-256 `ce23d291df868b87e84e342e3a9a3909677bc97ec60e2ef5a3d00ae7a5979ec4`.
- No package version or activation changed in this evidence pass.

## Applicability evidence

All probes used `patch --dry-run --forward --batch --verbose` on a pristine target. Reversed notices were not counted as application.

| Delta | Exact target applicability | Target status | Candidate action | Decision state |
|---|---|---|---|---|
| `agent-chain-clarify-opt-in.patch` | schema hunk succeeded at 275 with offset 29; chain hunk reversed and ignored at 504; exit 1 | partially absorbed | drop absorbed code; reimplement schema wording | provisional |
| `slim-parent-skill.patch` | whole-skill hunk failed at 10; exit 1 | still absent | reimplement compact parent skill | provisional |
| `detached-runner-peer-isolation.patch` | both hunks reversed and ignored at 5 and 16; exit 1 | fully absorbed | drop | supported, not package-landed |
| `async-runner-stderr.patch` | helper hunk succeeded at 277 with fuzz 1 and offset 62; spawn hunk failed at 266; exit 1 | partially absorbed | reimplement only the 64 KiB bound | provisional |
| `full-child-extension-bridge.patch` | two intercom hunks reversed and ignored; `pi-args` hunk succeeded at 141 with offset 16; exit 1 | partially absorbed | drop intercom hunks; reimplement extension inheritance | provisional |
| `acceptance-read-only-evidence.patch` | source and new-test hunks applied; exit 0, but unchanged replay regressed writer evidence | still absent | target-native read-only gate with negative writer test | provisional |

## Rationale provenance

The ledger ties each delta to immutable CriomOS-home commits and exact paths/hunks: clarify `1dd3f033b8b322c31609e1c56c4da4b99a62bc25`; compact skill `23665920be8e76c9029a546d1841654d68e39e54` plus `60ed02dfbbd34bddef417abc2c75e5270b652959`; peer isolation `6bf5e7ec700a00f33b19fe0c24d63e93f9ea61ce`; stderr `60528d041c0ad784ba069781c17035ba9cafc5bc` plus `f3fcf3e89b9448a5b99236415fe04fc207ddecd6`; bridge `bff854e76bf17457a201f643622ae3dc0334e2fe`; acceptance `df85cb32f687bb4dde1401d5cdfc6e75076c01f2`.

## Evidence gates

| Gate | Result |
|---|---|
| Per-delta pristine/reconciled witnesses | expected pristine failures; every retained reconciled witness passed |
| Focused acceptance, async, Pi-argument, and chain tests | 108 passed, 0 failed |
| Pristine `npm run test:all` | 981 total, 978 passed, 3 upstream test-double failures |
| Reconciled `npm run test:all` | 985 total, 982 passed, the same 3 upstream failures |
| Nix candidate package build | passed with npm dependency identity `sha256-IJJ3hceNvHUr5QFIa/+0tnxNiEPh7jifE9dvPHrLE58=` |
| Package-content verification | passed for metadata, dependencies, compact skill, clarify, stderr, bridge, and acceptance witnesses |
| Pi RPC extension load | exited 0; no load error; response had `command:"get_commands"` and `success:true` |

## Decision status

This is a mixed six-delta reconciliation: one upstream-owned delta supports drop; the four originally identified remainder-analysis deltas require target-native remainder work; and the cleanly applicable acceptance patch separately requires target-native reimplementation because unchanged replay regresses writer evidence.

The candidate is not called complete because both pristine and reconciled full suites retain three upstream failures. No gate is represented as passed when it was not executed or remained red. The remaining issue is technical test-fixture work, not a psyche authority, privacy, or value decision.
