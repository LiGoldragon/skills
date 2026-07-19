# Skill — Pi extension updates

Treat a Pi extension update as maintained-fork reconciliation, not a version bump. Keep installed/profile/store outputs read-only evidence; change the fork and declarative Nix owner, never effective runtime files.

## Compact flow

1. Establish primary/live sources and recent upstream activity. Record upstream, deployed fork revision, base, target, consumer pin, and activation path.
2. Extract mechanisms and tests from upstream; do not copy its brands, role names, or prompt bulk. Compare each local delta for overlap and local compatibility.
3. Make a semantic Jujutsu patch stack in an isolated workspace. For every delta decide: upstream, drop, carry/reimplement, or escalate. An automated proposal may aid comparison but must never blind-merge.
4. Validate the pristine target and reconciled result with focused tests, full relevant tests, package build, generated surfaces, and a runtime load/smoke witness. Record a failing pristine gate separately rather than weakening the reconciled gate.
5. Push the producer revision before updating a consumer pin. Reconcile the Nix/Home Manager source, build it, and deploy only after checks pass. Confirm the activated package is the pinned revision.

## Decisions and retirement

Carry only a small local behavior with a rationale and witness. Drop it only when upstream owns the behavior and the unmodified target passes its witness. Escalate only an authority, privacy, safety, or user-visible tradeoff; ordinary archaeology and patch repair stay with the worker.

A fork retires when the upstream target passes every retained witness without local deltas and the consumer can pin upstream directly. Re-audit whenever upstream activity, a deployment failure, or a changed local requirement invalidates the prior comparison.

Use `repository-management`, `version-control`, `feature-development`, `orchestration`, `helper-context-transfer`, and `design-quality` for their owning mechanics. Keep branch recipes and ledger detail out of ordinary dispatch prompts.
