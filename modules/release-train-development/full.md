# Skill — release train development

## Purpose

Use this skill when one feature spans independently locked repositories. A
release train makes one integration closure reproducible without treating a
branch name, universal Cargo.lock, local path, or cache as authority.

## Author

Author one `release-trains/<name>.nota` intent per epic. It names component
selectors, exact expected bases, and only explicitly admitted immutable
external components. Keep operational Synchronizer configuration separate.

Intent constrains discovered manifests; it never declares or overrides Cargo
or flake topology. A discovered internal edge outside the train, a missing
member, an unadmitted external, an expected-base mismatch, or selector movement
fails loudly.

## Resolve and materialize

Resolve every selector to a pushed commit before creating evidence. Emit an
immutable typed closure with a domain-separated identity. Candidate branches
are scoped `train/<name>` and are integration artifacts only: never write
mainline, worker branches, deployment pins, or production state.

Materialize each consumer from its exact source tree. Generate that component's
valid `Cargo.toml`/`Cargo.lock` changes through Cargo-aware resolution and its
own `flake.lock` through pinned commit plus narHash evidence. Cargo and Nix
locks are distinct domains; do not merge them and do not expect flake overrides
to alter Cargo Git locks.

Generate canonical `release-train.lock.json` only as a Textual JSON projection
of the resolved closure. An integration flake reads it and fetches only exact
commit/narHash inputs. No local paths may remain at the portable gate.

## Verify and hand off

Verify pushed candidate commits with the integration closure, recording exact
commits, lock identities, narHashes, closure identity, and check outcomes. A
failed train is discarded through its dedicated candidate branches; it has no
merge authority.

Do not build a shared compiled-artifact cache speculatively. First measure
matching Nix derivation keys. A future crate-source index is immutable,
Cargo-validated source materialization keyed by registry checksum or Git
commit; it is never a mutable latest-dependency registry.

Bootstrap JSON may use a deterministic adapter. Replace it with TextualJson
only when the shared structural-form boundary is ready, preserving the typed
closure and byte-stable JSON fixtures.
