# Skill — versioning

## Rules

Whenever behavior changes, update the version surface that names that behavior in the same change. Do not leave version bumps for later cleanup.

Bump only the affected surface:

- component release version;
- wire contract version or handshake value;
- storage schema version or migration guard;
- package, wrapper, or deployment slot version.

Docs-only changes do not bump versions unless the prose is a runtime-visible help or public documentation surface that ships as part of the component.

## Semver before 1.0

Use `major.minor.patch`.

- Patch: compatible fixes, internal behavior corrections, non-breaking improvements.
- Minor: new public behavior, new operation roots, compatible wire additions, deploy-slot changes, forward-managed migrations, or breaking public changes before 1.0.
- Major: requires explicit psyche authorization.

When a daemon consumes a changed contract, bump the daemon too.

## Check

Before finishing: inspect the package metadata, run the relevant tests, verify any exposed `--version` or wrapper surface, update downstream locks when the component is consumed elsewhere, and state whether the new version is only landed or also deployed.

If patch/minor/major, wire/storage/release, or landed/deployed status is unclear, ask instead of guessing.
