# Pi extension update reconciliation fixture

## Candidate

- Canonical upstream: npm `pi-subagents`, repository `nicobailon/pi-subagents`.
- Packaged source: npm `0.31.0`, locked by the CriomOS-home flake input `pi-subagents-src` with NAR identity `sha256-EmDqAPVqJ6hxuA3Yj8SikM2kA/oI6D1QEe/gPvJbIVw=`.
- Target: npm `0.34.0`, registry integrity `sha512-JGgSYaieZ/2QtsW6BwSV1SX6zMz+YpV0JXUjSTtgphpk+z5OOJVJ4D/tWnCxIURXKcgsam+1vQkQgQ5fhrasFA==`.
- Consumer: `packages/pi-subagents/default.nix`, exposed by the Pi Home Manager profile.
- Local maintenance: six package patches discovered by tracing the derivation. A canonical-name npm input therefore still has fork deltas.

Both tarballs were unpacked and compared. All six patches applied to `0.31.0`. Each patch was then checked forward-only against an untouched `0.34.0` tree and its target source was inspected; reversed-patch notices were not counted as successful applications.

## Delta Ledger Dry Run

| Delta | Purpose and witness | Target status | Technical path |
|---|---|---|---|
| `agent-chain-clarify-opt-in.patch` | Chains clarify only when explicitly requested; chain execution branch and packaged skill text witness it. | Partially absorbed: target uses `clarify === true`, while the local schema wording is not upstream. | Drop the absorbed code hunk; decide technically whether the schema wording still needs a target-native edit. |
| `slim-parent-skill.patch` | Keep the parent skill concise and preserve local parent/child operating rules; packaged line-count and phrase checks witness it. | Still absent: target still carries the long upstream skill and the old whole-file patch no longer fits. | Reimplement the maintained concise skill against target documentation. |
| `detached-runner-peer-isolation.patch` | Avoid loading a peer coding-agent module in the detached runner; config-directory resolution witnesses it. | Fully absorbed and extended: target has no eager peer import and resolves package metadata explicitly. | Drop after the target tests and detached-runner package witness pass. |
| `async-runner-stderr.patch` | Preserve bounded detached-runner diagnostics; package and async failure witnesses cover the log. | Partially absorbed: target records stdout/stderr and exposes a diagnostic tail, but the local bounded-file behavior is not identical. | Reimplement only any still-required bound, or drop that remainder if upstream tailing satisfies the established purpose. |
| `full-child-extension-bridge.patch` | Preserve required child extensions and supervisor tools; the packaged bridge diagnostic witnesses it. | Partially absorbed: target natively injects supervisor tools and removes the bridge sandbox gate, but still uses `--no-extensions` for an explicit extension list. | Drop absorbed bridge code and reimplement only extension inheritance still required by the witness. |
| `acceptance-read-only-evidence.patch` | Accept explicit empty change/test arrays for read-only work while rejecting missing fields; its added unit tests witness it. | Still absent; the source and test patch applies forward to the target. | Rebase and run the acceptance tests. |

## Result

This is a mixed technical reconciliation: two absorbed portions can drop, one patch can rebase, and three deltas need target-native remainder decisions or reimplementation. The update must not be represented as a clean version bump. No psyche escalation is warranted by this dry run because the unresolved work is source and witness analysis, not an authority or value choice.
