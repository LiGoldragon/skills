# Skill — testing

## Tests prove the edited contract

Choose tests that witness the behavior or invariant changed. Prefer small deterministic checks near the contract over broad smoke tests that can pass while the edited rule is broken.

## Nix is the normal test gate

Expose durable project checks through the flake. Pure checks belong in flake checks. Test-only binaries use clear test names and do not become runtime services.

Run the narrow check that proves the edit, then the broader gate required by the repo before commit. Build that single check directly (`nix build .#checks.<system>.<check>`); reserve full `nix flake check` for pre-commit, and run whole-engine sweeps dry-run first with background builds and logs on disk. Record exact failures when a check cannot run.

Do not substitute an unrelated passing check for the one that proves the edited surface.

## Keep state explicit

Stateful tests name their resources, host requirements, cleanup, and failure artifacts. They do not depend on hidden local state. If a test needs credentials or hardware, mark the requirement and provide a safe skip or manual gate. Place a daemon-sandbox unix socket under a short run root (e.g. `/tmp/<lane>/`) so its path stays under the SUN_LEN limit; never nest the socket under the deep session scratchpad path.

## Test architecture, not just regression

When the change protects architecture, write tests for forbidden dependencies, protocol compatibility, ownership boundaries, and invariant preservation. A compile-fail or metadata check can be the right test when runtime behavior is not the contract.

## Put tests in owned locations

Keep unit tests close to small pure logic. Put integration tests at crate or component boundaries. Keep generated-surface tests at the generator source, not by patching emitted output.

## Name tests by behavior

Test names state the behavior under proof. Avoid names that only repeat the function name or issue number.

## Failure output is part of the interface

Assertions should say what invariant failed and show the relevant values. Do not dump secrets or large unrelated state.

## Keep fixtures minimal

Fixtures carry only the state needed to prove the case. Prefer named builders or typed records over copied blobs. Internal test, eval, and diagnostic artifacts use typed Rust records with NOTA projection; non-NOTA text fixtures name the external consumer or protocol that requires that exact format. When a fixture encodes compatibility, state the compatibility boundary in the test name or nearby assertion.

## Do not weaken tests to pass

If a test fails because the contract changed, update the contract and test together. If it fails because the implementation is wrong, fix the implementation. Do not delete coverage without replacing the proof.
